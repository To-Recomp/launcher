import { invoke } from "@tauri-apps/api/core";
import type { SettingInput, SettingsView } from "../settings/types";

/** The launcher's own settings (src-tauri/src/launcher_settings), not a game's. */
export const loadLauncherSettings = () => invoke<SettingsView>("load_launcher_settings");

/** Validates and writes one change; resolves with the refreshed view, rejects with a message. */
export const setLauncherSetting = (id: string, input: SettingInput) =>
  invoke<SettingsView>("set_launcher_setting", { id, input });
