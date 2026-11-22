import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { SettingInput, SettingsView } from "../settings/types";

/** The launcher's own settings (src-tauri/src/launcher_settings), not a game's. */
export const loadLauncherSettings = () => invoke<SettingsView>("load_launcher_settings");

/** Validates and writes one change; resolves with the refreshed view, rejects with a message. */
export const setLauncherSetting = (id: string, input: SettingInput) =>
  invoke<SettingsView>("set_launcher_setting", { id, input });

/** Closes and starts the launcher again, for the settings that apply on the next start. */
export const restartLauncher = () => invoke<void>("restart_launcher");

/** Asks the player for a folder; `null` when they close the dialog. */
export async function pickFolder(title: string): Promise<string | null> {
  const picked = await open({ title, multiple: false, directory: true });
  return typeof picked === "string" ? picked : null;
}

/** Where the games are kept (src-tauri/src/launcher_settings/folders.rs). */
export interface Folders {
  /** The player's choice; `null` while the default is used. */
  configured: string | null;
  default: string;
  /** What this session uses; `null` when it couldn't be opened. */
  inUse: string | null;
}

export const launcherFolders = () => invoke<Folders>("launcher_folders");

/** Chooses the games folder (`null`: the default); it applies on the next start. */
export const setGamesFolder = (folder: string | null) => invoke<Folders>("set_games_folder", { folder });

/** The language the player chose for the interface; `null` to follow the system's. */
export const launcherLanguage = () => invoke<string | null>("launcher_language");
