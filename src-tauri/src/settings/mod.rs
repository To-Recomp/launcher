//! Settings screens generated from a game's `recomplauncher.json`.
//!
//! The frontend never sees the manifest: [`load_view`] resolves it against the game's
//! configuration files into a ready-to-render [`SettingsView`], and [`apply`] validates and
//! writes one change. Settings that go on the command line are remembered in the game's data
//! folder and turned into arguments by [`launch_args`].

mod format;
mod keys;
pub(crate) mod manifest;
pub(crate) mod paths;
pub(crate) mod saves;
mod state;
mod store;

use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use manifest::{Bindings, Manifest, Setting};
pub use paths::Instance;
use store::ConfigDoc;

const DEFAULT_SEPARATOR: &str = "|";
const DEFAULT_MAX_KEYS: usize = 2;
const SEGMENTED_MAX_CHOICES: usize = 3;
const SEGMENTED_MAX_LABEL: usize = 14;

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub settings: Vec<SettingView>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SettingView {
    pub id: String,
    pub label: String,
    pub help: Option<String>,
    pub tab: String,
    pub group: Option<String>,
    pub advanced: bool,
    /// False when the setting's `when` condition doesn't hold: shown greyed out.
    pub enabled: bool,
    pub control: Control,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Control {
    Choices {
        style: ChoiceStyle,
        options: Vec<String>,
        /// Index of the matching choice; `None` when the file holds something else.
        value: Option<usize>,
        /// What the file holds when no choice matches.
        custom: Option<String>,
    },
    Slider {
        min: f64,
        max: f64,
        step: f64,
        value: f64,
        format: Option<String>,
    },
    Text {
        value: String,
    },
    Bindings {
        max: usize,
        actions: Vec<ActionView>,
    },
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum ChoiceStyle {
    /// Two choices, the first off and the second on.
    Toggle,
    Segmented,
    Dropdown,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    pub label: String,
    pub key: String,
    pub keys: Vec<String>,
    pub default: Vec<String>,
    pub pad: bool,
}

/// One change made by the player.
#[derive(Debug, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SettingInput {
    Choice {
        index: usize,
    },
    Number {
        value: f64,
    },
    Text {
        value: String,
    },
    /// The keys of one binding action, in order.
    Bindings {
        action: String,
        keys: Vec<String>,
    },
}

fn read_manifest(instance: &Instance) -> Result<Manifest, String> {
    let path = instance.manifest_path();
    let raw = fs::read_to_string(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => {
            "This release has no recomplauncher.json, so it has no settings to show.".to_string()
        }
        _ => format!("Can't read {}: {e}", path.display()),
    })?;
    Manifest::parse(&raw)
}

/// The configuration files of a manifest, loaded on first use and kept for one operation.
struct Files<'a> {
    instance: &'a Instance,
    manifest: &'a Manifest,
    docs: HashMap<String, ConfigDoc>,
}

impl<'a> Files<'a> {
    fn new(instance: &'a Instance, manifest: &'a Manifest) -> Self {
        Files {
            instance,
            manifest,
            docs: HashMap::new(),
        }
    }

    fn doc(&mut self, id: &str) -> Result<&mut ConfigDoc, String> {
        if !self.docs.contains_key(id) {
            let config = self
                .manifest
                .config(id)
                .ok_or(format!("config `{id}` isn't declared"))?;
            let (path, format) = self.instance.config_location(config)?;
            self.docs
                .insert(id.to_string(), ConfigDoc::load(&path, format)?);
        }
        Ok(self.docs.get_mut(id).expect("just inserted"))
    }

    fn save(&mut self, id: &str) -> Result<(), String> {
        let config = self
            .manifest
            .config(id)
            .ok_or(format!("config `{id}` isn't declared"))?;
        let (path, _) = self.instance.config_location(config)?;
        self.doc(id)?.save(&path)
    }

    /// The game's default of `key`: the asking setting's own, else any setting's of the same
    /// file (a `when` condition may name a key another setting owns).
    fn default_of(&self, setting: &Setting, key: &str) -> Option<String> {
        setting.defaults.get(key).cloned().or_else(|| {
            self.manifest
                .settings
                .iter()
                .filter(|other| other.config_id() == setting.config_id())
                .find_map(|other| other.defaults.get(key).cloned())
        })
    }

