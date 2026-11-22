//! "Update launcher": in-app updates through the Tauri updater.
//!
//! Only builds made by the release workflow have it (`updater` feature plus the signing public
//! key in the config). Even then it is offered only where the updater can replace the running
//! program: an AppImage, or a Windows installation made by the installer. Portable ZIPs, the
//! Linux tarball and the Fedora package are updated by downloading or by `dnf`.

use std::path::Path;

pub fn self_update_supported(os: &str, running_appimage: bool, has_uninstaller: bool) -> bool {
    match os {
        "linux" => running_appimage,
        "windows" => has_uninstaller,
        _ => false,
    }
}

fn supported_here() -> bool {
    let has_uninstaller = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("uninstall.exe")))
        .is_some_and(|path| Path::new(&path).exists());
    self_update_supported(
        std::env::consts::OS,
        std::env::var_os("APPIMAGE").is_some(),
        has_uninstaller,
    )
}

/// Whether the "Update launcher" button should be offered.
#[tauri::command]
pub fn updater_available() -> bool {
    cfg!(feature = "updater") && supported_here()
}

/// Downloads and installs a newer release, then restarts into it. `Ok` means there is nothing
/// newer.
#[cfg(feature = "updater")]
#[tauri::command]
pub async fn update_launcher(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;

    if !supported_here() {
        return Err("This installation is updated outside the launcher.".to_string());
    }
    let updater = app
        .updater()
        .map_err(|e| format!("Update check failed: {e}"))?;
    let Some(update) = updater
        .check()
        .await
        .map_err(|e| format!("Update check failed: {e}"))?
    else {
        return Ok(());
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| format!("Update failed: {e}"))?;
    app.restart()
}

#[cfg(not(feature = "updater"))]
#[tauri::command]
pub async fn update_launcher() -> Result<(), String> {
    Err("This build has no self-update.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_installs_the_updater_can_replace_are_offered_updates() {
        assert!(self_update_supported("linux", true, false));
        assert!(!self_update_supported("linux", false, false)); // tarball or Fedora package
        assert!(self_update_supported("windows", false, true)); // installed with the installer
        assert!(!self_update_supported("windows", false, false)); // portable ZIP
        assert!(!self_update_supported("macos", true, true));
    }
}
