//! The diagnostic file: a local, scrubbed Markdown file the player reads and pastes into a public
//! issue. Nothing here touches the network, and nothing is written unless the player asks.
//!
//! ```text
//! Diagnostic format: 1
//! | table: launcher, OS, GPU, game, context |
//! ## Error            the message, scrubbed
//! <details> log tail  ≤ 200 lines, scrubbed
//! ```

pub mod panic;
pub mod scrub;
pub mod system;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::library;
use scrub::{limit_tail, scrub, Scrub, MAX_BYTES};
use system::System;

pub const FORMAT: u32 = 1;
/// Diagnostic files kept in the folder, the panic ones included.
pub const KEEP_FILES: usize = 10;
const ERROR_MAX_BYTES: usize = 8 * 1024;
/// Room kept for the fences and the headings around the log.
const LOG_FRAME_BYTES: usize = 512;
const FILE_MARK: &str = "-diagnostic-";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Install,
    Prepare,
    Game,
    Manual,
    Launcher,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Install => "install",
            Kind::Prepare => "prepare",
            Kind::Game => "game",
            Kind::Manual => "manual",
            Kind::Launcher => "launcher",
        }
    }

    /// Only `prepare.log` and `game.log` exist: an install failure has no log.
    pub fn log_file(self) -> Option<&'static str> {
        match self {
            Kind::Prepare => Some("prepare.log"),
            Kind::Game => Some("game.log"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Game {
    pub id: String,
    pub release: Option<String>,
    pub platform: String,
    pub region: String,
    pub state: String,
}

/// What the launcher knows about the game a report is about.
#[derive(Debug, Clone, Default)]
pub struct Subject {
    pub game: Game,
    /// `{library}`, `{data}`, `{install}`, `{input:<id>}`.
    pub paths: Vec<(String, String)>,
    pub log: Option<PathBuf>,
}

pub struct Facts<'a> {
    pub kind: Kind,
    pub game: Option<&'a Game>,
    pub error: &'a str,
    /// The log's file name and its raw tail.
    pub log: Option<(&'a str, &'a str)>,
    pub system: &'a System,
    pub version: &'a str,
    pub build: &'a str,
}

/// A table cell: one line, no pipe, scrubbed.
fn cell(text: &str, known: &Scrub) -> String {
    scrub(text, known)
        .replace(['\n', '\t'], " ")
        .replace('|', "/")
        .trim()
        .to_string()
}

/// A code fence that no line of `body` can close.
fn fenced(body: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in body.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat((longest + 1).max(3));
    format!("{fence}text\n{body}\n{fence}")
}

/// The Markdown file, always English, at most [`MAX_BYTES`] bytes.
pub fn render(facts: &Facts, known: &Scrub) -> String {
    let mut rows: Vec<(&str, String)> = vec![
        ("Launcher", format!("{} ({})", facts.version, facts.build)),
        ("OS family", facts.system.family.clone()),
    ];
    if let Some(name) = &facts.system.name {
        rows.push(("OS", name.clone()));
    }
    rows.push(("Architecture", facts.system.arch.clone()));
    for gpu in &facts.system.gpus {
        rows.push(("GPU", gpu.clone()));
    }
    if let Some(game) = facts.game {
        rows.push(("Game", game.id.clone()));
        if let Some(release) = &game.release {
            rows.push(("Release", release.clone()));
        }
        rows.push(("Platform", game.platform.clone()));
        rows.push(("Region", game.region.clone()));
        rows.push(("State", game.state.clone()));
    }
    rows.push(("Context", facts.kind.name().to_string()));

    let mut out = format!("Diagnostic format: {FORMAT}\n\n| Field | Value |\n|---|---|\n");
    for (field, value) in rows {
        out.push_str(&format!("| {field} | {} |\n", cell(&value, known)));
    }

    let error = limit_tail(&scrub(facts.error, known), 200, ERROR_MAX_BYTES);
    out.push_str(&format!("\n## Error\n\n{}\n", fenced(&error)));

    if let Some((name, raw)) = facts.log {
        let tail = scrub(raw, known);
        let budget = MAX_BYTES.saturating_sub(out.len() + LOG_FRAME_BYTES + name.len());
        let tail = limit_tail(&tail, scrub::MAX_LINES, budget);
        if !tail.trim().is_empty() {
            let lines = tail.lines().count();
            out.push_str(&format!(
                "\n<details><summary>{name}, last {lines} lines</summary>\n\n{}\n\n</details>\n",
                fenced(&tail)
            ));
        }
    }
    out
}

/// `YYYY-MM-DD-HHMMSS`, UTC.
fn stamp(now: SystemTime) -> String {
    let secs = now.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()) as i64;
    let (days, rest) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}-{:02}{:02}{:02}",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// Writes the file into `dir` as `<base>-diagnostic-<stamp>.md` and drops the oldest ones beyond
