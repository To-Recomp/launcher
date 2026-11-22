//! A game's release on GitHub: find the asset for this OS, download it, check the checksum the
//! API publishes, and unpack it.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Deserialize;
use sha2::{Digest, Sha256};

pub const CANCELLED: &str = "Cancelled.";

#[derive(Debug, Deserialize)]
struct ApiRelease {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    assets: Vec<ApiAsset>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ApiAsset {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    pub browser_download_url: String,
    /// `sha256:<hex>`, published by GitHub for every uploaded asset.
    pub digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Release {
    pub tag: String,
    pub asset: ApiAsset,
    pub sha256: String,
}

fn agent() -> Result<ureq::Agent, String> {
    Ok(crate::net::agent_builder()?
        .timeout_connect(Some(Duration::from_secs(10)))
        .timeout_recv_response(Some(Duration::from_secs(30)))
        .http_status_as_error(false)
        .build()
        .into())
}

fn get_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<Option<T>, String> {
    let mut response = agent()?
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header(
            "User-Agent",
            concat!("to-recomp-launcher/", env!("CARGO_PKG_VERSION")),
        )
        .call()
        .map_err(|e| format!("Can't reach GitHub: {e}"))?;
    match response.status().as_u16() {
        200 => response
            .body_mut()
            .read_json()
            .map(Some)
            .map_err(|e| format!("GitHub's answer is unreadable: {e}")),
        404 => Ok(None),
        403 | 429 => {
            Err("GitHub is limiting requests from this connection; try again later.".to_string())
        }
        status => Err(format!("GitHub answered {status}.")),
    }
}

/// The newest release of `repository` (`owner/name`) with an asset for `os`. A repository that
/// only has pre-releases has no "latest" one: its newest release is taken then.
pub fn latest_release(api: &str, repository: &str, os: &str) -> Result<Release, String> {
    let latest: Option<ApiRelease> =
        get_json(&format!("{api}/repos/{repository}/releases/latest"))?;
    let release = match latest {
        Some(release) => release,
        None => {
            let all: Vec<ApiRelease> =
                get_json(&format!("{api}/repos/{repository}/releases?per_page=10"))?
                    .ok_or_else(|| format!("{repository} doesn't exist or has no releases."))?;
            all.into_iter()
                .find(|r| !r.draft)
                .ok_or_else(|| format!("{repository} has no release yet."))?
        }
    };
    let asset = pick_asset(&release.assets, os)?.clone();
    let digest = asset
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
        .filter(|d| d.len() == 64 && d.chars().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| {
            format!(
                "GitHub publishes no SHA-256 checksum for {}, so it can't be verified.",
                asset.name
            )
        })?
        .to_ascii_lowercase();
    Ok(Release {
        tag: release.tag_name,
        asset,
        sha256: digest,
    })
}

/// What the home page shows of one published release.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseNote {
    pub tag: String,
    pub name: Option<String>,
    pub published_at: Option<String>,
    pub body: String,
    pub prerelease: bool,
}

#[derive(Debug, Deserialize)]
struct ApiNote {
    tag_name: String,
    name: Option<String>,
    published_at: Option<String>,
    body: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// The newest published releases of `repository` (`owner/name`), newest first. A repository
/// that doesn't exist or has no release gives an empty list.
pub fn recent_releases(
    api: &str,
    repository: &str,
    count: usize,
) -> Result<Vec<ReleaseNote>, String> {
    let all: Vec<ApiNote> = get_json(&format!(
        "{api}/repos/{repository}/releases?per_page={}",
        count + 5
    ))?
    .unwrap_or_default();
    Ok(all
        .into_iter()
        .filter(|r| !r.draft)
        .take(count)
        .map(|r| ReleaseNote {
            tag: r.tag_name,
            name: r.name.filter(|n| !n.trim().is_empty()),
            published_at: r.published_at,
            body: r.body.unwrap_or_default(),
            prerelease: r.prerelease,
        })
        .collect())
}

/// The license GitHub detected in a repository, for the game's About page.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLicense {
    pub name: String,
    /// The license file on GitHub.
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct ApiLicenseFile {
    html_url: Option<String>,
    license: Option<ApiLicense>,
}

#[derive(Debug, Deserialize)]
struct ApiLicense {
    name: String,
}

/// The license of `repository` (`owner/name`); `None` when it has none or doesn't exist.
pub fn repository_license(
    api: &str,
    repository: &str,
) -> Result<Option<RepositoryLicense>, String> {
    let file: Option<ApiLicenseFile> = get_json(&format!("{api}/repos/{repository}/license"))?;
    Ok(file.and_then(|f| {
        Some(RepositoryLicense {
            name: f.license?.name,
            url: f.html_url?,
        })
    }))
}

fn is_archive(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".zip") || lower.ends_with(".tar.gz") || lower.ends_with(".tgz")
}

/// The archive made for this OS (x86-64 preferred; ARM builds are never taken).
pub fn pick_asset<'a>(assets: &'a [ApiAsset], os: &str) -> Result<&'a ApiAsset, String> {
    let tokens: &[&str] = match os {
        "linux" => &["linux"],
        "windows" => &["windows", "win64", "win32"],
        "macos" => &["macos", "darwin", "mac"],
        _ => &[],
    };
    let mut candidates: Vec<&ApiAsset> = assets
        .iter()
        .filter(|a| {
            let name = a.name.to_ascii_lowercase();
            is_archive(&name)
                && tokens.iter().any(|t| name.contains(t))
                && !name.contains("arm64")
                && !name.contains("aarch64")
        })
        .collect();
    candidates.sort_by_key(|a| {
        let name = a.name.to_ascii_lowercase();
        let preferred = ["x86_64", "x64", "amd64"].iter().any(|t| name.contains(t));
        (!preferred, a.name.clone())
    });
    candidates
        .first()
        .copied()
        .ok_or_else(|| format!("This release has no download made for {os}."))
}

