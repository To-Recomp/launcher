//! The launcher's own settings, as opposed to a game's.
//!
//! They go through the same engine as a game's settings (`crate::settings`), with a manifest
//! built into the launcher instead of one read from a release. Values live in `launcher.json`
//! in the app's config folder. `args` settings don't apply here: nothing is launched with them.

mod config;
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

/// How the interface looks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Appearance {
    /// `dark`, `light` or `system`.
    pub theme: String,
    /// Tint the pages with colors taken from the game's logo.
    pub game_palette: bool,
}

pub fn appearance(config_dir: &Path) -> Result<Appearance, String> {
    let config = config::read(config_dir)?;
    let theme = match config.pointer("/appearance/theme").and_then(|v| v.as_str()) {
        Some(theme @ ("light" | "system")) => theme,
        _ => "dark",
    };
    Ok(Appearance {
        theme: theme.to_string(),
        game_palette: config
            .pointer("/appearance/gamePalette")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    })
}

#[tauri::command(async)]
pub fn launcher_appearance(app: tauri::AppHandle) -> Result<Appearance, String> {
    appearance(&config_dir(&app)?)
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

#[tauri::command(async)]
pub fn load_launcher_settings(app: tauri::AppHandle) -> Result<SettingsView, String> {
    let mut view = settings::build_view(&instance(&config_dir(&app)?), &manifest()?)?;
    if !crate::updater::updater_available() {
        // Another way of updating (a package, a ZIP): nothing to choose here.
        view.settings
            .retain(|s| s.group.as_deref() != Some(UPDATES_GROUP));
    }
    Ok(view)
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