    /// The file's value of `key`, else its default.
    fn raw(&mut self, setting: &Setting, key: &str) -> Result<Option<String>, String> {
        let from_file = self.doc(setting.config_id())?.get(key);
        Ok(from_file.or_else(|| self.default_of(setting, key)))
    }

    fn string(&mut self, setting: &Setting, key: &str) -> Result<Option<String>, String> {
        let from_file = self.doc(setting.config_id())?.get_string(key);
        Ok(from_file.or_else(|| {
            self.default_of(setting, key)
                .map(|d| unquote(&d).to_string())
        }))
    }
}

pub fn load_view(instance: &Instance) -> Result<SettingsView, String> {
    let manifest = read_manifest(instance)?;
    build_view(instance, &manifest)
}

pub(crate) fn build_view(instance: &Instance, manifest: &Manifest) -> Result<SettingsView, String> {
    let state = state::load(instance)?;
    let mut files = Files::new(instance, manifest);
    let mut settings = Vec::with_capacity(manifest.settings.len());
    for setting in &manifest.settings {
        let enabled = is_enabled(&mut files, setting)?;
        let advanced = setting.advanced
            || setting
                .group
                .as_deref()
                .is_some_and(|g| manifest.group_is_advanced(g));
        settings.push(SettingView {
            id: setting.id.clone(),
            label: setting.label.clone(),
            help: setting.help.clone(),
            tab: setting.tab().to_string(),
            group: setting.group.clone(),
            advanced,
            enabled,
            control: build_control(&mut files, &state, setting)?,
        });
    }
    Ok(SettingsView { settings })
}

