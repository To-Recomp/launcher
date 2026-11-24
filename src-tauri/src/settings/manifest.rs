//! The `settings` and `config` parts of a game's `recomplauncher.json`.
//!
//! The format is shared with RecompLauncher, so unknown fields are ignored. Two optional
//! fields are launcher extensions that RecompLauncher ignores in turn: `tab` (which settings
//! sub-tab a setting belongs to), `control` (a presentation hint for `choices`) and the
//! `text` control (a free-text value).
//!
//! The same file also says how to run the game: `inputs`, `start` and `prepare` (see
//! `library`).

use std::collections::{BTreeMap, HashSet};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// Sub-tabs a setting can live in. "About" is built into the launcher.
pub const TABS: [&str; 6] = ["Display", "Graphics", "Audio", "Controls", "Saves", "Game"];
pub const DEFAULT_TAB: &str = "Game";
/// `in` value of settings that go on the command line instead of in a file.
pub const ARGS: &str = "args";
pub const MAIN: &str = "main";
pub const MAX_TEXT_LEN: usize = 256;

#[derive(Debug, Default, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub config: Vec<ConfigFile>,
    #[serde(default)]
    pub settings: Vec<Setting>,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub inputs: Vec<Input>,
    /// Paths of the game's saves (`{data}/saves`), kept when a build is deleted.
    #[serde(default)]
    pub saves: Vec<String>,
    pub start: Option<Start>,
    pub prepare: Option<Prepare>,
}

