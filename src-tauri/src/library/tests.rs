use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tempfile::TempDir;
use tiny_http::{Response, Server};

use super::*;
use crate::settings::{Instance, SettingInput};

const REPO: &str = "To-Recomp/fake-game";
const GAME: &str = "fake";
static NO_CANCEL: AtomicBool = AtomicBool::new(false);

fn exe_name() -> String {
    format!("fake-game{}", std::env::consts::EXE_SUFFIX)
}

/// The fake game (`examples/fake_game.rs`), built once per test run with the toolchain that
/// runs the tests: where cargo puts examples differs between versions and platforms.
fn fake_exe() -> Vec<u8> {
    static EXE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    let path = EXE.get_or_init(|| {
        let output = std::process::Command::new(env!("CARGO"))
            .args([
                "build",
                "--example",
                "fake_game",
                "--message-format=json",
                "--manifest-path",
            ])
            .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
            .output()
            .expect("run cargo");
        assert!(
            output.status.success(),
            "building the fake game failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .find_map(|message| {
                (message["reason"] == "compiler-artifact"
                    && message["target"]["name"] == "fake_game")
                    .then(|| message["executable"].as_str().map(PathBuf::from))
                    .flatten()
            })
            .expect("cargo reports the fake game's executable")
    });
    fs::read(path).unwrap()
}

fn manifest(extra: &str) -> String {
    let exe = exe_name();
    format!(
        r#"{{
  "format": 1,
  "inputs": [
    {{ "id": "disc1", "label": "Disc 1", "role": "disc", "formats": [{{ "type": "file", "extensions": ["iso"] }}] }},
    {{ "id": "disc2", "label": "Disc 2", "role": "disc", "formats": [{{ "type": "file", "extensions": ["iso"] }}] }},
    {{ "id": "dlc", "label": "DLC", "role": "dlc", "optional": true, "formats": [{{ "type": "file" }}] }}
  ],
  "prepare": {{
    "program": ["{exe}"],
    "args": ["--prepare", "--data", "{{data}}", "--disc={{input:disc1}}", "--disc={{input:disc2}}", "--dlc={{input:dlc}}"],
    "label": "Building the game",
    "done": ["{{data}}/built.bin"],
    "keeps_copy": true
  }},
  "start": {{ "program": ["{exe}"], "args": ["--play", "--data", "{{data}}"], "env": {{ "FAKE_VAR": "{{data}}/x" }} }},
  "settings": [
    {{ "id": "fs", "label": "Fullscreen", "in": "args",
       "choices": [ {{ "label": "Off", "args": [] }}, {{ "label": "On", "args": ["--fullscreen"] }} ] }}
  ]{extra}
}}"#
    )
}

