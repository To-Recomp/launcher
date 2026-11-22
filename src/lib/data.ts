// Static data.

export const LAUNCHER_VERSION = "0.1.0";

/** Where the launcher's own bug reports go. */
export const LAUNCHER_REPOSITORY = "To-Recomp/launcher";

/** "launcher" is the launcher's own settings page, opened from the title bar's gear, not a tab. */
export type TabId = "home" | "settings" | "launcher";
export const tabs: { id: TabId; label: string }[] = [
  { id: "home", label: "Home" },
  { id: "settings", label: "Settings" },
];
// "Mods" is hidden for now (decision Q41).

export const settingsTabs = ["Display", "Graphics", "Audio", "Controls", "Saves", "Game", "About"] as const;
export type SettingsTab = (typeof settingsTabs)[number];

export const NON_AFFILIATION =
  "Not affiliated with or endorsed by Bandai Namco Entertainment, Sony Interactive Entertainment, Microsoft, Nintendo, or any game's publisher or developer.";

/** The launcher's own repository, as a link. */
export const LAUNCHER_URL = `https://github.com/${LAUNCHER_REPOSITORY}`;

/** What the launcher is built with; the complete lists are `Cargo.lock` and `package-lock.json`. */
export const CREDITS = [
  { name: "Tauri", license: "Apache-2.0 or MIT", url: "https://github.com/tauri-apps/tauri" },
  { name: "Svelte", license: "MIT", url: "https://github.com/sveltejs/svelte" },
  { name: "ureq", license: "MIT or Apache-2.0", url: "https://github.com/algesten/ureq" },
  { name: "toml_edit", license: "MIT or Apache-2.0", url: "https://github.com/toml-rs/toml" },
  { name: "SteamGridDB", license: "Game artwork", url: "https://www.steamgriddb.com" },
] as const;
