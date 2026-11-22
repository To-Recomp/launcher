//! A launcher panic writes its diagnostic file at once; the next start offers it in a banner.
//!
//! The hook only writes a local file (no network, no backtrace, no crash dump) and only for the
//! first panic of a run, so a panic loop can't push the older files out of the rotation.

use std::fs;
use std::panic::PanicHookInfo;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::SystemTime;

use super::scrub::Scrub;
use super::system::System;
use super::{build_channel, render, write_file, Created, Facts, Kind};

/// Names the file the banner should offer; removed when the player dismisses it.
const MARKER: &str = "panic-pending";

struct Armed {
    dir: PathBuf,
    known: Scrub,
}

static ARMED: OnceLock<Armed> = OnceLock::new();
static WRITTEN: AtomicBool = AtomicBool::new(false);

/// Where the hook writes and what it must hide. Until this is called a panic writes nothing.
pub fn arm(dir: PathBuf, known: Scrub) {
    let _ = ARMED.set(Armed { dir, known });
}

/// Installs the hook, keeping the previous one (the default message on stderr).
pub fn install() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        previous(info);
        if let Some(armed) = ARMED.get() {
            if !WRITTEN.swap(true, Ordering::SeqCst) {
                let _ = record(&armed.dir, &armed.known, &message(info), SystemTime::now());
            }
        }
    }));
}

fn message(info: &PanicHookInfo) -> String {
    let payload = info.payload();
    let text = payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "(no message)".to_string());
    match info.location() {
        Some(at) => format!(
            "The launcher panicked: {text}\nat {}:{}:{}",
            at.file(),
            at.line(),
            at.column()
        ),
        None => format!("The launcher panicked: {text}"),
    }
}

/// Writes the (scrubbed) file and the marker the next start looks for.
pub fn record(dir: &Path, known: &Scrub, error: &str, now: SystemTime) -> Result<PathBuf, String> {
    let system = System::collect();
    let text = render(
        &Facts {
            kind: Kind::Launcher,
            game: None,
            error,
            log: None,
            system: &system,
            version: env!("CARGO_PKG_VERSION"),
            build: build_channel(),
        },
        known,
    );
    let path = write_file(dir, "launcher", &text, now)?;
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    crate::fs_util::write_atomically(&dir.join(MARKER), &name)?;
    Ok(path)
}

/// The file left by a panic of the previous run, if it is still there.
pub fn pending(dir: &Path) -> Option<Created> {
    let marker = dir.join(MARKER);
    let name = fs::read_to_string(&marker).ok()?;
    let name = name.trim();
    // The marker only ever holds a name we wrote; refuse anything else.
    let path = dir.join(Path::new(name).file_name()?);
    match fs::read_to_string(&path) {
        Ok(text) => Some(Created {
            name: name.to_string(),
            path: path.to_string_lossy().into_owned(),
            folder: dir.to_string_lossy().into_owned(),
            text,
        }),
        Err(_) => {
            // Rotated away or deleted by hand: nothing to offer.
            let _ = fs::remove_file(marker);
            None
        }
    }
}

/// Forgets the pending panic; the file itself stays until rotation removes it.
pub fn dismiss(dir: &Path) {
    let _ = fs::remove_file(dir.join(MARKER));
}

#[tauri::command(async)]
pub fn pending_panic(app: tauri::AppHandle) -> Option<Created> {
    use tauri::Manager;
    pending(&app.path().app_log_dir().ok()?)
}

#[tauri::command(async)]
pub fn dismiss_panic(app: tauri::AppHandle) {
    use tauri::Manager;
    if let Ok(dir) = app.path().app_log_dir() {
        dismiss(&dir);
    }
}
