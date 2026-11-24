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

/** The tray menu's texts, in the interface language (the backend has no translations). */
export interface TrayLabels {
  show: string;
  quit: string;
}

/** Puts the notification area icon on or off as the settings say. */
export const applyBehavior = (labels: TrayLabels) => invoke<void>("apply_behavior", { labels });

/** Whether the launcher can be started with the system on this OS. */
export const autostartSupported = () => invoke<boolean>("autostart_supported");

export const autostartEnabled = () => invoke<boolean>("autostart_enabled");

/** Resolves with what the system says afterwards. */
export const setAutostart = (enabled: boolean) => invoke<boolean>("set_autostart", { enabled });

/** How the interface looks (src-tauri/src/launcher_settings). */
export interface Appearance {
  theme: "dark" | "light" | "system";
}

export const launcherAppearance = () => invoke<Appearance>("launcher_appearance");

/** The proxy every request of the launcher goes through (src-tauri/src/net). */
export interface ProxySettings {
  mode: "system" | "none" | "manual";
  protocol: "http" | "https" | "socks5";
  host: string;
  port: number;
  username: string;
  /** Hosts that bypass the proxy, separated by commas or spaces. */
  noProxy: string;
}

export interface ProxyView {
  settings: ProxySettings;
  /** The password is in the system keychain (it is never in a file). */
  hasPassword: boolean;
  /** Why the keychain can't be used, if it can't. */
  keychainError: string | null;
}

export const proxySettings = () => invoke<ProxyView>("proxy_settings");

/** `password`: `null` keeps the saved one, `""` forgets it. Rejects with why nothing was saved. */
export const setProxy = (settings: ProxySettings, password: string | null) =>
  invoke<ProxyView>("set_proxy", { settings, password });

/** Tries these values (not saved) against GitHub; resolves with what worked, rejects with what didn't. */
export const testProxy = (settings: ProxySettings, password: string | null) =>
  invoke<string>("test_proxy", { settings, password });

/** Makes the saved download speed limit the one in use, running downloads included. */
export const applyDownloadLimit = () => invoke<void>("apply_download_limit_now");