/// Downloads `url` to `dest`, checking the SHA-256 of what arrived; `progress` gets
/// (bytes so far, total if known).
pub fn download(
    url: &str,
    dest: &Path,
    sha256: &str,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, Option<u64>),
) -> Result<(), String> {
    let mut response = agent()?
        .get(url)
        .header(
            "User-Agent",
            concat!("to-recomp-launcher/", env!("CARGO_PKG_VERSION")),
        )
        .call()
        .map_err(|e| format!("Can't download the release: {e}"))?;
    if response.status().as_u16() != 200 {
        return Err(format!(
            "The download answered {}.",
            response.status().as_u16()
        ));
    }
    let total = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok());

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Can't write {}: {e}", parent.display()))?;
    }
    let mut file =
        File::create(dest).map_err(|e| format!("Can't write {}: {e}", dest.display()))?;
    let mut reader = response.body_mut().with_config().limit(u64::MAX).reader();
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    let mut done = 0u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.to_string());
        }
        let n = reader
            .read(&mut buffer)
            .map_err(|e| format!("The download was interrupted: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        file.write_all(&buffer[..n])
            .map_err(|e| format!("Can't write {}: {e}", dest.display()))?;
        done += n as u64;
        progress(done, total);
        crate::net::limit::shared().pace(n, cancel); // the budget shared by every game download
    }
    let actual = hex(&hasher.finalize());
    if actual != sha256 {
        return Err(format!(
            "The downloaded file doesn't match the checksum GitHub publishes (expected {sha256}, got {actual}). Nothing was installed."
        ));
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Unpacks a `.zip` or `.tar.gz` into `dest`. Paths leaving `dest` and links are refused.
pub fn extract(archive: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("Can't create {}: {e}", dest.display()))?;
    let name = archive.to_string_lossy().to_ascii_lowercase();
    if name.ends_with(".zip") {
        extract_zip(archive, dest)?;
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        extract_tar_gz(archive, dest)?;
    } else {
        return Err(format!(
            "{} isn't a .zip or .tar.gz archive.",
            archive.display()
        ));
    }
    flatten_single_folder(dest)
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = File::open(archive).map_err(|e| format!("Can't read {}: {e}", archive.display()))?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|e| format!("The release isn't a valid zip: {e}"))?;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| format!("The release zip is damaged: {e}"))?;
        let name = entry.name().map(|n| n.to_string()).unwrap_or_default();
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| format!("The release zip has an unsafe path: {name}"))?;
        let mode = entry.unix_mode();
        if mode.is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err(format!(
                "The release zip contains a link ({name}), which isn't allowed."
            ));
        }
        let target = dest.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target)
                .map_err(|e| format!("Can't create {}: {e}", target.display()))?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Can't create {}: {e}", parent.display()))?;
        }
        let mut out =
            File::create(&target).map_err(|e| format!("Can't write {}: {e}", target.display()))?;
        io::copy(&mut entry, &mut out)
            .map_err(|e| format!("Can't write {}: {e}", target.display()))?;
        #[cfg(unix)]
        if let Some(mode) = mode {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&target, fs::Permissions::from_mode(mode & 0o777));
        }
    }
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = File::open(archive).map_err(|e| format!("Can't read {}: {e}", archive.display()))?;
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let damaged = |e: io::Error| format!("The release archive is damaged: {e}");
    for entry in tar.entries().map_err(damaged)? {
        let mut entry = entry.map_err(damaged)?;
        let kind = entry.header().entry_type();
        if kind.is_symlink() || kind.is_hard_link() {
            return Err("The release archive contains a link, which isn't allowed.".to_string());
        }
        if !entry.unpack_in(dest).map_err(damaged)? {
            return Err("The release archive has an unsafe path.".to_string());
        }
    }
    Ok(())
}

/// Releases often wrap everything in one folder; the launcher wants `recomplauncher.json` at
/// the top.
fn flatten_single_folder(dest: &Path) -> Result<(), String> {
    if dest.join("recomplauncher.json").exists() {
        return Ok(());
    }
    let entries: Vec<_> = fs::read_dir(dest)
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    let [only] = entries.as_slice() else {
        return Ok(());
    };
    if !only.path().is_dir() {
        return Ok(());
    }
    let aside = dest.join(".flatten");
    let fail = |e: io::Error| format!("Can't arrange the release files: {e}");
    fs::rename(only.path(), &aside).map_err(fail)?;
    for child in fs::read_dir(&aside).map_err(fail)?.flatten() {
        fs::rename(child.path(), dest.join(child.file_name())).map_err(fail)?;
    }
    fs::remove_dir(&aside).map_err(fail)
}
