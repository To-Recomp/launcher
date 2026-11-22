//! Installations and instances.
//!
//! One **installation** per game: a release of the game's repository, downloaded, checked and
//! unpacked. One **instance** per platform of that game, with its own data folder,
//! settings and saves. The player gives their dump once, to the instance's *prepare* step;
//! afterwards the game starts without it. See `store` for the layout.

mod release;
mod run;
mod store;

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::settings::manifest::{Input, Manifest};
use crate::settings::Location;
pub use release::CANCELLED;
use run::{Context, Progress, RunError};
use store::{InstallationRecord, InstanceState, Library};

const DEFAULT_API: &str = "https://api.github.com";
/// Environment overrides for development and tests.
const LIBRARY_ENV: &str = "TO_RECOMP_LIBRARY";
const API_ENV: &str = "TO_RECOMP_GITHUB_API";
/// A step starts only with this much room on the game's drive.
const MIN_FREE_BYTES: u64 = 1 << 30;

/// Everything the operations depend on, so tests can point them at a temp folder and a fake
/// release server.
pub struct Env {
    pub library: Library,
    pub api: String,
    pub os: &'static str,
    pub launcher_version: &'static str,
    pub min_free_bytes: u64,
}

impl Env {
    pub fn new(root: PathBuf, api: String) -> Env {
        Env {
            library: Library::new(root),
            api,
            os: std::env::consts::OS,
            launcher_version: env!("CARGO_PKG_VERSION"),
            min_free_bytes: MIN_FREE_BYTES,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstallationView {
    pub game_id: String,
    pub repository: String,
    pub tag: String,
    /// What the player is asked for, from the release's `recomplauncher.json`.
    pub inputs: Vec<InputView>,
    pub prepare_label: Option<String>,
    /// False when the release runs as it is, with nothing to prepare.
    pub can_prepare: bool,
    /// A rebuild after an update starts from what the last run kept, not from the dump.
    pub keeps_copy: bool,
    pub instances: Vec<InstanceView>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputView {
    pub id: String,
    pub label: String,
    pub role: String,
    pub optional: bool,
    /// `file` and/or `folder`.
    pub kinds: Vec<String>,
    /// File extensions without the dot; empty for any.
    pub extensions: Vec<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstanceView {
    pub id: String,
    pub platform: String,
    pub state: InstanceState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub job: String,
    pub fraction: Option<f64>,
    pub message: String,
}

fn read_manifest(release_dir: &Path) -> Result<Manifest, String> {
    let path = release_dir.join("recomplauncher.json");
    let raw = fs::read_to_string(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => {
            "This release has no recomplauncher.json, so the launcher doesn't know how to run it."
                .to_string()
        }
        _ => format!("Can't read {}: {e}", path.display()),
    })?;
    let manifest = Manifest::parse(&raw)?;
    if manifest.start.is_none() {
        return Err("Invalid recomplauncher.json: it has no `start` command.".to_string());
    }
    Ok(manifest)
}

fn input_view(input: &Input) -> InputView {
    let mut kinds: Vec<String> = Vec::new();
    let mut extensions: Vec<String> = Vec::new();
    for format in &input.formats {
        if !kinds.contains(&format.kind) {
            kinds.push(format.kind.clone());
        }
        extensions.extend(format.extensions.iter().cloned());
    }
    if kinds.is_empty() {
        kinds.push("file".to_string());
    }
    InputView {
        id: input.id.clone(),
        label: input.label.clone().unwrap_or_else(|| input.id.clone()),
        role: input.role().to_string(),
        optional: input.optional,
        kinds,
        extensions,
    }
}

fn installation_view(
    env: &Env,
    game_id: &str,
    record: &InstallationRecord,
) -> Result<InstallationView, String> {
    let manifest = read_manifest(&env.library.release_dir(game_id)?)?;
    let mut instances = Vec::new();
    for (id, instance) in env.library.instances(game_id)? {
        let location = env.library.location(game_id, &id)?;
        instances.push(InstanceView {
            state: store::instance_state(&manifest, record, &instance, &location),
            id,
            platform: instance.platform,
        });
    }
    Ok(InstallationView {
        game_id: game_id.to_string(),
        repository: record.repository.clone(),
        tag: record.tag.clone(),
        inputs: manifest.inputs.iter().map(input_view).collect(),
        prepare_label: manifest.prepare.as_ref().and_then(|p| p.label.clone()),
        can_prepare: manifest.prepare.is_some() || !manifest.inputs.is_empty(),
        keeps_copy: manifest.prepare.as_ref().is_some_and(|p| p.keeps_copy),
        instances,
    })
}

/// The installed games and their instances. A damaged installation is skipped, not fatal.
pub fn snapshot(env: &Env) -> Vec<InstallationView> {
    env.library
        .installed_games()
        .into_iter()
        .filter_map(|id| {
            let record = env.library.installation(&id).ok()??;
            installation_view(env, &id, &record).ok()
        })
        .collect()
}

/// Downloads, verifies and unpacks the newest release of `repository`. Instances are kept.
pub fn install(
    env: &Env,
    game_id: &str,
    repository: &str,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(Progress),
) -> Result<InstallationView, String> {
    let game_dir = env.library.game_dir(game_id)?;
    progress(Progress {
        fraction: None,
        message: "Looking for the latest release".into(),
    });
    let release = release::latest_release(&env.api, repository, env.os)?;

    if let Some(current) = env.library.installation(game_id)? {
        if current.tag == release.tag && env.library.release_dir(game_id)?.exists() {
            progress(Progress {
                fraction: Some(1.0),
                message: format!("{} is already installed", release.tag),
            });
            return installation_view(env, game_id, &current);
        }
    }

    let work = game_dir.join(".installing");
    let _ = fs::remove_dir_all(&work);
    let result = download_and_unpack(env, game_id, repository, &release, &work, cancel, progress);
    let _ = fs::remove_dir_all(&work);
    result
}

fn download_and_unpack(
    env: &Env,
    game_id: &str,
    repository: &str,
    release: &release::Release,
    work: &Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(Progress),
) -> Result<InstallationView, String> {
    let archive = work.join(&release.asset.name);
    let message = format!("Downloading {}", release.asset.name);
    release::download(
        &release.asset.browser_download_url,
        &archive,
        &release.sha256,
        cancel,
        &mut |done, total| {
            let fraction = total
                .filter(|t| *t > 0)
                .map(|t| 0.9 * done as f64 / t as f64);
            progress(Progress {
                fraction,
                message: message.clone(),
            });
        },
    )?;

    progress(Progress {
        fraction: Some(0.92),
        message: "Unpacking".into(),
    });
    let unpacked = work.join("release");
    release::extract(&archive, &unpacked)?;
    read_manifest(&unpacked)?; // a release the launcher can't run is never installed

    let release_dir = env.library.release_dir(game_id)?;
    let old = work.join("previous");
    if release_dir.exists() {
        fs::rename(&release_dir, &old)
            .map_err(|e| format!("Can't replace the installed release: {e}"))?;
    }
    if let Err(e) = fs::rename(&unpacked, &release_dir) {
        if old.exists() {
            let _ = fs::rename(&old, &release_dir);
        }
        return Err(format!("Can't install the release: {e}"));
    }
    let record = InstallationRecord {
        repository: repository.to_string(),
        tag: release.tag.clone(),
        asset: release.asset.name.clone(),
        sha256: release.sha256.clone(),
    };
    env.library.save_installation(game_id, &record)?;
    progress(Progress {
        fraction: Some(1.0),
        message: format!("Installed {}", release.tag),
    });
    installation_view(env, game_id, &record)
}

/// Checks the files the player picked against the manifest's inputs.
fn validate_inputs(manifest: &Manifest, picked: &BTreeMap<String, PathBuf>) -> Result<(), String> {
    for (id, path) in picked {
        let input = manifest
            .inputs
            .iter()
            .find(|i| &i.id == id)
            .ok_or_else(|| format!("This game doesn't ask for `{id}`."))?;
        let label = input.label.as_deref().unwrap_or(id);
        let meta = fs::metadata(path)
            .map_err(|_| format!("{label}: {} doesn't exist.", path.display()))?;
        let view = input_view(input);
        let kind = if meta.is_dir() { "folder" } else { "file" };
        if !view.kinds.iter().any(|k| k == kind) {
            return Err(format!(
                "{label}: pick a {} here, not a {kind}.",
                view.kinds.join(" or ")
            ));
        }
        if meta.is_file() && !view.extensions.is_empty() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !view.extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
                return Err(format!(
                    "{label}: expected a .{} file.",
                    view.extensions.join(" or .")
                ));
            }
        }
    }
    Ok(())
}

/// Runs the release's prepare step for one instance, once the player gave their files; with
/// `keeps_copy` a rebuild after an update needs none.
pub fn prepare(
    env: &Env,
    game_id: &str,
    platform: &str,
    picked: BTreeMap<String, PathBuf>,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(Progress),
) -> Result<InstanceView, String> {
    let library = &env.library;
    let record = library
        .installation(game_id)?
        .ok_or("This game isn't installed yet.")?;
    let release_dir = library.release_dir(game_id)?;
    let manifest = read_manifest(&release_dir)?;
    let id = store::instance_id(platform);
    let location = library.location(game_id, &id)?;
    let mut instance = library.instance_record(game_id, &id, platform)?;

    validate_inputs(&manifest, &picked)?;
    let rebuild_from_kept = manifest.prepare.as_ref().is_some_and(|p| p.keeps_copy)
        && picked.is_empty()
        && instance.prepared_release.is_some();

    // The player's files: earlier picks that still exist, overridden by new ones.
    let mut inputs: BTreeMap<String, PathBuf> = instance
        .inputs
        .iter()
        .map(|(k, v)| (k.clone(), PathBuf::from(v)))
        .filter(|(_, p)| p.exists())
        .collect();
    inputs.extend(picked);

    fs::create_dir_all(&location.data)
        .map_err(|e| format!("Can't create {}: {e}", location.data.display()))?;

    if let Some(prepare) = &manifest.prepare {
        let free = fs4::available_space(&location.data).unwrap_or(u64::MAX);
        if free < env.min_free_bytes {
            return Err(format!(
                "Not enough free space: the preparation needs at least {} GB on this drive.",
                env.min_free_bytes.div_ceil(1 << 30)
            ));
        }
        let ctx = Context {
            location: &location,
            manifest: &manifest,
            inputs: &inputs,
            without_copy: rebuild_from_kept,
        };
        let launch = run::build(
            &ctx,
            &prepare.program,
            &prepare.args,
            &prepare.env,
            env.launcher_version,
        )?;
        let tmp = release_dir.join(".recomplauncher-tmp");
        let log = library.instance_dir(game_id, &id)?.join("prepare.log");
        progress(Progress {
            fraction: None,
            message: prepare
                .label
                .clone()
                .unwrap_or_else(|| "Preparing the game".into()),
        });
        run::run_prepare(&launch, &release_dir, &tmp, &log, cancel, progress).map_err(
            |e| match e {
                RunError::Cancelled => CANCELLED.to_string(),
                RunError::Failed(message) => message,
            },
        )?;
    } else if let Some(missing) = manifest
        .inputs
        .iter()
        .find(|i| !i.optional && !inputs.contains_key(&i.id))
    {
        return Err(format!(
            "Missing file: {}.",
            missing.label.as_deref().unwrap_or(&missing.id)
        ));
    }

    instance.inputs = inputs
        .iter()
        .map(|(k, v)| (k.clone(), v.to_string_lossy().into_owned()))
        .collect();
    instance.prepared_release = Some(record.tag.clone());
    library.save_instance_record(game_id, &id, &instance)?;

    let state = store::instance_state(&manifest, &record, &instance, &location);
    if state != InstanceState::Ready {
        return Err("The preparation finished but the game's files aren't all there. The log has the details.".to_string());
    }
    progress(Progress {
        fraction: Some(1.0),
        message: "Ready to play".into(),
    });
    Ok(InstanceView {
        id,
        platform: platform.to_string(),
        state,
    })
}

/// Starts the game of a ready instance, without the dump. `on_exit` gets the exit code.
pub fn start(
    env: &Env,
    game_id: &str,
    platform: &str,
    on_exit: impl FnOnce(Option<i32>) + Send + 'static,
) -> Result<(), String> {
    let library = &env.library;
    let record = library
        .installation(game_id)?
        .ok_or("This game isn't installed yet.")?;
    let manifest = read_manifest(&library.release_dir(game_id)?)?;
    let id = store::instance_id(platform);
    let location = library.location(game_id, &id)?;
    let instance = library.instance_record(game_id, &id, platform)?;
    if store::instance_state(&manifest, &record, &instance, &location) != InstanceState::Ready {
        return Err("This game isn't ready: prepare it first.".to_string());
    }

    let inputs: BTreeMap<String, PathBuf> = instance
        .inputs
        .iter()
        .map(|(k, v)| (k.clone(), PathBuf::from(v)))
        .collect();
    let ctx = Context {
        location: &location,
        manifest: &manifest,
        inputs: &inputs,
        without_copy: false,
    };
    let start = manifest
        .start
        .as_ref()
        .expect("read_manifest checks `start`");
    let mut launch = run::build(
        &ctx,
        &start.program,
        &start.args,
        &start.env,
        env.launcher_version,
    )?;
    // The player's own choices (settings that go on the command line) come last.
    launch.args.extend(crate::settings::launch_args(&location)?);

    fs::create_dir_all(&location.data)
        .map_err(|e| format!("Can't create {}: {e}", location.data.display()))?;
    let log = library.instance_dir(game_id, &id)?.join("game.log");
    let mut child = run::spawn_game(&launch, &location.install, &log)?;
    std::thread::spawn(move || {
        let code = child.wait().ok().and_then(|s| s.code());
        run::append_log(
            &log,
            &format!(
                "-- exited with code {}",
                code.map_or("none".to_string(), |c| c.to_string())
            ),
        );
        on_exit(code);
    });
    Ok(())
}

pub fn read_log(
    env: &Env,
    game_id: &str,
    platform: &str,
    kind: &str,
    max_lines: usize,
) -> Result<String, String> {
    let file = match kind {
        "prepare" => "prepare.log",
        "game" => "game.log",
        other => return Err(format!("There is no `{other}` log.")),
    };
    let dir = env
        .library
        .instance_dir(game_id, &store::instance_id(platform))?;
    Ok(run::tail(&dir.join(file), max_lines))
}

/// Running jobs by key (`install:<game>`, `prepare:<game>/<instance>`, `game:<game>/<instance>`):
/// each holds the flag that cancels it, and a key can only run once at a time.
#[derive(Clone, Default)]
pub struct Jobs(Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>);

pub struct JobGuard {
    jobs: Jobs,
    key: String,
    pub cancel: Arc<AtomicBool>,
}

impl Jobs {
    pub fn begin(&self, key: &str) -> Result<JobGuard, String> {
        let mut jobs = self.0.lock().unwrap();
        if jobs.contains_key(key) {
            return Err("That is already running.".to_string());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        jobs.insert(key.to_string(), Arc::clone(&cancel));
        Ok(JobGuard {
            jobs: self.clone(),
            key: key.to_string(),
            cancel,
        })
    }

    pub fn cancel(&self, key: &str) {
        if let Some(flag) = self.0.lock().unwrap().get(key) {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

impl Drop for JobGuard {
    fn drop(&mut self) {
        self.jobs.0.lock().unwrap().remove(&self.key);
    }
}

pub(crate) fn app_env(app: &tauri::AppHandle) -> Result<Env, String> {
    let root = match std::env::var_os(LIBRARY_ENV) {
        Some(root) => PathBuf::from(root),
        None => app
            .path()
            .app_data_dir()
            .map_err(|e| format!("No place to keep the games: {e}"))?
            .join("games"),
    };
    let api = std::env::var(API_ENV).unwrap_or_else(|_| DEFAULT_API.to_string());
    Ok(Env::new(root, api))
}

fn reporter(app: &tauri::AppHandle, job: &str) -> impl FnMut(Progress) {
    let app = app.clone();
    let job = job.to_string();
    move |p: Progress| {
        let _ = app.emit(
            "library-progress",
            ProgressEvent {
                job: job.clone(),
                fraction: p.fraction,
                message: p.message,
            },
        );
    }
}

#[tauri::command(async)]
pub fn library_snapshot(app: tauri::AppHandle) -> Result<Vec<InstallationView>, String> {
    Ok(snapshot(&app_env(&app)?))
}

#[tauri::command(async)]
pub fn install_game(
    app: tauri::AppHandle,
    jobs: tauri::State<Jobs>,
    game_id: String,
    repository: String,
) -> Result<InstallationView, String> {
    let env = app_env(&app)?;
    let key = format!("install:{game_id}");
    let guard = jobs.begin(&key)?;
    install(
        &env,
        &game_id,
        &repository,
        &guard.cancel,
        &mut reporter(&app, &key),
    )
}

#[tauri::command(async)]
pub fn prepare_instance(
    app: tauri::AppHandle,
    jobs: tauri::State<Jobs>,
    game_id: String,
    platform: String,
    inputs: BTreeMap<String, String>,
) -> Result<InstanceView, String> {
    let env = app_env(&app)?;
    let key = format!("prepare:{game_id}/{}", store::instance_id(&platform));
    let guard = jobs.begin(&key)?;
    let picked = inputs
        .into_iter()
        .map(|(k, v)| (k, PathBuf::from(v)))
        .collect();
    prepare(
        &env,
        &game_id,
        &platform,
        picked,
        &guard.cancel,
        &mut reporter(&app, &key),
    )
}

#[tauri::command(async)]
pub fn start_game(
    app: tauri::AppHandle,
    jobs: tauri::State<Jobs>,
    game_id: String,
    platform: String,
) -> Result<(), String> {
    let env = app_env(&app)?;
    let instance = store::instance_id(&platform);
    let key = format!("game:{game_id}/{instance}");
    let guard = jobs.begin(&key)?;
    let exited = app.clone();
    start(&env, &game_id, &platform, move |code| {
        let _ = exited.emit(
            "library-game-exited",
            serde_json::json!({ "job": key, "code": code }),
        );
        drop(guard); // the game can be started again
    })
}

/// Stops an install or a preparation.
#[tauri::command]
pub fn cancel_job(jobs: tauri::State<Jobs>, job: String) {
    jobs.cancel(&job);
}

#[tauri::command(async)]
pub fn read_instance_log(
    app: tauri::AppHandle,
    game_id: String,
    platform: String,
    kind: String,
) -> Result<String, String> {
    read_log(&app_env(&app)?, &game_id, &platform, &kind, 200)
}

/// Where the settings of an instance live, once its game is installed.
#[tauri::command(async)]
pub fn instance_location(
    app: tauri::AppHandle,
    game_id: String,
    platform: String,
) -> Result<Option<Location>, String> {
    let env = app_env(&app)?;
    if env.library.installation(&game_id)?.is_none() {
        return Ok(None);
    }
    let location = env
        .library
        .location(&game_id, &store::instance_id(&platform))?;
    Ok(Some(Location {
        install_dir: location.install.to_string_lossy().into_owned(),
        data_dir: Some(location.data.to_string_lossy().into_owned()),
    }))
}

/// The last `max_lines` lines of a log file.
pub fn tail_of(path: &Path, max_lines: usize) -> String {
    run::tail(path, max_lines)
}

/// What a diagnostic file says about one instance, and the paths it must hide. A game that failed
/// to install has no release: the subject then only knows the library.
pub fn diagnostic_subject(
    env: &Env,
    game_id: &str,
    platform: &str,
    log_file: Option<&str>,
) -> Result<crate::diagnostics::Subject, String> {
    let instance_id = store::instance_id(platform);
    let mut paths = vec![(
        env.library.root().to_string_lossy().into_owned(),
        "{library}".to_string(),
    )];
    let mut game = crate::diagnostics::Game {
        id: game_id.to_string(),
        release: None,
        platform: platform.to_string(),
        state: "not installed".to_string(),
    };
    if let Some(record) = env.library.installation(game_id)? {
        let location = env.library.location(game_id, &instance_id)?;
        paths.push((
            location.data.to_string_lossy().into_owned(),
            "{data}".to_string(),
        ));
        paths.push((
            location.install.to_string_lossy().into_owned(),
            "{install}".to_string(),
        ));
        game.release = Some(record.tag.clone());
        game.state = "unknown".to_string();
        let instance = env
            .library
            .instances(game_id)?
            .into_iter()
            .find(|(id, _)| *id == instance_id)
            .map(|(_, instance)| instance);
        if let Some(instance) = instance {
            for (id, path) in &instance.inputs {
                paths.push((path.clone(), format!("{{input:{id}}}")));
            }
            if let Ok(manifest) = read_manifest(&env.library.release_dir(game_id)?) {
                game.state =
                    match store::instance_state(&manifest, &record, &instance, &location) {
                        InstanceState::NotPrepared => "not prepared",
                        InstanceState::Ready => "ready",
                        InstanceState::NeedsRebuild => "needs rebuild",
                    }
                    .to_string();
            }
        } else {
            game.state = "not prepared".to_string();
        }
    }
    let log = log_file.map(|file| {
        env.library
            .instance_dir(game_id, &instance_id)
            .map(|dir| dir.join(file))
    });
    Ok(crate::diagnostics::Subject {
        game,
        paths,
        log: log.transpose()?,
    })
}

#[cfg(test)]
mod tests;
