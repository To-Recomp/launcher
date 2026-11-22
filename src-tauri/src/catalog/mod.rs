//! The game catalog: `catalog.json` of the `To-Recomp/catalog` repository, fetched with a
//! conditional request, validated against its JSON Schema (format 1) and kept on disk so the
//! launcher works offline. An invalid download never replaces the last valid copy.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::Manager;

use crate::fs_util::write_atomically;

pub const DEFAULT_URL: &str =
    "https://raw.githubusercontent.com/To-Recomp/catalog/main/catalog.json";
/// Points the launcher at another catalog (a local server, a fork) for development.
const URL_ENV: &str = "TO_RECOMP_CATALOG_URL";
/// Copy of `schema/catalog-1.json` of the catalog repository.
const SCHEMA: &str = include_str!("catalog-1.schema.json");
const MAX_BYTES: u64 = 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(10);
const MAX_REPORTED_ERRORS: usize = 3;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub games: Vec<Game>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    #[serde(rename(serialize = "japanReleaseYear", deserialize = "japan_release_year"))]
    pub japan_release_year: u32,
    pub repository: String,
    pub platforms: Vec<Platform>,
}

/// One console of a game. The regions are information (what the game covers there): the
/// launcher lists consoles, and an instance is one console.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Platform {
    pub platform: String,
    pub regions: Vec<String>,
    pub state: State,
    /// Only for a playable console.
    pub maturity: Option<Maturity>,
}

/// What the launcher can do with a console.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    Planned,
    InProgress,
    /// A release can be installed.
    Playable,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Maturity {
    Alpha,
    Beta,
    Stable,
}

/// Parses and validates a catalog; games come back ordered by first Japanese release year.
pub fn parse(text: &str) -> Result<Catalog, String> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| format!("The catalog isn't valid JSON: {e}"))?;

    let errors: Vec<String> = validator()
        .iter_errors(&value)
        .map(|e| format!("{} {e}", e.instance_path))
        .collect();
    if !errors.is_empty() {
        let shown: Vec<&str> = errors
            .iter()
            .take(MAX_REPORTED_ERRORS)
            .map(String::as_str)
            .collect();
        return Err(format!(
            "The catalog doesn't match format 1: {}",
            shown.join("; ")
        ));
    }

    let mut catalog: Catalog =
        serde_json::from_value(value).map_err(|e| format!("The catalog is unreadable: {e}"))?;

    let mut ids = HashSet::new();
    for game in &catalog.games {
        if !ids.insert(game.id.as_str()) {
            return Err(format!(
                "The catalog lists the game id \"{}\" twice.",
                game.id
            ));
        }
        let mut platforms = HashSet::new();
        for p in &game.platforms {
            if !platforms.insert(p.platform.as_str()) {
                return Err(format!(
                    "The catalog lists {} twice for \"{}\".",
                    p.platform, game.id
                ));
            }
        }
    }

    catalog
        .games
        .sort_by(|a, b| (a.japan_release_year, &a.name).cmp(&(b.japan_release_year, &b.name)));
    Ok(catalog)
}

fn validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| {
        let schema: Value =
            serde_json::from_str(SCHEMA).expect("the bundled catalog schema is JSON");
        jsonschema::options()
            .should_validate_formats(true)
            .build(&schema)
            .expect("the bundled catalog schema is a valid schema")
    })
}

/// The last valid copy kept in `dir`, if any. A damaged copy counts as none.
pub fn load_cached(dir: &Path) -> Option<Catalog> {
    parse(&fs::read_to_string(dir.join("catalog.json")).ok()?).ok()
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Refresh {
    /// The best catalog we have: the new one, or the last valid copy.
    pub catalog: Option<Catalog>,
    /// True when a newer catalog was downloaded.
    pub updated: bool,
    /// Why the download didn't produce a newer catalog (offline, invalid, ...), if it didn't
    /// because of a problem. "Nothing new" isn't one.
    pub error: Option<String>,
}

pub fn refresh(dir: &Path, url: &str) -> Refresh {
    let cached = load_cached(dir);
    match fetch(dir, url, cached.is_some()) {
        Ok(Some(fresh)) => Refresh {
            catalog: Some(fresh),
            updated: true,
            error: None,
        },
        Ok(None) => Refresh {
            catalog: cached,
            updated: false,
            error: None,
        },
        Err(error) => Refresh {
            catalog: cached,
            updated: false,
            error: Some(error),
        },
    }
}

/// `Ok(None)`: the server says our copy is current.
fn fetch(dir: &Path, url: &str, have_valid_copy: bool) -> Result<Option<Catalog>, String> {
    let etag_path = dir.join("etag");
    let agent: ureq::Agent = crate::net::agent_builder()?
        .timeout_global(Some(TIMEOUT))
        .http_status_as_error(false)
        .build()
        .into();

    let mut request = agent.get(url);
    if have_valid_copy {
        if let Ok(etag) = fs::read_to_string(&etag_path) {
            request = request.header("If-None-Match", etag.trim());
        }
    }
    let mut response = request
        .call()
        .map_err(|e| format!("Can't reach the game catalog: {e}"))?;

    match response.status().as_u16() {
        304 if have_valid_copy => Ok(None),
        200 => {
            let etag = response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            let body = response
                .body_mut()
                .with_config()
                .limit(MAX_BYTES)
                .read_to_string()
                .map_err(|e| format!("Can't read the game catalog: {e}"))?;
            let catalog = parse(&body)?;
            write_atomically(&dir.join("catalog.json"), &body)?;
            match etag {
                Some(etag) => write_atomically(&etag_path, &etag)?,
                None => {
                    let _ = fs::remove_file(&etag_path);
                }
            }
            Ok(Some(catalog))
        }
        status => Err(format!("The game catalog server answered {status}.")),
    }
}

fn cache_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("catalog"))
        .map_err(|e| format!("No place to keep the game catalog: {e}"))
}

/// The saved catalog, without touching the network.
#[tauri::command(async)]
pub fn cached_catalog(app: tauri::AppHandle) -> Option<Catalog> {
    load_cached(&cache_dir(&app).ok()?)
}

#[tauri::command(async)]
pub fn refresh_catalog(app: tauri::AppHandle) -> Refresh {
    let url = std::env::var(URL_ENV).unwrap_or_else(|_| DEFAULT_URL.to_string());
    match cache_dir(&app) {
        Ok(dir) => refresh(&dir, &url),
        Err(error) => Refresh {
            catalog: None,
            updated: false,
            error: Some(error),
        },
    }
}

#[cfg(test)]
mod tests;
