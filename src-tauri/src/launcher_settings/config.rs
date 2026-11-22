//! Reading `launcher.json` outside the settings engine, for what the launcher itself needs at
//! startup or on a command (the engine edits the same file and keeps what it doesn't know).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

pub fn file(config_dir: &Path) -> PathBuf {
    config_dir.join("launcher.json")
}

/// The whole file; an empty object when there is none yet.
pub fn read(config_dir: &Path) -> Result<Value, String> {
    let path = file(config_dir);
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| format!("{} isn't valid JSON: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Object(Map::new())),
        Err(e) => Err(format!("Can't read {}: {e}", path.display())),
    }
}

/// Reads the file, lets `change` edit its top-level object, and writes it back, keeping every
/// other key as it was.
pub fn update(
    config_dir: &Path,
    change: impl FnOnce(&mut Map<String, Value>) -> Result<(), String>,
) -> Result<(), String> {
    let mut config = read(config_dir)?;
    let root = config
        .as_object_mut()
        .ok_or_else(|| format!("{} isn't a JSON object.", file(config_dir).display()))?;
    change(root)?;
    let text = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())? + "\n";
    crate::fs_util::write_atomically(&file(config_dir), &text)
}
