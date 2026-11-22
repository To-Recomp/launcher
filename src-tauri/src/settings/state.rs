//! Choices of `args` settings, remembered per game in `<data>/launcher-settings.json`.

use std::collections::BTreeMap;
use std::fs;

use serde_json::Value;

use super::paths::Instance;
use super::store::write_atomically;

pub type State = BTreeMap<String, Value>;

pub fn load(instance: &Instance) -> Result<State, String> {
    let path = instance.state_path();
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| format!("{} isn't valid JSON: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(State::new()),
        Err(e) => Err(format!("Can't read {}: {e}", path.display())),
    }
}

pub fn save(instance: &Instance, state: &State) -> Result<(), String> {
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())? + "\n";
    write_atomically(&instance.state_path(), &text)
}