/// A file or folder the player gives (a disc image).
#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub id: String,
    pub label: Option<String>,
    /// `game` (default), `disc`, `dlc`, `update`, ...
    pub role: Option<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub formats: Vec<InputFormat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InputFormat {
    /// `file` or `folder`.
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Start {
    /// Paths inside the release, tried in order; the first that exists runs.
    pub program: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Prepare {
    pub program: Vec<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub label: Option<String>,
    /// Files the step leaves when it has finished.
    #[serde(default)]
    pub done: Vec<String>,
    /// The step can run again from what it kept, without the player's copy.
    #[serde(default)]
    pub keeps_copy: bool,
}

impl Input {
    pub fn role(&self) -> &str {
        self.role.as_deref().unwrap_or("game")
    }

    /// The copy of the game itself (not DLC or an update): left out when a rebuild keeps it.
    pub fn is_copy(&self) -> bool {
        matches!(self.role(), "game" | "disc")
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    #[serde(default = "main_id")]
    pub id: String,
    pub format: Option<Format>,
    pub file: FileSpec,
}

fn main_id() -> String {
    MAIN.to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Toml,
    Json,
    Ini,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FileSpec {
    One(String),
    Candidates(Vec<FileCandidate>),
}

#[derive(Debug, Deserialize)]
pub struct FileCandidate {
    pub path: String,
    #[serde(rename = "if")]
    pub condition: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    pub id: String,
    #[serde(default)]
    pub advanced: bool,
}

#[derive(Debug, Deserialize)]
pub struct Setting {
    pub id: String,
    pub label: String,
    pub help: Option<String>,
    pub group: Option<String>,
    pub tab: Option<String>,
    #[serde(default)]
    pub advanced: bool,
    /// A config id, or `"args"`. Defaults to the `main` config.
    #[serde(rename = "in")]
    pub target: Option<String>,
    pub control: Option<String>,
    pub choices: Option<Vec<Choice>>,
    pub slider: Option<Slider>,
    pub bindings: Option<Bindings>,
    pub text: Option<TextControl>,
    /// Command line of a slider or text setting; `{value}` is replaced.
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default, deserialize_with = "scalar_map")]
    pub defaults: BTreeMap<String, String>,
    #[serde(default, deserialize_with = "scalar_map")]
    pub when: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub label: String,
    #[serde(default, deserialize_with = "scalar_map")]
    pub set: BTreeMap<String, String>,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Slider {
    pub key: Option<String>,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Bindings {
    pub separator: Option<String>,
    pub max: Option<usize>,
    #[serde(default)]
    pub names: BTreeMap<String, String>,
    pub actions: Vec<BindingAction>,
}

#[derive(Debug, Deserialize)]
pub struct BindingAction {
    pub label: String,
    pub key: String,
    #[serde(default)]
    pub pad: bool,
}

#[derive(Debug, Deserialize)]
pub struct TextControl {
    pub key: Option<String>,
}

/// Settings and defaults hold numbers and booleans as readily as strings; keep them as text.
fn scalar_map<'de, D>(deserializer: D) -> Result<BTreeMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: BTreeMap<String, Value> = BTreeMap::deserialize(deserializer)?;
    raw.into_iter()
        .map(|(key, value)| match value {
            Value::String(s) => Ok((key, s)),
            Value::Number(n) => Ok((key, n.to_string())),
            Value::Bool(b) => Ok((key, b.to_string())),
            other => Err(D::Error::custom(format!(
                "the value of `{key}` must be a string, a number or a boolean, not {other}"
            ))),
        })
        .collect()
}

impl Manifest {
    pub fn parse(raw: &str) -> Result<Manifest, String> {
        let manifest: Manifest =
            serde_json::from_str(raw).map_err(|e| format!("Invalid recomplauncher.json: {e}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn config(&self, id: &str) -> Option<&ConfigFile> {
        self.config.iter().find(|c| c.id == id)
    }

    pub fn setting(&self, id: &str) -> Option<&Setting> {
        self.settings.iter().find(|s| s.id == id)
    }

    pub fn group_is_advanced(&self, group: &str) -> bool {
        self.groups.iter().any(|g| g.id == group && g.advanced)
    }

    fn validate(&self) -> Result<(), String> {
        let mut input_ids = HashSet::new();
        for input in &self.inputs {
            if input.id.is_empty() || !input_ids.insert(input.id.as_str()) {
                return Err(invalid(format!(
                    "input id `{}` is empty or declared twice",
                    input.id
                )));
            }
        }
        if let Some(start) = &self.start {
            validate_command("start", &start.program, &start.env)?;
        }
        if let Some(prepare) = &self.prepare {
            validate_command("prepare", &prepare.program, &prepare.env)?;
        }

        let mut config_ids = HashSet::new();
        for config in &self.config {
            if !config_ids.insert(config.id.as_str()) {
                return Err(invalid(format!("config `{}` is declared twice", config.id)));
            }
            if config.format == Some(Format::Ini) {
                return Err(invalid(format!(
                    "config `{}`: the `ini` format isn't supported yet (use toml or json)",
                    config.id
                )));
            }
        }

        let mut ids = HashSet::new();
        for setting in &self.settings {
            if setting.id.is_empty() || !ids.insert(setting.id.as_str()) {
                return Err(invalid(format!(
                    "setting id `{}` is empty or declared twice",
                    setting.id
                )));
            }
            self.validate_setting(setting)?;
        }
        Ok(())
    }

    fn validate_setting(&self, s: &Setting) -> Result<(), String> {
        let err = |msg: &str| Err(invalid(format!("setting `{}`: {msg}", s.id)));

        let controls = [
            s.choices.is_some(),
            s.slider.is_some(),
            s.bindings.is_some(),
            s.text.is_some(),
        ];
        if controls.iter().filter(|present| **present).count() != 1 {
            return err("needs exactly one of `choices`, `slider`, `bindings` or `text`");
        }
        if let Some(tab) = &s.tab {
            if !TABS.contains(&tab.as_str()) {
                return err(&format!("unknown tab `{tab}` (one of {})", TABS.join(", ")));
            }
        }

        let in_args = s.target.as_deref() == Some(ARGS);
        if !in_args && self.config(s.config_id()).is_none() {
            return err(&format!("config `{}` isn't declared", s.config_id()));
        }

        if let Some(choices) = &s.choices {
            if choices.is_empty() {
                return err("`choices` is empty");
            }
            if in_args && choices.iter().any(|c| !c.set.is_empty()) {
                return err("choices of an `args` setting use `args`, not `set`");
            }
            if !in_args && choices.iter().any(|c| c.set.is_empty()) {
                return err("every choice needs a `set`");
            }
            match s.control.as_deref() {
                None | Some("segmented") | Some("dropdown") => {}
                Some("toggle") if choices.len() == 2 => {}
                Some("toggle") => return err("a toggle needs exactly two choices"),
                Some(other) => return err(&format!("unknown control `{other}`")),
            }
        } else if s.control.is_some() {
            return err("`control` only applies to `choices`");
        }

        if let Some(slider) = &s.slider {
            let sane = slider.min < slider.max && slider.step > 0.0;
            if !sane {
                return err("a slider needs min < max and step > 0");
            }
            if let Some(format) = &slider.format {
                if crate::settings::format::parse(format).is_none() {
                    return err(&format!("unsupported number format `{format}`"));
                }
            }
            if !in_args && slider.key.is_none() {
                return err("a slider needs a `key`");
            }
        }
        if let Some(text) = &s.text {
            if !in_args && text.key.is_none() {
                return err("a text setting needs a `key`");
            }
        }
        if in_args
            && (s.slider.is_some() || s.text.is_some())
            && !s.args.iter().any(|a| a.contains("{value}"))
        {
            return err("an `args` slider or text needs `args` containing `{value}`");
        }
        if let Some(bindings) = &s.bindings {
            if in_args {
                return err("key bindings can't go on the command line");
            }
            if bindings.actions.is_empty() {
                return err("`bindings.actions` is empty");
            }
            if bindings.max == Some(0) {
                return err("`bindings.max` must be at least 1");
            }
        }
        Ok(())
    }
}

impl Setting {
    pub fn is_args(&self) -> bool {
        self.target.as_deref() == Some(ARGS)
    }

    pub fn config_id(&self) -> &str {
        self.target.as_deref().unwrap_or(MAIN)
    }

    pub fn tab(&self) -> &str {
        self.tab.as_deref().unwrap_or(DEFAULT_TAB)
    }
}

/// Environment names the game may not set: system ones and the launcher's own.
const RESERVED_ENV: [&str; 6] = [
    "PATH",
    "HOME",
    "USER",
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "TMPDIR",
];

fn validate_command(
    what: &str,
    program: &[String],
    env: &BTreeMap<String, String>,
) -> Result<(), String> {
    if program.is_empty() {
        return Err(invalid(format!("`{what}.program` is empty")));
    }
    for name in env.keys() {
        let plain = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !plain || RESERVED_ENV.contains(&name.as_str()) || name.starts_with("RECOMPLAUNCHER") {
            return Err(invalid(format!("`{what}.env`: `{name}` can't be set")));
        }
    }
    Ok(())
}

fn invalid(msg: String) -> String {
    format!("Invalid recomplauncher.json: {msg}")
}
