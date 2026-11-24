//! The launcher's own settings, as opposed to a game's.
//!
//! They go through the same engine as a game's settings (`crate::settings`), with a manifest
//! built into the launcher instead of one read from a release. Values live in `launcher.json`
//! in the app's config folder. `args` settings don't apply here: nothing is launched with them.

pub(crate) mod config;
pub mod folders;

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::Manager;

use crate::settings::manifest::Manifest;
use crate::settings::{self, Instance, SettingInput, SettingsView};

/// Whether the launcher checks for updates when it starts, and which releases it looks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePreferences {
    pub auto: bool,
    pub prerelease: bool,
}

pub fn update_preferences(config_dir: &Path) -> Result<UpdatePreferences, String> {
    let config = config::read(config_dir)?;
    Ok(UpdatePreferences {
        auto: config
            .pointer("/updates/auto")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        prerelease: config.pointer("/updates/channel").and_then(|v| v.as_str())
            == Some("prerelease"),
    })
}

/// What the launcher does around its window and the games it starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Behavior {
    /// Closing the window hides it in the notification area instead of quitting.
    pub tray: bool,
    /// The launcher quits once a game has started.
    pub close_on_launch: bool,
}

pub fn behavior(config_dir: &Path) -> Result<Behavior, String> {
    let config = config::read(config_dir)?;
    let flag = |pointer: &str| {
        config
            .pointer(pointer)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };
    Ok(Behavior {
        tray: flag("/behavior/tray"),
        close_on_launch: flag("/behavior/closeOnLaunch"),
    })
}

/// The language the player chose for the interface, `None` to follow the system's.
pub fn language(config_dir: &Path) -> Result<Option<String>, String> {
    Ok(config::read(config_dir)?
        .pointer("/language")
        .and_then(|v| v.as_str())
        .filter(|code| *code != "auto")
        .map(str::to_string))
}

#[tauri::command(async)]
pub fn launcher_language(app: tauri::AppHandle) -> Result<Option<String>, String> {
    language(&config_dir(&app)?)
}

#[tauri::command(async)]
pub fn launcher_update_preferences(app: tauri::AppHandle) -> Result<UpdatePreferences, String> {
    update_preferences(&config_dir(&app)?)
}

/// The heading of the update settings in the built-in manifest.
const UPDATES_GROUP: &str = "Updates";
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

/// What the page shows: the update settings only where the launcher can replace itself. Both
/// commands go through this, so a change never brings back what loading left out.
fn shown(mut view: SettingsView, updater_available: bool) -> SettingsView {
    if !updater_available {
        // Another way of updating (a package, a ZIP): nothing to choose here.
        view.settings
            .retain(|s| s.group.as_deref() != Some(UPDATES_GROUP));
    }
    view
}

fn load_view(dir: &Path, updater_available: bool) -> Result<SettingsView, String> {
    Ok(shown(
        settings::build_view(&instance(dir), &manifest()?)?,
        updater_available,
    ))
}

fn change_view(
    dir: &Path,
    id: &str,
    input: SettingInput,
    updater_available: bool,
) -> Result<SettingsView, String> {
    Ok(shown(
        settings::apply_with(&instance(dir), &manifest()?, id, input)?,
        updater_available,
    ))
}

#[tauri::command(async)]
pub fn load_launcher_settings(app: tauri::AppHandle) -> Result<SettingsView, String> {
    load_view(&config_dir(&app)?, crate::updater::updater_available())
}

#[tauri::command(async)]
pub fn set_launcher_setting(
    app: tauri::AppHandle,
    id: String,
    input: SettingInput,
) -> Result<SettingsView, String> {
    change_view(
        &config_dir(&app)?,
        &id,
        input,
        crate::updater::updater_available(),
    )
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
