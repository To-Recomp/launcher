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
