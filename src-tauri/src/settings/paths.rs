//! The path placeholders of a manifest (`{install}`, `{data}`, ...) and config file lookup.

use std::path::PathBuf;

use super::manifest::{ConfigFile, FileSpec, Format};

/// Where one installed game lives.
#[derive(Debug, Clone)]
pub struct Instance {
    pub install: PathBuf,
    pub data: PathBuf,
}

impl Instance {
    pub fn new(install: PathBuf, data: Option<PathBuf>) -> Instance {
        let data = data.unwrap_or_else(|| install.join("recompdata"));
        Instance { install, data }
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.install.join("recomplauncher.json")
    }

    pub fn state_path(&self) -> PathBuf {
        self.data.join("launcher-settings.json")
    }

    pub fn resolve(&self, template: &str) -> Result<PathBuf, String> {
        self.expand_text(template).map(PathBuf::from)
    }

    /// Fills in the placeholders of a path, an argument or a value.
    pub fn expand_text(&self, template: &str) -> Result<String, String> {
        let known = [
            ("{install}", Some(self.install.clone())),
            ("{data}", Some(self.data.clone())),
            ("{home}", dirs::home_dir()),
            ("{documents}", dirs::document_dir()),
            ("{local}", dirs::data_local_dir()),
            ("{config}", dirs::config_dir()),
        ];
        let mut out = template.to_string();
        for (placeholder, dir) in known {
            if out.contains(placeholder) {
                let dir =
                    dir.ok_or_else(|| format!("{placeholder} isn't available on this system"))?;
                out = out.replace(placeholder, &dir.to_string_lossy());
            }
        }
        if let Some(start) = out.find('{') {
            let rest = &out[start..];
            let end = rest.find('}').map_or(rest.len(), |i| i + 1);
            return Err(format!(
                "`{}` can't be used in a settings file path",
                &rest[..end]
            ));
        }
        Ok(out)
    }

    /// The file of a config entry (first candidate whose `if` exists, else the unconditional
    /// one) and its format (declared, or from the extension).
    pub fn config_location(&self, config: &ConfigFile) -> Result<(PathBuf, Format), String> {
        let path = match &config.file {
            FileSpec::One(path) => self.resolve(path)?,
            FileSpec::Candidates(candidates) => {
                let mut chosen = None;
                for candidate in candidates {
                    let applies = match &candidate.condition {
                        Some(condition) => self.resolve(condition)?.exists(),
                        None => true,
                    };
                    if applies {
                        chosen = Some(self.resolve(&candidate.path)?);
                        break;
                    }
                }
                chosen
                    .ok_or_else(|| format!("config `{}`: no candidate file applies", config.id))?
            }
        };
        let format = config.format.or_else(|| match path.extension()?.to_str()? {
            "toml" => Some(Format::Toml),
            "json" => Some(Format::Json),
            _ => None,
        });
        let format =
            format.ok_or_else(|| format!("config `{}`: set `format` (toml or json)", config.id))?;
        Ok((path, format))
    }
}
