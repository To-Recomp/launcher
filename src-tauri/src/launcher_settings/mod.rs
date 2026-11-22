//! The launcher's own settings, as opposed to a game's.
//!
//! They go through the same engine as a game's settings (`crate::settings`), with a manifest
//! built into the launcher instead of one read from a release. Values live in `launcher.json`
//! in the app's config folder. `args` settings don't apply here: nothing is launched with them.

use std::path::{Path, PathBuf};

use tauri::Manager;

use crate::settings::manifest::Manifest;
use crate::settings::{self, Instance, SettingInput, SettingsView};

const MANIFEST: &str = include_str!("manifest.json");

fn manifest() -> Result<Manifest, String> {
    Manifest::parse(MANIFEST)
}

/// The launcher has no install folder of its own to configure: its config folder is both.
fn instance(dir: &Path) -> Instance {
    Instance::new(dir.to_path_buf(), Some(dir.to_path_buf()))
}

fn config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|e| format!("Can't find the launcher's settings folder: {e}"))
}

#[tauri::command(async)]
pub fn load_launcher_settings(app: tauri::AppHandle) -> Result<SettingsView, String> {
    settings::build_view(&instance(&config_dir(&app)?), &manifest()?)
}

#[tauri::command(async)]
pub fn set_launcher_setting(
    app: tauri::AppHandle,
    id: String,
    input: SettingInput,
) -> Result<SettingsView, String> {
    settings::apply_with(&instance(&config_dir(&app)?), &manifest()?, &id, input)
}

#[cfg(test)]
mod tests;
