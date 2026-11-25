// Static data.

import { t } from "./i18n/index.svelte";

export const LAUNCHER_VERSION = "0.1.0";

/** Where the launcher's own bug reports go. */
export const LAUNCHER_REPOSITORY = "To-Recomp/launcher";

/** What the main area shows: the game's page, its settings, or the launcher's own settings. */
export type View = "game" | "settings" | "launcher";

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
  { name: "Lucide", license: "ISC", url: "https://github.com/lucide-icons/lucide" },
];
