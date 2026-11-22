// Static data.

import { t, type MessageKey } from "./i18n/index.svelte";

export const LAUNCHER_VERSION = "0.1.0";

/** Where the launcher's own bug reports go. */
export const LAUNCHER_REPOSITORY = "To-Recomp/launcher";

/** "launcher" is the launcher's own settings page, opened from the title bar's gear, not a tab. */
export type TabId = "home" | "settings" | "launcher";
export const tabs: { id: TabId; label: MessageKey }[] = [
  { id: "home", label: "shell.tabs.home" },
  { id: "settings", label: "shell.tabs.settings" },
];
// "Mods" is hidden for now (decision Q41).

export const settingsTabs = ["Display", "Graphics", "Audio", "Controls", "Saves", "Game", "About"] as const;
export type SettingsTab = (typeof settingsTabs)[number];

/** The disclaimer shown in the footer and the About page. */
export const nonAffiliation = () => t("shell.nonAffiliation");

/** The launcher's own repository, as a link. */
export const LAUNCHER_URL = `https://github.com/${LAUNCHER_REPOSITORY}`;

/** What the launcher is built with; the complete lists are `Cargo.lock` and `package-lock.json`. */
export interface Credit {
  name: string;
  url: string;
  /** A license name, which is data and stays as is. */
  license: string;
}
export const CREDITS: readonly Credit[] = [
  { name: "Tauri", license: "Apache-2.0 or MIT", url: "https://github.com/tauri-apps/tauri" },
  { name: "Svelte", license: "MIT", url: "https://github.com/sveltejs/svelte" },
  { name: "ureq", license: "MIT or Apache-2.0", url: "https://github.com/algesten/ureq" },
  { name: "toml_edit", license: "MIT or Apache-2.0", url: "https://github.com/toml-rs/toml" },
];
