// Static data.

export const LAUNCHER_VERSION = "0.1.0";

/** Where the launcher's own bug reports go. */
export const LAUNCHER_REPOSITORY = "To-Recomp/launcher";

export type TabId = "home" | "settings";
export const tabs: { id: TabId; label: string }[] = [
  { id: "home", label: "Home" },
  { id: "settings", label: "Settings" },
];
// "Mods" is hidden for now (decision Q41).

export const settingsTabs = ["Display", "Graphics", "Audio", "Controls", "Saves", "Game", "About"] as const;
export type SettingsTab = (typeof settingsTabs)[number];

export const NON_AFFILIATION =
  "Not affiliated with or endorsed by Bandai Namco Entertainment, Sony Interactive Entertainment, Microsoft, Nintendo, or any game's publisher or developer.";
