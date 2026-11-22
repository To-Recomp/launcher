//! The launcher's own settings, as opposed to a game's.
//!
//! They go through the same engine as a game's settings (`crate::settings`), with a manifest
//! built into the launcher instead of one read from a release. Values live in `launcher.json`
//! in the app's config folder. `args` settings don't apply here: nothing is launched with them.

pub mod folders;

use std::path::{Path, PathBuf};

use serde::Serialize;
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

pub(crate) fn config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
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

/// Where the games are kept: the player's choice, what this session uses, and the default.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folders {
    /// `None` while the default is chosen.
    pub configured: Option<String>,
    pub default: String,
    /// What this session uses; `None` when it couldn't be opened.
    pub in_use: Option<String>,
}

fn folders_of(app: &tauri::AppHandle) -> Result<Folders, String> {
    Ok(Folders {
        configured: folders::configured(&config_dir(app)?)?
            .map(|p| p.to_string_lossy().into_owned()),
        default: crate::library::default_root(app)?
            .to_string_lossy()
            .into_owned(),
        in_use: crate::library::root_in_use(app).map(|p| p.to_string_lossy().into_owned()),
    })
}

#[tauri::command(async)]
pub fn launcher_folders(app: tauri::AppHandle) -> Result<Folders, String> {
    folders_of(&app)
}

/// Chooses the games folder (`None`: the default); it applies on the next start.
#[tauri::command(async)]
pub fn set_games_folder(app: tauri::AppHandle, folder: Option<String>) -> Result<Folders, String> {
    folders::set(&config_dir(&app)?, folder.as_deref())?;
    folders_of(&app)
}

/// For the settings that apply on the next start (see `RestartBanner`).
#[tauri::command]
pub fn restart_launcher(app: tauri::AppHandle) {
    app.restart()
}

#[cfg(test)]
mod tests;
