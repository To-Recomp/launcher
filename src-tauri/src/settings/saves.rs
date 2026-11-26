//! The saves of a game: the folders its manifest lists in `saves`, listed, backed up to one
//! `.tar.gz`, restored from one, and cleaned file by file.
//!
//! A backup keeps each folder under `save<i>/` (`i` is the position in the manifest's `saves`),
//! so it can be restored even where the folders live somewhere else.

use std::fs::{self, File};
use std::io;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::Serialize;

use super::{Instance, Location};

/// Files listed per folder; a save folder is a handful of files, this only bounds a mistake.
const MAX_FILES: usize = 1000;
const PREFIX: &str = "save";

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SavesView {
    pub folders: Vec<SaveFolder>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SaveFolder {
    pub path: String,
    pub exists: bool,
    pub files: Vec<SaveFile>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SaveFile {
    /// Relative to the folder, always with `/`.
    pub name: String,
    pub size: u64,
    /// Seconds since the Unix epoch; `None` when the system doesn't say.
    pub modified: Option<u64>,
}

fn folders(instance: &Instance) -> Result<Vec<PathBuf>, String> {
    let manifest = super::read_manifest(instance)?;
    manifest
        .saves
        .iter()
        .map(|template| instance.resolve(template))
        .collect()
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<SaveFile>) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        if out.len() >= MAX_FILES {
            return Ok(());
        }
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)?;
        if meta.is_dir() {
            walk(root, &path, out)?;
        } else if meta.is_file() {
            let name = path
                .strip_prefix(root)
                .expect("under root")
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            out.push(SaveFile {
                name,
                size: meta.len(),
                modified,
            });
        }
    }
    Ok(())
}

pub fn list(instance: &Instance) -> Result<SavesView, String> {
    let mut view = SavesView { folders: vec![] };
    for folder in folders(instance)? {
        let exists = folder.is_dir();
        let mut files = vec![];
        if exists {
            walk(&folder, &folder, &mut files)
                .map_err(|e| format!("Can't read {}: {e}", folder.display()))?;
        }
        view.folders.push(SaveFolder {
            path: folder.to_string_lossy().into_owned(),
            exists,
            files,
        });
    }
    Ok(view)
}

/// Writes every save file of the game to `target` (a `.tar.gz`). Returns how many files it holds.
pub fn export(instance: &Instance, target: &Path) -> Result<usize, String> {
    let mut count = 0;
    let mut files = vec![];
    for (i, folder) in folders(instance)?.into_iter().enumerate() {
        if !folder.is_dir() {
            continue;
        }
        let mut found = vec![];
        walk(&folder, &folder, &mut found)
            .map_err(|e| format!("Can't read {}: {e}", folder.display()))?;
        files.extend(found.into_iter().map(|f| (i, folder.clone(), f.name)));
    }
    if files.is_empty() {
        return Err("There are no saves to back up yet.".to_string());
    }
    let out = File::create(target).map_err(|e| format!("Can't write {}: {e}", target.display()))?;
    let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
        out,
        flate2::Compression::default(),
    ));
    let fail = |e: io::Error| format!("Can't write {}: {e}", target.display());
    for (i, folder, name) in files {
        tar.append_path_with_name(folder.join(&name), format!("{PREFIX}{i}/{name}"))
            .map_err(fail)?;
        count += 1;
    }
    tar.into_inner().and_then(|gz| gz.finish()).map_err(fail)?;
    Ok(count)
}

/// The save folder an archive entry belongs to and the path inside it; `None` for a folder entry.
fn entry_target(folders: &[PathBuf], path: &Path) -> Result<Option<(usize, PathBuf)>, String> {
    let bad = || format!("The backup has an unexpected path: {}", path.display());
    let mut parts = path.components();
    let Some(Component::Normal(first)) = parts.next() else {
        return Err(bad());
    };
    let index = first
        .to_str()
        .and_then(|s| s.strip_prefix(PREFIX))
        .and_then(|n| n.parse::<usize>().ok())
        .filter(|i| *i < folders.len())
        .ok_or_else(bad)?;
    let mut relative = PathBuf::new();
    for part in parts {
        match part {
            Component::Normal(p) => relative.push(p),
            _ => return Err(bad()),
        }
    }
    Ok((!relative.as_os_str().is_empty()).then_some((index, relative)))
}