/// A release archive; `files` are (path, bytes, executable).
fn zip_bytes(files: &[(&str, Vec<u8>, bool)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes, exec) in files {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(if *exec { 0o755 } else { 0o644 });
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn good_release(manifest_json: &str) -> Vec<u8> {
    zip_bytes(&[
        (
            "recomplauncher.json",
            manifest_json.as_bytes().to_vec(),
            false,
        ),
        (&exe_name(), fake_exe(), true),
    ])
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A fake GitHub: the release API and the asset downloads.
type Routes = Arc<Mutex<BTreeMap<String, (u16, Vec<u8>)>>>;

struct Fake {
    url: String,
    routes: Routes,
    hits: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
}

impl Fake {
    fn new() -> Fake {
        let server = Server::http("127.0.0.1:0").unwrap();
        let url = format!("http://{}", server.server_addr().to_ip().unwrap());
        let routes: Routes = Arc::default();
        let hits: Arc<Mutex<Vec<String>>> = Arc::default();
        let stop = Arc::new(AtomicBool::new(false));
        let (r, h, s) = (Arc::clone(&routes), Arc::clone(&hits), Arc::clone(&stop));
        thread::spawn(move || {
            while !s.load(Ordering::Relaxed) {
                let Ok(Some(request)) = server.recv_timeout(Duration::from_millis(50)) else {
                    continue;
                };
                h.lock().unwrap().push(request.url().to_string());
                let (status, body) = r
                    .lock()
                    .unwrap()
                    .get(request.url())
                    .cloned()
                    .unwrap_or((404, Vec::new()));
                let _ = request.respond(Response::from_data(body).with_status_code(status));
            }
        });
        Fake {
            url,
            routes,
            hits,
            stop,
        }
    }

    fn route(&self, path: &str, status: u16, body: Vec<u8>) {
        self.routes
            .lock()
            .unwrap()
            .insert(path.to_string(), (status, body));
    }

    /// Publishes `tag` with `archive` as the asset for this OS.
    fn publish(&self, tag: &str, archive: &[u8], digest: Option<String>) {
        let name = format!("fake-{tag}-{}-x86_64.zip", std::env::consts::OS);
        let digest = digest.map(|d| format!("sha256:{d}"));
        let body = serde_json::json!({
            "tag_name": tag,
            "assets": [{ "name": name, "size": archive.len(), "browser_download_url": format!("{}/download/{name}", self.url), "digest": digest }],
        });
        self.route(
            &format!("/repos/{REPO}/releases/latest"),
            200,
            body.to_string().into_bytes(),
        );
        self.route(&format!("/download/{name}"), 200, archive.to_vec());
    }

    fn publish_ok(&self, tag: &str, archive: &[u8]) {
        self.publish(tag, archive, Some(sha(archive)));
    }

    fn downloads(&self) -> usize {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|u| u.starts_with("/download/"))
            .count()
    }
}

impl Drop for Fake {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

struct Setup {
    _dir: TempDir,
    root: PathBuf,
    env: Env,
    fake: Fake,
}

fn setup() -> Setup {
    let dir = TempDir::new().unwrap();
    let fake = Fake::new();
    let root = dir.path().join("library");
    let mut env = Env::new(root.clone(), fake.url.clone());
    env.min_free_bytes = 1; // the CI drive is not the player's
    Setup {
        _dir: dir,
        root,
        env,
        fake,
    }
}

fn do_install(s: &Setup) -> Result<InstallationView, String> {
    install(&s.env, GAME, REPO, &NO_CANCEL, &mut |_| {})
}

fn file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, content).unwrap();
    path
}

fn discs(dir: &Path, one: &str, two: &str) -> BTreeMap<String, PathBuf> {
    BTreeMap::from([
        ("disc1".to_string(), file(dir, "d1.iso", one)),
        ("disc2".to_string(), file(dir, "d2.iso", two)),
    ])
}

fn do_prepare(s: &Setup, picked: BTreeMap<String, PathBuf>) -> Result<InstanceView, String> {
    prepare(&s.env, GAME, "GameCube", picked, &NO_CANCEL, &mut |_| {})
}

fn installed_and_published(s: &Setup) {
    s.fake.publish_ok("v1.0.0", &good_release(&manifest("")));
    do_install(s).unwrap();
}

fn instance_location(s: &Setup) -> Instance {
    s.env
        .library
        .location(GAME, &store::instance_id("GameCube"))
        .unwrap()
}

// ---- release assets ----

fn asset(name: &str) -> release::ApiAsset {
    release::ApiAsset {
        name: name.to_string(),
        size: 1,
        browser_download_url: format!("https://x/{name}"),
        digest: None,
    }
}

#[test]
fn picks_the_archive_made_for_this_os() {
    let assets = [
        asset("game-1.0-windows-x86_64.zip"),
        asset("game-1.0-linux-aarch64.tar.gz"),
        asset("game-1.0-linux-x86_64.tar.gz"),
        asset("game-1.0-linux.zip"),
        asset("game-1.0-linux-x86_64.AppImage"),
        asset("SHA256SUMS"),
    ];
    assert_eq!(
        release::pick_asset(&assets, "linux").unwrap().name,
        "game-1.0-linux-x86_64.tar.gz"
    );
    assert_eq!(
        release::pick_asset(&assets, "windows").unwrap().name,
        "game-1.0-windows-x86_64.zip"
    );
    assert!(release::pick_asset(&assets, "macos")
        .unwrap_err()
        .contains("macos"));
    assert!(release::pick_asset(&[asset("game-1.0-linux-arm64.zip")], "linux").is_err());
}

// ---- install ----

#[test]
fn installs_a_release_and_reports_what_it_asks_for() {
    let s = setup();
    let archive = good_release(&manifest(""));
    s.fake.publish_ok("v1.0.0", &archive);

    let mut events = Vec::new();
    let view = install(&s.env, GAME, REPO, &NO_CANCEL, &mut |p| events.push(p)).unwrap();

    assert_eq!((view.game_id.as_str(), view.tag.as_str()), (GAME, "v1.0.0"));
    assert_eq!(
        view.inputs
            .iter()
            .map(|i| i.id.as_str())
            .collect::<Vec<_>>(),
        ["disc1", "disc2", "dlc"]
    );
    assert_eq!(view.inputs[0].extensions, ["iso"]);
    assert!(view.inputs[2].optional && view.can_prepare && view.keeps_copy);
    assert_eq!(view.prepare_label.as_deref(), Some("Building the game"));
    assert!(view.instances.is_empty());

    let release = s.root.join(GAME).join("release");
    assert!(release.join("recomplauncher.json").is_file() && release.join(exe_name()).is_file());
    assert!(
        !s.root.join(GAME).join(".installing").exists(),
        "scratch files are removed"
    );
    let record = s.env.library.installation(GAME).unwrap().unwrap();
    assert_eq!(
        (record.tag.as_str(), record.sha256),
        ("v1.0.0", sha(&archive))
    );

    let fractions: Vec<f64> = events.iter().filter_map(|e| e.fraction).collect();
    assert!(fractions.windows(2).all(|w| w[0] <= w[1]), "{fractions:?}");
    assert_eq!(fractions.last(), Some(&1.0));
    assert_eq!(snapshot(&s.env), vec![view]);
}

#[cfg(unix)]
#[test]
fn the_unpacked_program_is_executable() {
    use std::os::unix::fs::PermissionsExt;
    let s = setup();
    installed_and_published(&s);
    let mode = fs::metadata(s.root.join(GAME).join("release").join(exe_name()))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o111, 0o111);
}

