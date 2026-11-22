// Static placeholder data: the release notes strip is replaced when releases are read (#3).

export interface UpdateNote {
  date: string;
  version: string;
  title: string;
  body: string;
}

export const LAUNCHER_VERSION = "0.1.0";

export const updates: UpdateNote[] = [
  { date: "Nov 21, 2026", version: "v1.2.0", title: "Library improvements", body: "Improved startup times and refined library navigation." },
  { date: "Nov 20, 2026", version: "v1.1.2", title: "Bug fixes", body: "Fixed folder detection issues and resolved a crash when reopening the launcher." },
  { date: "Nov 19, 2026", version: "v1.1.0", title: "UI polish", body: "Refined menu spacing, updated icons, and improved readability across the launcher." },
];

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