/// Puts the files of a backup back, replacing those with the same name and leaving the others.
/// The whole archive is checked first, so a damaged or foreign one changes nothing.
/// Returns how many files were restored.
pub fn import(instance: &Instance, source: &Path) -> Result<usize, String> {
    let folders = folders(instance)?;
    let open = || -> Result<_, String> {
        let file =
            File::open(source).map_err(|e| format!("Can't read {}: {e}", source.display()))?;
        Ok(tar::Archive::new(flate2::read::GzDecoder::new(file)))
    };
    let damaged = |e: io::Error| format!("This isn't a valid backup: {e}");

    let mut archive = open()?;
    let mut count = 0;
    for entry in archive.entries().map_err(damaged)? {
        let entry = entry.map_err(damaged)?;
        let kind = entry.header().entry_type();
        let path = entry.path().map_err(damaged)?;
        if kind.is_dir() {
            entry_target(&folders, &path)?;
        } else if kind.is_file() {
            entry_target(&folders, &path)?
                .ok_or_else(|| format!("The backup has an unexpected path: {}", path.display()))?;
            count += 1;
        } else {
            return Err("The backup contains something other than files.".to_string());
        }
    }
    if count == 0 {
        return Err("This backup has no saves in it.".to_string());
    }

    let mut archive = open()?;
    for entry in archive.entries().map_err(damaged)? {
        let mut entry = entry.map_err(damaged)?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path().map_err(damaged)?.into_owned();
        let (index, relative) = entry_target(&folders, &path)?.expect("checked above");
        let target = folders[index].join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Can't create {}: {e}", parent.display()))?;
        }
        // Written beside the target, then moved: a failure never leaves half a save.
        let partial = target.with_extension("restoring");
        let mut write = |partial: &Path| -> io::Result<()> {
            io::copy(&mut entry, &mut File::create(partial)?)?;
            fs::rename(partial, &target)
        };
        write(&partial).map_err(|e| {
            let _ = fs::remove_file(&partial);
            format!("Can't restore {}: {e}", target.display())
        })?;
    }
    Ok(count)
}

/// Deletes one save file, named as `list` gives it.
pub fn delete(instance: &Instance, folder: usize, name: &str) -> Result<(), String> {
    let folders = folders(instance)?;
    let root = folders
        .get(folder)
        .ok_or_else(|| "That save folder doesn't exist.".to_string())?;
    let mut target = root.clone();
    for part in Path::new(name).components() {
        match part {
            Component::Normal(p) => target.push(p),
            _ => return Err(format!("`{name}` isn't a save file.")),
        }
    }
    if target == *root || !fs::symlink_metadata(&target).is_ok_and(|m| m.is_file()) {
        return Err(format!("`{name}` isn't a save file."));
    }
    fs::remove_file(&target).map_err(|e| format!("Can't delete {}: {e}", target.display()))
}

#[tauri::command(async)]
pub fn list_saves(location: Location) -> Result<SavesView, String> {
    list(&location.into())
}

#[tauri::command(async)]
pub fn export_saves(location: Location, file: String) -> Result<usize, String> {
    export(&location.into(), Path::new(&file))
}

#[tauri::command(async)]
pub fn import_saves(location: Location, file: String) -> Result<usize, String> {
    import(&location.into(), Path::new(&file))
}

