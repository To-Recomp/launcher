//! Small filesystem helpers shared by the backend modules.

use std::fs;
use std::path::Path;

/// Writes next to the target then renames, so a crash never leaves half a configuration file.
pub fn write_atomically(path: &Path, contents: &str) -> Result<(), String> {
    let fail = |e: std::io::Error| format!("Can't write {}: {e}", path.display());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(fail)?;
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    fs::write(&tmp, contents).map_err(fail)?;
    fs::rename(&tmp, path).map_err(fail)
}