/// [`KEEP_FILES`].
pub fn write_file(dir: &Path, base: &str, text: &str, now: SystemTime) -> Result<PathBuf, String> {
    let fail =
        |e: std::io::Error| format!("Can't write the diagnostic file in {}: {e}", dir.display());
    fs::create_dir_all(dir).map_err(fail)?;
    let base: String = base
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect();
    let base = if base.is_empty() {
        "launcher".to_string()
    } else {
        base
    };
    let stamp = stamp(now);
    let mut n = 1;
    let path = loop {
        let suffix = if n == 1 {
            String::new()
        } else {
            format!("-{n}")
        };
        let path = dir.join(format!("{base}{FILE_MARK}{stamp}{suffix}.md"));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                std::io::Write::write_all(&mut file, text.as_bytes()).map_err(fail)?;
                break path;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => n += 1,
            Err(e) => return Err(fail(e)),
        }
    };
    rotate(dir, KEEP_FILES);
    Ok(path)
}

/// Deletes the oldest diagnostic files so that `keep` remain.
pub fn rotate(dir: &Path, keep: usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut files: Vec<(SystemTime, String, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !(name.contains(FILE_MARK) && name.ends_with(".md")) {
                return None;
            }
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((modified, name, entry.path()))
        })
        .collect();
    files.sort_by(|a, b| (b.0, &b.1).cmp(&(a.0, &a.1)));
    for (_, _, path) in files.into_iter().skip(keep) {
        let _ = fs::remove_file(path);
    }
}

fn env_names(keys: &[&str]) -> Vec<String> {
    keys.iter()
        .filter_map(|key| std::env::var(key).ok())
        .filter(|v| !v.trim().is_empty())
        .collect()
}

/// This machine's home, user names and host names.
pub fn local_scrub() -> Scrub {
    let mut known = Scrub::default();
    let home = dirs::home_dir();
    known.home.extend(env_names(&["HOME", "USERPROFILE"]));
    if let Some(home) = &home {
        known.home.push(home.to_string_lossy().into_owned());
        if let Some(name) = home.file_name() {
            known.users.push(name.to_string_lossy().into_owned());
        }
    }
    known
        .users
        .extend(env_names(&["USER", "USERNAME", "LOGNAME"]));
    known.hosts.extend(env_names(&["HOSTNAME", "COMPUTERNAME"]));
    if let Ok(host) = fs::read_to_string("/etc/hostname") {
        known.hosts.push(host.trim().to_string());
    }
    known.home.sort();
    known.home.dedup();
    known.users.sort();
    known.users.dedup();
    known.hosts.sort();
    known.hosts.dedup();
    known
}

/// This machine's names plus the launcher's own folders, which become `{app}`.
pub fn app_scrub<R: tauri::Runtime>(app: &impl Manager<R>) -> Scrub {
    let mut known = local_scrub();
    let resolver = app.path();
    for dir in [
        resolver.app_data_dir(),
        resolver.app_config_dir(),
        resolver.app_cache_dir(),
        resolver.app_log_dir(),
    ]
    .into_iter()
    .flatten()
    {
        known
            .paths
            .push((dir.to_string_lossy().into_owned(), "{app}".to_string()));
    }
    known
}

pub(crate) fn build_channel() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else if std::env::var_os("APPIMAGE").is_some() {
        "appimage"
    } else {
        "release"
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub context: Kind,
    pub game_id: Option<String>,
    pub platform: Option<String>,
    pub region: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Created {
    pub name: String,
    pub path: String,
    pub folder: String,
    /// Exactly the file's content: what *Copy* puts on the clipboard.
    pub text: String,
}

/// Builds and writes the diagnostic file for the player to review.
#[tauri::command(async)]
pub fn build_report(app: tauri::AppHandle, request: Request) -> Result<Created, String> {
    let resolver = app.path();
    let dir = resolver
        .app_log_dir()
        .map_err(|e| format!("No place to keep the diagnostic file: {e}"))?;

    let mut known = app_scrub(&app);
    let subject = match (&request.game_id, &request.platform, &request.region) {
        (Some(id), Some(platform), Some(region)) => {
            let env = library::app_env(&app)?;
            Some(library::diagnostic_subject(
                &env,
                id,
                platform,
                region,
                request.context.log_file(),
            )?)
        }
        _ => None,
    };
    if let Some(subject) = &subject {
        known.paths.extend(subject.paths.iter().cloned());
    }
    let log = subject
        .as_ref()
        .and_then(|s| s.log.as_ref())
        .map(|path| library::tail_of(path, scrub::MAX_LINES));
    let name = request.context.log_file();
    let system = System::collect();
    let text = render(
        &Facts {
            kind: request.context,
            game: subject.as_ref().map(|s| &s.game),
            error: request.error.as_deref().unwrap_or(""),
            log: name.zip(log.as_deref()),
            system: &system,
            version: env!("CARGO_PKG_VERSION"),
            build: build_channel(),
        },
        &known,
    );
    let base = subject.as_ref().map_or("launcher", |s| s.game.id.as_str());
    let path = write_file(&dir, base, &text, SystemTime::now())?;
    Ok(Created {
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        path: path.to_string_lossy().into_owned(),
        folder: dir.to_string_lossy().into_owned(),
        text,
    })
}

#[cfg(test)]
mod tests;
