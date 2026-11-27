//! Where installations and instances live, and what state an instance is in.
//!
//! ```text
//! <library>/<game id>/installation.json        what is installed (release tag, checksum)
//! <library>/<game id>/release/                 the release: `{install}`, replaced on update
//! <library>/<game id>/instances/<id>/          one platform of the game
//!     instance.json                            the player's files, which release it was prepared for
//!     data/                                    `{data}`: settings, saves, the prepared game
//!     prepare.log, game.log
//! ```
//! Updating a release replaces `release/` only: instances (and so settings, saves and the
//! extracted game files a rebuild starts from) stay.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::run::{Notice, ReleaseFound};
use crate::fs_util::write_atomically;
use crate::settings::manifest::Manifest;
use crate::settings::Instance;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstallationRecord {
    pub repository: String,
    pub tag: String,
    pub asset: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct InstanceRecord {
    pub platform: String,
    /// The player's files by input id, as picked.
    #[serde(default)]
    pub inputs: BTreeMap<String, String>,
    /// The release tag the instance was last prepared for.
    pub prepared_release: Option<String>,
    /// A file was chosen again after the last preparation: the build no longer matches the player's files
    /// and is to be made again (from these files, not from what the last build kept).
    #[serde(default)]
    pub inputs_changed: bool,
    /// What the last preparation said about the player's files; replaced by the next one.
    #[serde(default)]
    pub prepare_notes: PrepareNotes,
    /// What the player agreed to go on with (the discs are not the tested ones): the same again
    /// isn't asked a second time, a rebuild for instance.
    #[serde(default)]
    pub accepted_notes: Option<PrepareNotes>,
}

/// The `release` and `warn` lines of a preparation, in the order printed.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PrepareNotes {
    pub release: Option<ReleaseFound>,
    pub warnings: Vec<String>,
}

impl PrepareNotes {
    /// The discs don't match the ones the release was tested with: the step warned, or found
    /// them to be no known dump. A release that is only young (alpha) is not a reason.
    pub fn needs_attention(&self) -> bool {
        !self.warnings.is_empty()
            || self
                .release
                .as_ref()
                .is_some_and(|found| found.status == "unsupported")
    }

