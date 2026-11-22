import { invoke } from "@tauri-apps/api/core";

/** Whether this installation can update itself (see src-tauri/src/updater.rs). */
export async function updaterAvailable(): Promise<boolean> {
  try {
    return await invoke<boolean>("updater_available");
  } catch {
    return false; // plain browser, or no Tauri runtime
  }
}

/** Installs a newer release and restarts into it; resolves when there is nothing newer. */
export const updateLauncher = () => invoke<void>("update_launcher");

/** A newer release, as the launcher tells the player about it. */
export interface UpdateInfo {
  version: string;
}

/** Looks for a newer release on the chosen channel without installing it; `null` when there is none. */
export const checkLauncherUpdate = () => invoke<UpdateInfo | null>("check_launcher_update");

/** What the launcher settings say about update checks (src-tauri/src/launcher_settings). */
export interface UpdatePreferences {
  /** Check when the launcher starts. */
  auto: boolean;
  /** Also look at pre-releases. */
  prerelease: boolean;
}

export const updatePreferences = () => invoke<UpdatePreferences>("launcher_update_preferences");