#[test]
fn installing_the_same_release_again_downloads_nothing() {
    let s = setup();
    installed_and_published(&s);
    assert_eq!(s.fake.downloads(), 1);
    let mut last = None;
    install(&s.env, GAME, REPO, &NO_CANCEL, &mut |p| last = Some(p)).unwrap();
    assert_eq!(s.fake.downloads(), 1);
    assert!(last.unwrap().message.contains("already installed"));
}

#[test]
fn a_download_that_doesnt_match_the_published_checksum_installs_nothing() {
    let s = setup();
    s.fake
        .publish("v1.0.0", &good_release(&manifest("")), Some("0".repeat(64)));
    let err = do_install(&s).unwrap_err();
    assert!(err.contains("doesn't match the checksum"), "{err}");
    assert!(s.env.library.installation(GAME).unwrap().is_none());
    assert!(!s.root.join(GAME).join("release").exists());
    assert!(!s.root.join(GAME).join(".installing").exists());
}

#[test]
fn a_release_without_a_published_checksum_is_refused() {
    let s = setup();
    s.fake.publish("v1.0.0", &good_release(&manifest("")), None);
    assert!(do_install(&s).unwrap_err().contains("no SHA-256 checksum"));
    assert_eq!(s.fake.downloads(), 0);
}

#[test]
fn a_repository_with_only_pre_releases_installs_its_newest() {
    let s = setup();
    let archive = good_release(&manifest(""));
    s.fake.publish_ok("v0.1.0-rc.1", &archive);
    let name = format!("fake-v0.1.0-rc.1-{}-x86_64.zip", std::env::consts::OS);
    let list = serde_json::json!([
        { "tag_name": "v0.2.0-draft", "draft": true, "assets": [] },
        { "tag_name": "v0.1.0-rc.1", "assets": [{ "name": name, "size": 1, "browser_download_url": format!("{}/download/{name}", s.fake.url), "digest": format!("sha256:{}", sha(&archive)) }] },
    ]);
    s.fake
        .route(&format!("/repos/{REPO}/releases/latest"), 404, Vec::new());
    s.fake.route(
        &format!("/repos/{REPO}/releases?per_page=10"),
        200,
        list.to_string().into_bytes(),
    );
    assert_eq!(do_install(&s).unwrap().tag, "v0.1.0-rc.1");
}

#[test]
fn an_unknown_repository_says_so() {
    let s = setup();
    assert!(do_install(&s)
        .unwrap_err()
        .contains("doesn't exist or has no releases"));
}