    pub fn record(&mut self, notice: &Notice) {
        match notice {
            Notice::Release(found) => self.release = Some(found.clone()),
            Notice::Warn { text } => {
                if !self.warnings.contains(text) {
                    self.warnings.push(text.clone());
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InstanceState {
    NotPrepared,
    Ready,
    /// Prepared before, but the release changed or the prepared files are gone.
    NeedsRebuild,
}

/// Game ids come from the catalog; anything else could escape the library folder.
pub fn valid_game_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// `"GameCube"` -> `gamecube`, `"Xbox Series X|S"` -> `xbox-series-x-s`. The frontend mirrors this.
pub fn instance_id(platform: &str) -> String {
    slug(platform)
}

fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() && c.is_ascii() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "x".to_string()
    } else {
        trimmed.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Library {
    root: PathBuf,
}

impl Library {
    pub fn new(root: PathBuf) -> Library {
        Library { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn game_dir(&self, game_id: &str) -> Result<PathBuf, String> {
        if !valid_game_id(game_id) {
            return Err(format!("`{game_id}` isn't a valid game id."));
        }
        Ok(self.root.join(game_id))
    }

    pub fn release_dir(&self, game_id: &str) -> Result<PathBuf, String> {
        Ok(self.game_dir(game_id)?.join("release"))
    }

    pub fn instance_dir(&self, game_id: &str, instance: &str) -> Result<PathBuf, String> {
        Ok(self.game_dir(game_id)?.join("instances").join(instance))
    }

    /// `{install}` and `{data}` of one instance.
    pub fn location(&self, game_id: &str, instance: &str) -> Result<Instance, String> {
        Ok(Instance::new(
            self.release_dir(game_id)?,
            Some(self.instance_dir(game_id, instance)?.join("data")),
        ))
    }

    pub fn installation(&self, game_id: &str) -> Result<Option<InstallationRecord>, String> {
        read_json(&self.game_dir(game_id)?.join("installation.json"))
    }

    pub fn save_installation(
        &self,
        game_id: &str,
        record: &InstallationRecord,
    ) -> Result<(), String> {
        write_json(&self.game_dir(game_id)?.join("installation.json"), record)
    }

    pub fn instance_record(
        &self,
        game_id: &str,
        instance: &str,
        platform: &str,
    ) -> Result<InstanceRecord, String> {
        let path = self.instance_dir(game_id, instance)?.join("instance.json");
        Ok(read_json(&path)?.unwrap_or_else(|| InstanceRecord {
            platform: platform.to_string(),
            ..InstanceRecord::default()
        }))
    }

    pub fn save_instance_record(
        &self,
        game_id: &str,
        instance: &str,
        record: &InstanceRecord,
    ) -> Result<(), String> {
        write_json(
            &self.instance_dir(game_id, instance)?.join("instance.json"),
            record,
        )
    }

    /// Ids of the games that have an installation.
    pub fn installed_games(&self) -> Vec<String> {
        let mut ids: Vec<String> = fs::read_dir(&self.root)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().join("installation.json").is_file())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|id| valid_game_id(id))
            .collect();
        ids.sort();
        ids
    }

    /// Instance ids and records of a game, for the instances that exist.
    pub fn instances(&self, game_id: &str) -> Result<Vec<(String, InstanceRecord)>, String> {
        let dir = self.game_dir(game_id)?.join("instances");
        let mut found = Vec::new();
        for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
            let Ok(id) = entry.file_name().into_string() else {
                continue;
            };
            if let Some(record) = read_json::<InstanceRecord>(&entry.path().join("instance.json"))?
            {
                found.push((id, record));
            }
        }
        found.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(found)
    }
}

/// Input ids a list of templates refers to (`{input:game}` -> `game`).
pub fn referenced_inputs(templates: &[String]) -> Vec<String> {
    let mut ids = Vec::new();
    for template in templates {
        let mut rest = template.as_str();
        while let Some(start) = rest.find("{input:") {
            let after = &rest[start + "{input:".len()..];
            let Some(end) = after.find('}') else { break };
            ids.push(after[..end].to_string());
            rest = &after[end + 1..];
        }
    }
    ids
}

/// What the player can do with an instance of this release.
pub fn instance_state(
    manifest: &Manifest,
    installation: &InstallationRecord,
    record: &InstanceRecord,
    location: &Instance,
) -> InstanceState {
    // Files the game is started with must still be there (a deleted disc image is asked for again).
    let mut start_templates: Vec<String> = Vec::new();
    if let Some(start) = &manifest.start {
        start_templates.extend(start.args.iter().cloned());
        start_templates.extend(start.env.values().cloned());
    }
    let inputs_ok = referenced_inputs(&start_templates).iter().all(|id| {
        let optional = manifest
            .inputs
            .iter()
            .find(|i| &i.id == id)
            .is_none_or(|i| i.optional);
        optional || record.inputs.get(id).is_some_and(|p| Path::new(p).exists())
    });

    let Some(prepare) = &manifest.prepare else {
        return if inputs_ok {
            InstanceState::Ready
        } else {
            InstanceState::NotPrepared
        };
    };
    let Some(prepared) = &record.prepared_release else {
        return InstanceState::NotPrepared;
    };
    let done_ok = prepare
        .done
        .iter()
        .all(|file| location.resolve(file).is_ok_and(|p| p.exists()));
    if *prepared != installation.tag || !done_ok || record.inputs_changed {
        InstanceState::NeedsRebuild
    } else if inputs_ok {
        InstanceState::Ready
    } else {
        InstanceState::NotPrepared
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, String> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| format!("{} is damaged: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Can't read {}: {e}", path.display())),
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())? + "\n";
    write_atomically(path, &text)
}
