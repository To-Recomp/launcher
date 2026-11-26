import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { Location } from "./types";

// Mirrors src-tauri/src/settings/saves.rs (serde camelCase).

export interface SaveFile {
  /** Relative to its folder, with `/`. */
  name: string;
  size: number;
  /** Seconds since the Unix epoch. */
  modified: number | null;
}

export interface SaveFolder {
  path: string;
  exists: boolean;
  files: SaveFile[];
}

export interface SavesView {
  folders: SaveFolder[];
}

export const listSaves = (location: Location) => invoke<SavesView>("list_saves", { location });

/** Writes the saves to `file` (a `.tar.gz`); resolves with how many files it holds. */
export const exportSaves = (location: Location, file: string) => invoke<number>("export_saves", { location, file });

/** Puts the files of a backup back; resolves with how many were restored. */
export const importSaves = (location: Location, file: string) => invoke<number>("import_saves", { location, file });

export const deleteSave = (location: Location, folder: number, name: string) =>
  invoke<void>("delete_save", { location, folder, name });

const FILTERS = [{ name: "tar.gz", extensions: ["gz", "tgz"] }];

/** Asks where to write a backup; `null` when the player closes the dialog. */
export async function pickBackupTarget(title: string, name: string): Promise<string | null> {
  return save({ title, defaultPath: `${name}.tar.gz`, filters: FILTERS });
}

/** Asks for a backup to restore; `null` when the player closes the dialog. */
export async function pickBackup(title: string): Promise<string | null> {
  const picked = await open({ title, multiple: false, directory: false, filters: FILTERS });
  return typeof picked === "string" ? picked : null;
}