#[test]
fn unsafe_archives_and_unusable_releases_install_nothing() {
    let manifest_json = manifest("");
    let bad: Vec<(&str, Vec<u8>, &str)> = vec![
        (
            "path leaving the folder",
            zip_bytes(&[
                ("../evil.txt", b"x".to_vec(), false),
                (
                    "recomplauncher.json",
                    manifest_json.clone().into_bytes(),
                    false,
                ),
            ]),
            "unsafe path",
        ),
        (
            "no manifest",
            zip_bytes(&[(&exe_name(), fake_exe(), true)]),
            "no recomplauncher.json",
        ),
        (
            "manifest without start",
            zip_bytes(&[("recomplauncher.json", br#"{"format":1}"#.to_vec(), false)]),
            "no `start`",
        ),
        (
            "invalid manifest",
            zip_bytes(&[("recomplauncher.json", b"not json".to_vec(), false)]),
            "Invalid recomplauncher.json",
        ),
    ];
    for (what, archive, expected) in bad {
        let s = setup();
        s.fake.publish_ok("v1.0.0", &archive);
        let err = do_install(&s).unwrap_err();
        assert!(err.contains(expected), "{what}: {err}");
        assert!(
            s.env.library.installation(GAME).unwrap().is_none(),
            "{what}"
        );
        assert!(!s.root.join(GAME).join("release").exists(), "{what}");
    }
}

#[test]
fn a_release_wrapped_in_one_folder_is_unpacked_at_the_top() {
    let s = setup();
    let archive = zip_bytes(&[
        (
            "fake-1.0/recomplauncher.json",
            manifest("").into_bytes(),
            false,
        ),
        (&format!("fake-1.0/{}", exe_name()), fake_exe(), true),
    ]);
    s.fake.publish_ok("v1.0.0", &archive);
    do_install(&s).unwrap();
    assert!(s
        .root
        .join(GAME)
        .join("release")
        .join("recomplauncher.json")
        .is_file());
}

#[test]
fn tar_gz_archives_are_unpacked_and_links_refused() {
    let dir = TempDir::new().unwrap();
    let make = |entries: &dyn Fn(&mut tar::Builder<flate2::write::GzEncoder<fs::File>>)| {
        let path = dir.path().join("a.tar.gz");
        let gz = flate2::write::GzEncoder::new(
            fs::File::create(&path).unwrap(),
            flate2::Compression::fast(),
        );
        let mut builder = tar::Builder::new(gz);
        entries(&mut builder);
        builder.into_inner().unwrap().finish().unwrap();
        path
    };
    let archive = make(&|b| {
        let mut header = tar::Header::new_gnu();
        header.set_size(2);
        header.set_mode(0o644);
        header.set_cksum();
        b.append_data(&mut header, "recomplauncher.json", &b"{}"[..])
            .unwrap();
    });
    release::extract(&archive, &dir.path().join("out")).unwrap();
    assert!(dir.path().join("out").join("recomplauncher.json").is_file());

    let linked = make(&|b| {
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        header.set_cksum();
        b.append_link(&mut header, "link", "/etc/passwd").unwrap();
    });
    assert!(release::extract(&linked, &dir.path().join("out2"))
        .unwrap_err()
        .contains("link"));
}

// ---- prepare ----

#[test]
fn prepares_once_then_the_game_is_ready_without_the_discs() {
    let s = setup();
    installed_and_published(&s);
    let inputs_dir = TempDir::new().unwrap();

    let mut events = Vec::new();
    let view = prepare(
        &s.env,
        GAME,
        "GameCube",
        discs(inputs_dir.path(), "one", "two"),
        &NO_CANCEL,
        &mut |p| events.push(p),
    )
    .unwrap();

    assert_eq!(
        (view.id.as_str(), view.state),
        ("gamecube", InstanceState::Ready)
    );
    let location = instance_location(&s);
    assert_eq!(
        fs::read_to_string(location.data.join("built.bin")).unwrap(),
        "built from one+two"
    );
    let messages: Vec<&str> = events.iter().map(|e| e.message.as_str()).collect();
    assert!(
        messages.contains(&"Reading the discs") && messages.contains(&"Compiling"),
        "{messages:?}"
    );
    assert_eq!(events.last().unwrap().fraction, Some(1.0));

    // The step got the variables every game gets, the release as its folder, and its own temp.
    let env_file = fs::read_to_string(location.data.join("prepare-env.txt")).unwrap();
    assert!(
        env_file.contains("RECOMPLAUNCHER=1")
            && env_file.contains(&format!("DATA={}", location.data.display())),
        "{env_file}"
    );
    assert!(env_file.contains(".recomplauncher-tmp"), "{env_file}");
    assert!(
        !location.install.join(".recomplauncher-tmp").exists(),
        "the temp folder is removed"
    );
    let log = read_log(&s.env, GAME, "GameCube", "prepare", 50).unwrap();
    assert!(
        log.contains("progress 0.6 Compiling") && log.contains("-- finished"),
        "{log}"
    );

    // The dump is only needed once.
    drop(inputs_dir);
    let snap = snapshot(&s.env);
    assert_eq!(snap[0].instances, vec![view]);
    let (tx, rx) = mpsc::channel();
    start(&s.env, GAME, "GameCube", move |code| tx.send(code).unwrap()).unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_secs(20)).unwrap(), Some(0));
}

#[test]
fn bad_files_are_refused_before_anything_runs() {
    let s = setup();
    installed_and_published(&s);
    let dir = TempDir::new().unwrap();
    let ok = discs(dir.path(), "one", "two");

    let mut wrong_extension = ok.clone();
    wrong_extension.insert("disc2".into(), file(dir.path(), "d2.bin", "two"));
    let mut a_folder = ok.clone();
    a_folder.insert("disc2".into(), dir.path().to_path_buf());
    let mut missing = ok.clone();
    missing.insert("disc2".into(), dir.path().join("nope.iso"));
    let mut unknown = ok.clone();
    unknown.insert("disc9".into(), dir.path().join("d1.iso"));
    let mut only_one = ok.clone();
    only_one.remove("disc2");

    for (what, picked, expected) in [
        ("wrong extension", wrong_extension, "expected a .iso file"),
        ("a folder", a_folder, "not a folder"),
        ("missing file", missing, "doesn't exist"),
        ("unknown input", unknown, "doesn't ask for `disc9`"),
        (
            "a required file not given",
            only_one,
            "Missing file: Disc 2.",
        ),
    ] {
        let err = do_prepare(&s, picked).unwrap_err();
        assert!(err.contains(expected), "{what}: {err}");
    }
    assert!(s.env.library.instances(GAME).unwrap().is_empty());
    assert!(!instance_location(&s).data.join("built.bin").exists());
}

#[test]
fn a_failed_preparation_reports_why_and_leaves_the_instance_unprepared() {
    let s = setup();
    installed_and_published(&s);
    let dir = TempDir::new().unwrap();
    let err = do_prepare(&s, discs(dir.path(), "FAIL", "two")).unwrap_err();
    assert!(err.contains("exit code 7"), "{err}");
    let log = read_log(&s.env, GAME, "GameCube", "prepare", 50).unwrap();
    assert!(log.contains("disc is unreadable"), "{log}");
    assert!(s.env.library.instances(GAME).unwrap().is_empty());

    let err = start(&s.env, GAME, "GameCube", |_| {}).unwrap_err();
    assert!(err.contains("prepare it first"), "{err}");
}

#[test]
fn a_preparation_can_be_cancelled_and_stops_the_program() {
    let s = setup();
    installed_and_published(&s);
    let dir = TempDir::new().unwrap();
    let picked = discs(dir.path(), "HANG", "two");
    let cancel = AtomicBool::new(false);

    let started = Instant::now();
    let result = prepare(&s.env, GAME, "GameCube", picked, &cancel, &mut |p| {
        if p.message == "Working" {
            cancel.store(true, Ordering::Relaxed);
        }
    });

    assert_eq!(result.unwrap_err(), CANCELLED);
    assert!(started.elapsed() < Duration::from_secs(20));
    assert!(s.env.library.instances(GAME).unwrap().is_empty());
    assert!(!instance_location(&s)
        .install
        .join(".recomplauncher-tmp")
        .exists());
    assert!(read_log(&s.env, GAME, "GameCube", "prepare", 50)
        .unwrap()
        .contains("-- cancelled"));
}

#[test]
fn preparation_needs_free_space() {
    let mut s = setup();
    installed_and_published(&s);
    s.env.min_free_bytes = u64::MAX;
    let dir = TempDir::new().unwrap();
    assert!(do_prepare(&s, discs(dir.path(), "one", "two"))
        .unwrap_err()
        .contains("Not enough free space"));
}

#[test]
fn a_release_that_runs_as_it_is_only_records_the_players_files() {
    let s = setup();
    let archive = zip_bytes(&[
        ("recomplauncher.json", br#"{"inputs":[{"id":"game","label":"Your disc"}],"start":{"program":["fake-game"],"args":["--play","--data","{data}","--iso={input:game}"]}}"#.to_vec(), false),
        ("fake-game", fake_exe(), true),
    ]);
    s.fake.publish_ok("v1", &archive);
    do_install(&s).unwrap();
    let dir = TempDir::new().unwrap();
    // Started with the disc in its arguments: not ready until the player gave it.
    assert_eq!(snapshot(&s.env).len(), 1);
    assert!(start(&s.env, GAME, "PS1", |_| {}).is_err());
    let iso = file(dir.path(), "g.iso", "x");
    let view = prepare(
        &s.env,
        GAME,
        "PS1",
        BTreeMap::from([("game".into(), iso.clone())]),
        &NO_CANCEL,
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(view.state, InstanceState::Ready);
    // A deleted disc is asked for again.
    fs::remove_file(iso).unwrap();
    assert_eq!(
        snapshot(&s.env)[0].instances[0].state,
        InstanceState::NotPrepared
    );
}

// ---- play ----

#[test]
fn starts_the_game_with_its_settings_and_environment() {
    let s = setup();
    installed_and_published(&s);
    let dir = TempDir::new().unwrap();
    do_prepare(&s, discs(dir.path(), "one", "two")).unwrap();
    let location = instance_location(&s);
    crate::settings::apply(&location, "fs", SettingInput::Choice { index: 1 }).unwrap();

    let (tx, rx) = mpsc::channel();
    start(&s.env, GAME, "GameCube", move |code| tx.send(code).unwrap()).unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_secs(20)).unwrap(), Some(0));

    let played = fs::read_to_string(location.data.join("played.txt")).unwrap();
    assert!(
        played.contains(&format!(
            "args=--play --data {} --fullscreen",
            location.data.display()
        )),
        "{played}"
    );
    assert!(
        played.contains(&format!("data={}", location.data.display())),
        "{played}"
    );
    let log = read_log(&s.env, GAME, "GameCube", "game", 50).unwrap();
    assert!(
        log.contains("playing") && log.contains("-- exited with code 0"),
        "{log}"
    );
}

// ---- updates ----

#[test]
fn an_update_keeps_instances_and_rebuilds_from_what_was_kept() {
    let s = setup();
    installed_and_published(&s);
    let dir = TempDir::new().unwrap();
    do_prepare(&s, discs(dir.path(), "one", "two")).unwrap();
    let location = instance_location(&s);
    fs::write(location.data.join("save.dat"), "my save").unwrap();

    s.fake.publish_ok(
        "v2.0.0",
        &good_release(&manifest(r#", "extra": "a changed release""#)),
    );
    let view = do_install(&s).unwrap();
    assert_eq!(view.tag, "v2.0.0");
    assert_eq!(view.instances.len(), 1);
    assert_eq!(view.instances[0].state, InstanceState::NeedsRebuild);
    assert_eq!(
        fs::read_to_string(location.data.join("save.dat")).unwrap(),
        "my save",
        "instances survive updates"
    );
    assert!(start(&s.env, GAME, "GameCube", |_| {}).is_err());

    // The discs are gone; the rebuild starts from the extracted files the instance kept.
    fs::remove_dir_all(dir.path()).unwrap();
    let rebuilt = do_prepare(&s, BTreeMap::new()).unwrap();
    assert_eq!(rebuilt.state, InstanceState::Ready);
    assert_eq!(
        fs::read_to_string(location.data.join("built.bin")).unwrap(),
        "built from one+two"
    );
    let record = s
        .env
        .library
        .instance_record(GAME, &rebuilt.id, "GameCube")
        .unwrap();
    assert_eq!(record.prepared_release.as_deref(), Some("v2.0.0"));
}

#[test]
fn a_failed_update_download_keeps_the_installed_release() {
    let s = setup();
    installed_and_published(&s);
    s.fake
        .publish("v2.0.0", &good_release(&manifest("")), Some("1".repeat(64)));
    assert!(do_install(&s).is_err());
    assert_eq!(
        s.env.library.installation(GAME).unwrap().unwrap().tag,
        "v1.0.0"
    );
    assert!(s
        .root
        .join(GAME)
        .join("release")
        .join("recomplauncher.json")
        .is_file());
}

// ---- small pieces ----

#[test]
fn arguments_are_filled_in_or_left_out() {
    let m = Manifest::parse(&manifest("")).unwrap();
    let location = Instance::new(PathBuf::from("/g/release"), Some(PathBuf::from("/g/data")));
    let picked = BTreeMap::from([
        ("disc1".to_string(), PathBuf::from("/d/1.iso")),
        ("dlc".to_string(), PathBuf::from("/d/dlc.pkg")),
    ]);
    let ctx = Context {
        location: &location,
        manifest: &m,
        inputs: &picked,
        without_copy: false,
    };
    let expand = |t: &str, c: &Context| run::expand(t, c);
    assert_eq!(
        expand("--disc={input:disc1}", &ctx).unwrap(),
        Some("--disc=/d/1.iso".to_string())
    );
    assert_eq!(
        expand("{data}/x", &ctx).unwrap(),
        Some("/g/data/x".to_string())
    );
    assert!(
        expand("--disc={input:disc2}", &ctx)
            .unwrap_err()
            .contains("Disc 2"),
        "a required file is needed"
    );
    assert!(expand("{input:nope}", &ctx)
        .unwrap_err()
        .contains("unknown input"));

    let mut without = Context {
        without_copy: true,
        ..ctx
    };
    without.inputs = &picked;
    assert_eq!(
        expand("--disc={input:disc1}", &without).unwrap(),
        None,
        "a rebuild doesn't pass the copy"
    );
    assert_eq!(
        expand("--dlc={input:dlc}", &without).unwrap(),
        Some("--dlc=/d/dlc.pkg".to_string())
    );
    let none = BTreeMap::new();
    let optional = Context {
        location: &location,
        manifest: &m,
        inputs: &none,
        without_copy: false,
    };
    assert_eq!(
        expand("--dlc={input:dlc}", &optional).unwrap(),
        None,
        "an optional file nobody picked leaves its argument out"
    );
}

#[test]
fn ids_are_safe_and_stable() {
    assert_eq!(store::instance_id("GameCube"), "gamecube");
    assert_eq!(store::instance_id("PS1"), "ps1");
    assert_eq!(store::instance_id("Xbox Series X|S"), "xbox-series-x-s");
    assert_eq!(store::instance_id("../../etc"), "etc");
    assert_eq!(store::instance_id("日本"), "x");
    for ok in ["tos", "tales-of-eternia", "d2"] {
        assert!(store::valid_game_id(ok), "{ok}");
    }
    for bad in ["", "../x", "a/b", "A", "a--b", "-a", "a_b", "a.b"] {
        assert!(!store::valid_game_id(bad), "{bad}");
    }
    assert!(Library::new(PathBuf::from("/lib"))
        .game_dir("../x")
        .is_err());
}

#[test]
fn a_job_runs_once_and_can_be_cancelled() {
    let jobs = Jobs::default();
    let guard = jobs.begin("prepare:tos/gc").unwrap();
    assert!(jobs.begin("prepare:tos/gc").is_err());
    assert!(jobs.begin("prepare:tos/other").is_ok());
    assert!(!guard.cancel.load(Ordering::Relaxed));
    jobs.cancel("prepare:tos/gc");
    assert!(guard.cancel.load(Ordering::Relaxed));
    drop(guard);
    assert!(jobs.begin("prepare:tos/gc").is_ok());
}

#[test]
fn json_wire_format_matches_the_frontend_types() {
    let view = InstallationView {
        game_id: "tos".into(),
        repository: "To-Recomp/tos-recomp".into(),
        tag: "v1".into(),
        inputs: vec![InputView {
            id: "disc1".into(),
            label: "Disc 1".into(),
            role: "disc".into(),
            optional: false,
            kinds: vec!["file".into()],
            extensions: vec!["iso".into()],
        }],
        prepare_label: None,
        can_prepare: true,
        keeps_copy: true,
        instances: vec![InstanceView {
            id: "gc".into(),
            platform: "GC".into(),
            state: InstanceState::NeedsRebuild,
        }],
    };
    assert_eq!(
        serde_json::to_value(&view).unwrap(),
        serde_json::json!({
            "gameId": "tos", "repository": "To-Recomp/tos-recomp", "tag": "v1",
            "inputs": [{ "id": "disc1", "label": "Disc 1", "role": "disc", "optional": false, "kinds": ["file"], "extensions": ["iso"] }],
            "prepareLabel": null, "canPrepare": true, "keepsCopy": true,
            "instances": [{ "id": "gc", "platform": "GC", "state": "needsRebuild" }]
        })
    );
    assert_eq!(
        serde_json::to_value(ProgressEvent {
            job: "install:tos".into(),
            fraction: Some(0.5),
            message: "x".into(),
            stage: None
        })
        .unwrap(),
        serde_json::json!({ "job": "install:tos", "fraction": 0.5, "message": "x", "stage": null })
    );
    assert_eq!(
        serde_json::to_value(ProgressEvent {
            job: "prepare:tos/gc".into(),
            fraction: None,
            message: "".into(),
            stage: Some(run::Stage {
                id: "compile".into(),
                event: run::StageEvent::Progress { done: 12, total: 37, text: "Compiling".into() }
            })
        })
        .unwrap(),
        serde_json::json!({
            "job": "prepare:tos/gc", "fraction": null, "message": "",
            "stage": { "id": "compile", "event": "progress", "done": 12, "total": 37, "text": "Compiling" }
        })
    );
    assert_eq!(
        serde_json::to_value(InstanceState::NotPrepared).unwrap(),
        "notPrepared"
    );
}

#[test]
fn recent_releases_skip_drafts_and_a_missing_repository_has_none() {
    let fake = Fake::new();
    fake.route(
        &format!("/repos/{REPO}/releases?per_page=6"),
        200,
        serde_json::json!([
            { "tag_name": "v3", "draft": true },
            { "tag_name": "v2", "name": "Second", "published_at": "2026-11-20T10:00:00Z", "body": "Fixes", "prerelease": true },
            { "tag_name": "v1", "name": "", "body": null }
        ])
        .to_string()
        .into_bytes(),
    );
    let notes = release::recent_releases(&fake.url, REPO, 1).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].tag, "v2");
    assert_eq!(notes[0].name.as_deref(), Some("Second"));
    assert!(notes[0].prerelease);

    assert!(release::recent_releases(&fake.url, "To-Recomp/none", 3)
        .unwrap()
        .is_empty());
}

#[test]
fn a_repository_license_has_a_name_and_a_link_and_none_is_not_an_error() {
    let fake = Fake::new();
    fake.route(
        &format!("/repos/{REPO}/license"),
        200,
        serde_json::json!({
            "html_url": "https://github.com/To-Recomp/tos-recomp/blob/main/LICENSE",
            "license": { "name": "GNU General Public License v3.0" }
        })
        .to_string()
        .into_bytes(),
    );
    assert_eq!(
        release::repository_license(&fake.url, REPO).unwrap(),
        Some(release::RepositoryLicense {
            name: "GNU General Public License v3.0".into(),
            url: "https://github.com/To-Recomp/tos-recomp/blob/main/LICENSE".into(),
        })
    );
    assert_eq!(
        release::repository_license(&fake.url, "To-Recomp/none").unwrap(),
        None
    );
}

/// The limit is process-wide: the tests that set it take turns.
static LIMIT: Mutex<()> = Mutex::new(());

#[test]
fn a_download_keeps_to_the_shared_speed_limit_and_can_be_cancelled_while_waiting() {
    use sha2::{Digest, Sha256};

    let _turn = LIMIT.lock().unwrap_or_else(|e| e.into_inner());
    let body = vec![7u8; 400_000];
    let sha: String = Sha256::digest(&body)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let fake = Fake::new();
    fake.route("/file.zip", 200, body);
    let dir = tempfile::TempDir::new().unwrap();
    let url = format!("{}/file.zip", fake.url);

    crate::net::limit::shared().set(200_000);
    let started = std::time::Instant::now();
    release::download(
        &url,
        &dir.path().join("a.zip"),
        &sha,
        &AtomicBool::new(false),
        &mut |_, _| {},
    )
    .unwrap();
    let took = started.elapsed();

    // 400 kB at 200 kB/s, the first chunk and the last one's wait being free.
    assert!(took >= Duration::from_millis(1400), "took {took:?}");

    crate::net::limit::shared().set(10_000);
    let cancel = Arc::new(AtomicBool::new(false));
    let stopper = Arc::clone(&cancel);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        stopper.store(true, Ordering::Relaxed);
    });
    let started = std::time::Instant::now();
    let err = release::download(
        &url,
        &dir.path().join("b.zip"),
        &sha,
        &cancel,
        &mut |_, _| {},
    )
    .unwrap_err();
    crate::net::limit::shared().set(0);
    assert_eq!(err, CANCELLED);
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "a paused download still stops at once"
    );
}

