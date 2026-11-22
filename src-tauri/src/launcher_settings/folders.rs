//! Where the games are kept (installations and their instances' data), chosen by the player.
//!
//! Stored as `library.folder` in `launcher.json`, next to the settings the engine edits. It is
//! read once, when the launcher starts: a change applies on the next start, and games that are
//! already installed stay where they were.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use super::config;
use crate::fs_util::write_atomically;

/// The folder the player chose, `None` for the default one.
pub fn configured(config_dir: &Path) -> Result<Option<PathBuf>, String> {
    Ok(config::read(config_dir)?
        .pointer("/library/folder")
        .and_then(Value::as_str)
        .filter(|folder| !folder.is_empty())
        .map(PathBuf::from))
}

/// A folder the games can live in: absolute, existing and writable.
pub fn validate(folder: &Path) -> Result<(), String> {
    if !folder.is_absolute() {
        return Err(format!("{} isn't an absolute path.", folder.display()));
    }
    if !folder.is_dir() {
        return Err(format!("{} isn't an existing folder.", folder.display()));
    }
    let probe = folder.join(".to-recomp-write-test");
    fs::write(&probe, b"").map_err(|e| format!("Can't write in {}: {e}", folder.display()))?;
    let _ = fs::remove_file(&probe);
    Ok(())
}

/// Remembers `folder` (`None`: back to the default), after checking it.
pub fn set(config_dir: &Path, folder: Option<&str>) -> Result<(), String> {
    if let Some(folder) = folder {
        validate(Path::new(folder))?;
    }
    let mut config = config::read(config_dir)?;
    let root = config.as_object_mut().ok_or_else(|| {
        format!(
            "{} isn't a JSON object.",
            config::file(config_dir).display()
        )
    })?;
    match folder {
        Some(folder) => {
            let library = root
                .entry("library")
                .or_insert_with(|| Value::Object(Map::new()));
            library
                .as_object_mut()
                .ok_or("`library` in launcher.json isn't an object.")?
                .insert("folder".into(), Value::String(folder.to_string()));
        }
        None => {
            if let Some(library) = root.get_mut("library").and_then(Value::as_object_mut) {
                library.remove("folder");
            }
        }
    }
    let text = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())? + "\n";
    write_atomically(&config::file(config_dir), &text)
}
