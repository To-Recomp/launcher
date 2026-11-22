import { invoke } from "@tauri-apps/api/core";
import type { Location, SettingInput, SettingsView } from "./types";

export const loadSettings = (location: Location) => invoke<SettingsView>("load_settings", { location });

/** Validates and writes one change; resolves with the refreshed view, rejects with a message. */
export const setSetting = (location: Location, id: string, input: SettingInput) =>
  invoke<SettingsView>("set_setting", { location, id, input });

/** Commands reject with the backend's message string. */
export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}