fn is_enabled(files: &mut Files, setting: &Setting) -> Result<bool, String> {
    for (key, expected) in &setting.when {
        let actual = files.raw(setting, key)?;
        if actual.as_deref().map(unquote) != Some(unquote(expected)) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn build_control(files: &mut Files, state: &state::State, s: &Setting) -> Result<Control, String> {
    if let Some(choices) = &s.choices {
        let options: Vec<String> = choices.iter().map(|c| c.label.clone()).collect();
        let (value, custom) = if s.is_args() {
            let index = state
                .get(&s.id)
                .and_then(Value::as_u64)
                .map_or(0, |i| i as usize);
            (Some(index.min(choices.len() - 1)), None)
        } else {
            let mut found = None;
            for (index, choice) in choices.iter().enumerate() {
                let mut all = true;
                for (key, expected) in &choice.set {
                    let actual = files.raw(s, key)?;
                    if actual.as_deref().map(unquote) != Some(unquote(expected)) {
                        all = false;
                        break;
                    }
                }
                if all {
                    found = Some(index);
                    break;
                }
            }
            let custom = match (found, choices[0].set.keys().next()) {
                (None, Some(key)) => files.string(s, key)?,
                _ => None,
            };
            (found, custom)
        };
        return Ok(Control::Choices {
            style: choice_style(s, &options),
            options,
            value,
            custom,
        });
    }

    if let Some(slider) = &s.slider {
        let stored = if s.is_args() {
            state.get(&s.id).and_then(Value::as_f64)
        } else {
            let key = slider.key.as_deref().unwrap_or_default();
            files.string(s, key)?.and_then(|t| t.trim().parse().ok())
        };
        return Ok(Control::Slider {
            min: slider.min,
            max: slider.max,
            step: slider.step,
            value: stored.unwrap_or(slider.min),
            format: slider.format.clone(),
        });
    }

    if let Some(text) = &s.text {
        let value = if s.is_args() {
            state.get(&s.id).and_then(Value::as_str).map(str::to_string)
        } else {
            files.string(s, text.key.as_deref().unwrap_or_default())?
        };
        return Ok(Control::Text {
            value: value.unwrap_or_default(),
        });
    }

    let bindings = s.bindings.as_ref().expect("validated: one control");
    let separator = separator(bindings);
    let mut actions = Vec::with_capacity(bindings.actions.len());
    for action in &bindings.actions {
        let current = files.string(s, &action.key)?;
        let default = s.defaults.get(&action.key).map(|d| unquote(d).to_string());
        actions.push(ActionView {
            label: action.label.clone(),
            key: action.key.clone(),
            keys: split_keys(
                bindings,
                separator,
                current.as_deref().or(default.as_deref()),
            ),
            default: split_keys(bindings, separator, default.as_deref()),
            pad: action.pad,
        });
    }
    Ok(Control::Bindings {
        max: bindings.max.unwrap_or(DEFAULT_MAX_KEYS),
        actions,
    })
}

fn choice_style(setting: &Setting, labels: &[String]) -> ChoiceStyle {
    match setting.control.as_deref() {
        Some("toggle") => return ChoiceStyle::Toggle,
        Some("segmented") => return ChoiceStyle::Segmented,
        Some("dropdown") => return ChoiceStyle::Dropdown,
        _ => {}
    }
    const OFF_ON: [(&str, &str); 4] = [
        ("off", "on"),
        ("no", "yes"),
        ("disabled", "enabled"),
        ("false", "true"),
    ];
    if let [off, on] = labels {
        let pair = (off.to_lowercase(), on.to_lowercase());
        if OFF_ON.iter().any(|(a, b)| pair.0 == *a && pair.1 == *b) {
            return ChoiceStyle::Toggle;
        }
    }
    if labels.len() <= SEGMENTED_MAX_CHOICES
        && labels
            .iter()
            .all(|l| l.chars().count() <= SEGMENTED_MAX_LABEL)
    {
        ChoiceStyle::Segmented
    } else {
        ChoiceStyle::Dropdown
    }
}

fn separator(bindings: &Bindings) -> &str {
    bindings.separator.as_deref().unwrap_or(DEFAULT_SEPARATOR)
}

/// File spelling to the launcher's own key names (`names` maps ours to the file's).
fn split_keys(bindings: &Bindings, separator: &str, raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or_default()
        .split(separator)
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(|k| {
            bindings
                .names
                .iter()
                .find(|(_, file_name)| file_name.as_str() == k)
                .map_or(k, |(ours, _)| ours.as_str())
                .to_string()
        })
        .collect()
}

fn unquote(raw: &str) -> &str {
    let raw = raw.trim();
    raw.strip_prefix('"')
        .and_then(|r| r.strip_suffix('"'))
        .unwrap_or(raw)
}

/// Validates and writes one change, then returns the refreshed view.
pub fn apply(instance: &Instance, id: &str, input: SettingInput) -> Result<SettingsView, String> {
    let manifest = read_manifest(instance)?;
    apply_with(instance, &manifest, id, input)
}

/// [`apply`] for a manifest the caller already has (the launcher's own settings are built in).
pub(crate) fn apply_with(
    instance: &Instance,
    manifest: &Manifest,
    id: &str,
    input: SettingInput,
) -> Result<SettingsView, String> {
    let setting = manifest
        .setting(id)
        .ok_or(format!("Unknown setting `{id}`."))?;
    let mut files = Files::new(instance, manifest);
    if !is_enabled(&mut files, setting)? {
        return Err(format!(
            "{} isn't available with the current settings.",
            setting.label
        ));
    }

    let mut state = state::load(instance)?;
    let target = setting.config_id().to_string();
    let label = &setting.label;

    match input {
        SettingInput::Choice { index } => {
            let choices = setting
                .choices
                .as_ref()
                .ok_or(format!("{label} has no choices."))?;
            let choice = choices
                .get(index)
                .ok_or(format!("{label}: choice {index} doesn't exist."))?;
            if setting.is_args() {
                state.insert(id.to_string(), Value::from(index));
            } else {
                let doc = files.doc(&target)?;
                for (key, value) in &choice.set {
                    doc.set_raw(key, value)?;
                }
            }
        }
        SettingInput::Number { value } => {
            let slider = setting
                .slider
                .as_ref()
                .ok_or(format!("{label} isn't a slider."))?;
            if !value.is_finite() || value < slider.min || value > slider.max {
                return Err(format!(
                    "{label} must be between {} and {}.",
                    slider.min, slider.max
                ));
            }
            if setting.is_args() {
                state.insert(id.to_string(), Value::from(value));
            } else {
                let number_format = slider
                    .format
                    .as_deref()
                    .map_or(Some(format::NumberFormat::General), format::parse)
                    .ok_or("unsupported number format")?;
                let key = slider.key.as_deref().unwrap_or_default();
                files
                    .doc(&target)?
                    .set_raw(key, &format::render(number_format, value))?;
            }
        }
        SettingInput::Text { value } => {
            let text = setting
                .text
                .as_ref()
                .ok_or(format!("{label} isn't a text setting."))?;
            if value.chars().count() > manifest::MAX_TEXT_LEN || value.chars().any(char::is_control)
            {
                return Err(format!(
                    "{label} must be plain text of at most {} characters.",
                    manifest::MAX_TEXT_LEN
                ));
            }
            if setting.is_args() {
                state.insert(id.to_string(), Value::from(value));
            } else {
                files
                    .doc(&target)?
                    .set_string(text.key.as_deref().unwrap_or_default(), &value)?;
            }
        }
        SettingInput::Bindings { action, keys } => {
            let bindings = setting
                .bindings
                .as_ref()
                .ok_or(format!("{label} has no key bindings."))?;
            let found = bindings
                .actions
                .iter()
                .find(|a| a.key == action)
                .ok_or(format!("{label}: unknown action `{action}`."))?;
            let max = bindings.max.unwrap_or(DEFAULT_MAX_KEYS);
            if keys.len() > max {
                return Err(format!("An action takes at most {max} keys."));
            }
            for (i, key) in keys.iter().enumerate() {
                if !keys::is_valid(key, found.pad) {
                    return Err(format!("`{key}` isn't a key this setting accepts."));
                }
                if keys[..i].contains(key) {
                    return Err(format!("`{key}` is listed twice."));
                }
            }
            let written: Vec<&str> = keys
                .iter()
                .map(|k| bindings.names.get(k).map_or(k.as_str(), String::as_str))
                .collect();
            files
                .doc(&target)?
                .set_string(&found.key, &written.join(separator(bindings)))?;
        }
    }

    if setting.is_args() {
        state::save(instance, &state)?;
    } else {
        files.save(&target)?;
    }
    build_view(instance, manifest)
}

/// Command line contributed by `args` settings: a choice's arguments, and a slider or text
/// setting's once the player has set it.
pub fn launch_args(instance: &Instance) -> Result<Vec<String>, String> {
    let manifest = read_manifest(instance)?;
    let state = state::load(instance)?;
    let mut args = Vec::new();
    for setting in manifest.settings.iter().filter(|s| s.is_args()) {
        let remembered = state.get(&setting.id);
        if let Some(choices) = &setting.choices {
            let index = remembered.and_then(Value::as_u64).map_or(0, |i| i as usize);
            if let Some(choice) = choices.get(index).or(choices.first()) {
                args.extend(choice.args.iter().cloned());
            }
        } else if let Some(value) = remembered {
            let text = match (&setting.slider, value) {
                (Some(slider), v) => {
                    let number_format = slider
                        .format
                        .as_deref()
                        .map_or(Some(format::NumberFormat::General), format::parse)
                        .ok_or("unsupported number format")?;
                    format::render(number_format, v.as_f64().unwrap_or(slider.min))
                }
                (None, v) => v.as_str().unwrap_or_default().to_string(),
            };
            args.extend(setting.args.iter().map(|a| a.replace("{value}", &text)));
        }
    }
    Ok(args)
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub install_dir: String,
    pub data_dir: Option<String>,
}

impl From<Location> for Instance {
    fn from(l: Location) -> Instance {
        Instance::new(l.install_dir.into(), l.data_dir.map(Into::into))
    }
}

#[tauri::command(async)]
pub fn load_settings(location: Location) -> Result<SettingsView, String> {
    load_view(&location.into())
}

#[tauri::command(async)]
pub fn set_setting(
    location: Location,
    id: String,
    input: SettingInput,
) -> Result<SettingsView, String> {
    apply(&location.into(), &id, input)
}

#[tauri::command(async)]
pub fn settings_launch_args(location: Location) -> Result<Vec<String>, String> {
    launch_args(&location.into())
}

#[cfg(test)]
mod tests;
