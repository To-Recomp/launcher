//! "Update launcher": in-app updates through the Tauri updater.
//!
//! Only builds made by the release workflow have it (`updater` feature plus the signing public
//! key in the config). Even then it is offered only where the updater can replace the running
//! program: an AppImage, or a Windows installation made by the installer. Portable ZIPs, the
//! Linux tarball and the Fedora package are updated by downloading or by `dnf`.

use std::path::Path;

use serde::Serialize;

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

#[cfg(feature = "updater")]
const REPOSITORY: &str = "To-Recomp/launcher";
#[cfg(feature = "updater")]
const DEFAULT_API: &str = "https://api.github.com";
/// The releases listing is only read to find the newest pre-release's `latest.json`.
#[cfg(feature = "updater")]
const API_ENV: &str = "TO_RECOMP_GITHUB_API";

#[cfg(any(feature = "updater", test))]
#[derive(Debug, serde::Deserialize)]
struct ApiRelease {
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    assets: Vec<ApiAsset>,
}

#[cfg(any(feature = "updater", test))]
#[derive(Debug, serde::Deserialize)]
struct ApiAsset {
    name: String,
    browser_download_url: String,
}

/// The `latest.json` of the newest release that has one, pre-releases included. GitHub's own
/// `releases/latest` link never points at a pre-release, which is why it can't be used for
/// that channel.
#[cfg(any(feature = "updater", test))]
fn newest_latest_json(releases: &[ApiRelease]) -> Option<&str> {
    releases
        .iter()
        .filter(|r| !r.draft)
        .find_map(|r| r.assets.iter().find(|a| a.name == "latest.json"))
        .map(|a| a.browser_download_url.as_str())
}

/// What the player is told about a newer release.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
}

#[cfg(feature = "updater")]
async fn updater_for(app: &tauri::AppHandle) -> Result<tauri_plugin_updater::Updater, String> {
    use tauri_plugin_updater::UpdaterExt;

    let preferences = crate::launcher_settings::config_dir(app)
        .and_then(|dir| crate::launcher_settings::update_preferences(&dir))?;
    let mut builder = app.updater_builder();
    if preferences.prerelease {
        let api = std::env::var(API_ENV).unwrap_or_else(|_| DEFAULT_API.to_string());
        let found =
            tauri::async_runtime::spawn_blocking(move || -> Result<Option<String>, String> {
                let agent: ureq::Agent = crate::net::agent_builder()?.build().into();
                let releases: Vec<ApiRelease> = agent
                    .get(&format!("{api}/repos/{REPOSITORY}/releases?per_page=10"))
                    .header("Accept", "application/vnd.github+json")
                    .header(
                        "User-Agent",
                        concat!("to-recomp-launcher/", env!("CARGO_PKG_VERSION")),
                    )
                    .call()
                    .map_err(|e| format!("Can't look for pre-releases: {e}"))?
                    .body_mut()
                    .read_json()
                    .map_err(|e| format!("GitHub's answer is unreadable: {e}"))?;
                Ok(newest_latest_json(&releases).map(str::to_string))
            })
            .await
            .map_err(|e| e.to_string())??;
        // No release carries one yet: the stable endpoint is the only place left to look.
        if let Some(url) = found {
            let url = url
                .parse()
                .map_err(|e| format!("Not a usable update address: {e}"))?;
            builder = builder
                .endpoints(vec![url])
                .map_err(|e| format!("Not a usable update address: {e}"))?;
        }
    }
    builder = match crate::net::updater_proxy() {
        crate::net::UpdaterProxy::System => builder,
        crate::net::UpdaterProxy::Direct => builder.no_proxy(),
        crate::net::UpdaterProxy::Proxy { url, no_proxy } => {
            builder.configure_client(move |client| {
                match reqwest::Proxy::all(&url) {
                    Ok(proxy) => {
                        client.proxy(proxy.no_proxy(reqwest::NoProxy::from_string(&no_proxy)))
                    }
                    // Never fall back to a direct connection when the proxy can't be used.
                    Err(_) => {
                        client.proxy(reqwest::Proxy::all("http://127.0.0.1:9").expect("valid"))
                    }
                }
            })
        }
    };
    builder
        .build()
        .map_err(|e| format!("Update check failed: {e}"))
}

/// Looks for a newer release, on the chosen channel, without installing it.
#[cfg(feature = "updater")]
#[tauri::command]
pub async fn check_launcher_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    if !supported_here() {
        return Err("This installation is updated outside the launcher.".to_string());
    }
    let update = updater_for(&app)
        .await?
        .check()
        .await
        .map_err(|e| format!("Update check failed: {e}"))?;
    Ok(update.map(|u| UpdateInfo { version: u.version }))
}

/// Downloads and installs a newer release, then restarts into it. `Ok` means there is nothing
/// newer.
#[cfg(feature = "updater")]
#[tauri::command]
pub async fn update_launcher(app: tauri::AppHandle) -> Result<(), String> {
    if !supported_here() {
        return Err("This installation is updated outside the launcher.".to_string());
    }
    let Some(update) = updater_for(&app)
        .await?
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
pub async fn check_launcher_update() -> Result<Option<UpdateInfo>, String> {
    Err("This build has no self-update.".to_string())
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

    fn release(draft: bool, assets: &[&str]) -> ApiRelease {
        ApiRelease {
            draft,
            assets: assets
                .iter()
                .map(|name| ApiAsset {
                    name: name.to_string(),
                    browser_download_url: format!("https://example.test/{name}"),
                })
                .collect(),
        }
    }

    #[test]
    fn the_prerelease_channel_follows_the_newest_release_that_has_an_update_file() {
        let releases = [
            release(true, &["latest.json"]), // a draft is not published
            release(false, &["notes.txt"]),  // unsigned: no updater entry
            release(false, &["latest.json"]),
            release(false, &["latest.json", "x"]), // older
        ];
        assert_eq!(
            newest_latest_json(&releases),
            Some("https://example.test/latest.json")
        );
        assert_eq!(newest_latest_json(&releases[..2]), None);
    }
}
