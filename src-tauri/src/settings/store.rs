//! Reading and writing keys of a game's TOML or JSON configuration file.
//!
//! Keys are paths (`Section/Key`). TOML is edited in place, so comments, order and keys the
//! launcher doesn't know about are kept. Values given in a manifest are written exactly as
//! given: TOML strings keep their quotes (`"\"xenos\""`).

use std::fs;
use std::path::Path;

use serde_json::{Map, Value};
use toml_edit::{DocumentMut, Item};

use super::manifest::Format;

pub enum ConfigDoc {
    Toml(DocumentMut),
    Json(Value),
}

impl ConfigDoc {
    /// A missing or empty file is an empty document.
    pub fn load(path: &Path, format: Format) -> Result<ConfigDoc, String> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(format!("Can't read {}: {e}", path.display())),
        };
        let blank = text.trim().is_empty();
        match format {
            Format::Toml => text
                .parse::<DocumentMut>()
                .map(ConfigDoc::Toml)
                .map_err(|e| format!("{} isn't valid TOML: {e}", path.display())),
            Format::Json if blank => Ok(ConfigDoc::Json(Value::Object(Map::new()))),
            Format::Json => serde_json::from_str(&text)
                .map(ConfigDoc::Json)
                .map_err(|e| format!("{} isn't valid JSON: {e}", path.display())),
            Format::Ini => Err("The ini format isn't supported".to_string()),
        }
    }

    /// The value written as the file spells it (`"xenos"`, `1`, `true`), or `None` if absent.
    pub fn get(&self, key: &str) -> Option<String> {
        match self {
            ConfigDoc::Toml(doc) => {
                let item = toml_item(doc, key)?;
                let value = item.as_value()?.clone().decorated("", "");
                Some(value.to_string())
            }
            ConfigDoc::Json(root) => serde_json::to_string(json_value(root, key)?).ok(),
        }
    }

    /// Like [`get`](Self::get) but a string comes back without its quotes.
    pub fn get_string(&self, key: &str) -> Option<String> {
        match self {
            ConfigDoc::Toml(doc) => {
                let item = toml_item(doc, key)?;
                match item.as_str() {
                    Some(s) => Some(s.to_string()),
                    None => self.get(key),
                }
            }
            ConfigDoc::Json(root) => match json_value(root, key)? {
                Value::String(s) => Some(s.clone()),
                other => Some(other.to_string()),
            },
        }
    }

    /// Writes `raw` as the file's own syntax (TOML or JSON). A JSON value that doesn't parse
    /// is stored as a string.
    pub fn set_raw(&mut self, key: &str, raw: &str) -> Result<(), String> {
        match self {
            ConfigDoc::Toml(doc) => {
                let value: toml_edit::Value = raw
                    .trim()
                    .parse()
                    .map_err(|e| format!("`{raw}` isn't a valid TOML value for `{key}`: {e}"))?;
                set_toml(doc, key, value)
            }
            ConfigDoc::Json(root) => {
                let value =
                    serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()));
                set_json(root, key, value)
            }
        }
    }

    pub fn set_string(&mut self, key: &str, text: &str) -> Result<(), String> {
        match self {
            ConfigDoc::Toml(doc) => set_toml(doc, key, toml_edit::Value::from(text)),
            ConfigDoc::Json(root) => set_json(root, key, Value::String(text.to_string())),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let text = match self {
            ConfigDoc::Toml(doc) => doc.to_string(),
            ConfigDoc::Json(root) => {
                serde_json::to_string_pretty(root).map_err(|e| e.to_string())? + "\n"
            }
        };
        crate::fs_util::write_atomically(path, &text)
    }
}

fn toml_item<'a>(doc: &'a DocumentMut, key: &str) -> Option<&'a Item> {
    key.split('/')
        .try_fold(doc.as_item(), |item, part| item.get(part))
}

fn set_toml(doc: &mut DocumentMut, key: &str, mut value: toml_edit::Value) -> Result<(), String> {
    let parts: Vec<&str> = key.split('/').collect();
    let (last, parents) = parts.split_last().ok_or("empty key")?;
    let mut table = doc.as_table_mut();
    for part in parents {
        table = table
            .entry(part)
            .or_insert_with(toml_edit::table)
            .as_table_mut()
            .ok_or_else(|| format!("`{part}` in `{key}` isn't a table"))?;
    }
    if let Some(old) = table.get(last).and_then(Item::as_value) {
        *value.decor_mut() = old.decor().clone();
    }
    match table.get_mut(last) {
        // Replace in place: the key's own decoration (a comment above it) stays.
        Some(item) => *item = Item::Value(value),
        None => {
            table.insert(last, Item::Value(value));
        }
    }
    Ok(())
}

fn json_value<'a>(root: &'a Value, key: &str) -> Option<&'a Value> {
    key.split('/').try_fold(root, |value, part| value.get(part))
}

fn set_json(root: &mut Value, key: &str, value: Value) -> Result<(), String> {
    let parts: Vec<&str> = key.split('/').collect();
    let (last, parents) = parts.split_last().ok_or("empty key")?;
    let mut object = root
        .as_object_mut()
        .ok_or("the configuration isn't a JSON object")?;
    for part in parents {
        object = object
            .entry(part.to_string())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or_else(|| format!("`{part}` in `{key}` isn't an object"))?;
    }
    object.insert(last.to_string(), value);
    Ok(())
}