/// The real thing: the published tos-recomp release prepared from the player's own discs.
/// Slow (minutes, needs a C++ compiler and SDL3) and needs game files, so it only runs on request:
///
///   TOS_RELEASE_ARCHIVE=/path/tos-recomp-X-linux-x86_64.tar.gz TOS_DISC1=... TOS_DISC2=... \
///     cargo test --lib real_tos_release -- --ignored --nocapture
#[test]
#[ignore = "needs a real release archive and both discs, see the doc comment"]
fn real_tos_release_prepares_from_real_discs() {
    let var = |n: &str| PathBuf::from(std::env::var(n).unwrap_or_else(|_| panic!("set {n}")));
    let archive = fs::read(var("TOS_RELEASE_ARCHIVE")).unwrap();
    let mut s = setup();
    s.env.min_free_bytes = 3 << 30;
    let name = format!("tos-recomp-test-{}-x86_64.tar.gz", std::env::consts::OS);
    let body = serde_json::json!({
        "tag_name": "v-test",
        "assets": [{ "name": name, "size": archive.len(),
            "browser_download_url": format!("{}/download/{name}", s.fake.url),
            "digest": format!("sha256:{}", sha(&archive)) }],
    });
    s.fake.route(
        &format!("/repos/{REPO}/releases/latest"),
        200,
        body.to_string().into_bytes(),
    );
    s.fake.route(&format!("/download/{name}"), 200, archive);
    do_install(&s).unwrap();

    let picked = BTreeMap::from([
        ("disc1".to_string(), var("TOS_DISC1")),
        ("disc2".to_string(), var("TOS_DISC2")),
    ]);
    let mut last = String::new();
    let view = prepare(&s.env, GAME, "GameCube", picked, &NO_CANCEL, &mut |p| {
        if p.message != last {
            println!("{:?} {}", p.fraction, p.message);
            last = p.message;
        }
    })
    .unwrap();
    assert_eq!(view.state, InstanceState::Ready);
}