#[tauri::command(async)]
pub fn delete_save(location: Location, folder: usize, name: String) -> Result<(), String> {
    delete(&location.into(), folder, &name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn instance(saves: &str) -> (TempDir, Instance) {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("recomplauncher.json"),
            format!(r#"{{ "saves": {saves} }}"#),
        )
        .unwrap();
        let instance = Instance::new(dir.path().to_path_buf(), Some(dir.path().join("data")));
        (dir, instance)
    }

    fn put(path: PathBuf, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn lists_files_of_every_folder_and_reports_a_missing_one() {
        let (_dir, instance) = instance(r#"["{data}/saves", "{data}/other"]"#);
        put(instance.data.join("saves/b.raw"), "bb");
        put(instance.data.join("saves/sub/a.raw"), "a");
        let view = list(&instance).unwrap();
        assert_eq!(view.folders.len(), 2);
        assert!(view.folders[0].exists);
        let names: Vec<_> = view.folders[0]
            .files
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, ["b.raw", "sub/a.raw"]);
        assert_eq!(view.folders[0].files[0].size, 2);
        assert!(!view.folders[1].exists && view.folders[1].files.is_empty());
    }

    #[test]
    fn a_backup_restores_what_was_deleted_and_keeps_newer_files() {
        let (dir, instance) = instance(r#"["{data}/saves"]"#);
        put(instance.data.join("saves/card.raw"), "old");
        put(instance.data.join("saves/sub/slot.raw"), "slot");
        let backup = dir.path().join("backup.tar.gz");
        assert_eq!(export(&instance, &backup).unwrap(), 2);

        fs::remove_file(instance.data.join("saves/sub/slot.raw")).unwrap();
        put(instance.data.join("saves/card.raw"), "changed");
        put(instance.data.join("saves/new.raw"), "new");
        assert_eq!(import(&instance, &backup).unwrap(), 2);

        let read = |p: &str| fs::read_to_string(instance.data.join(p)).unwrap();
        assert_eq!(read("saves/card.raw"), "old");
        assert_eq!(read("saves/sub/slot.raw"), "slot");
        assert_eq!(read("saves/new.raw"), "new");
    }

    #[test]
    fn nothing_to_back_up_is_an_error_and_writes_no_file() {
        let (dir, instance) = instance(r#"["{data}/saves"]"#);
        let backup = dir.path().join("backup.tar.gz");
        assert!(export(&instance, &backup).is_err());
        assert!(!backup.exists());
    }

    fn archive(dir: &Path, entries: &[(&str, tar::EntryType)]) -> PathBuf {
        let path = dir.join("bad.tar.gz");
        let gz = flate2::write::GzEncoder::new(
            File::create(&path).unwrap(),
            flate2::Compression::default(),
        );
        let mut builder = tar::Builder::new(gz);
        for (name, kind) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(*kind);
            header.set_size(if kind.is_file() { 1 } else { 0 });
            if kind.is_symlink() {
                header.set_link_name("/etc/passwd").unwrap();
            }
            // Written raw: `set_path` itself refuses `..`.
            let raw = header.as_old_mut();
            raw.name[..name.len()].copy_from_slice(name.as_bytes());
            header.set_cksum();
            builder
                .append(&header, if kind.is_file() { &b"x"[..] } else { &b""[..] })
                .unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap();
        path
    }

    #[test]
    fn a_backup_with_a_foreign_path_or_a_link_changes_nothing() {
        let (dir, instance) = instance(r#"["{data}/saves"]"#);
        for entries in [
            vec![
                ("save0/ok.raw", tar::EntryType::Regular),
                ("save0/../../evil", tar::EntryType::Regular),
            ],
            vec![
                ("save0/ok.raw", tar::EntryType::Regular),
                ("save7/x.raw", tar::EntryType::Regular),
            ],
            vec![
                ("save0/ok.raw", tar::EntryType::Regular),
                ("save0/link", tar::EntryType::Symlink),
            ],
            vec![("elsewhere/x.raw", tar::EntryType::Regular)],
        ] {
            let bad = archive(dir.path(), &entries);
            assert!(import(&instance, &bad).is_err(), "{entries:?}");
            assert!(!instance.data.join("saves").exists(), "{entries:?}");
            assert!(!dir.path().join("evil").exists());
        }
        let not_a_backup = dir.path().join("x.tar.gz");
        fs::write(&not_a_backup, "plain text").unwrap();
        assert!(import(&instance, &not_a_backup).is_err());
    }

    #[test]
    fn deleting_removes_one_file_and_refuses_paths_leaving_the_folder() {
        let (dir, instance) = instance(r#"["{data}/saves"]"#);
        put(instance.data.join("saves/a.raw"), "a");
        put(instance.data.join("saves/b.raw"), "b");
        put(dir.path().join("outside.txt"), "o");
        delete(&instance, 0, "a.raw").unwrap();
        assert!(!instance.data.join("saves/a.raw").exists());
        assert!(instance.data.join("saves/b.raw").exists());
        for bad in ["../../outside.txt", "/etc/passwd", "", ".", "missing.raw"] {
            assert!(delete(&instance, 0, bad).is_err(), "{bad}");
        }
        assert!(delete(&instance, 3, "b.raw").is_err());
        assert!(dir.path().join("outside.txt").exists());
    }
}
