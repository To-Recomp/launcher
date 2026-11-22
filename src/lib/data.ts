// Static placeholder data; replaced by the catalog (#2) and installations (#3).

export type GameState = "not-installed" | "installed" | "ready";

export interface Game {
  id: string;
  icon: string;
  name: string;
  year: number;
  state: GameState;
  platforms: string[];
}

export interface UpdateNote {
  date: string;
  version: string;
  title: string;
  body: string;
}

export const LAUNCHER_VERSION = "0.1.0";

export const games: Game[] = [
  { id: "phantasia", icon: "P", name: "Tales of Phantasia", year: 1995, state: "installed", platforms: ["SFC"] },
  { id: "destiny", icon: "D", name: "Tales of Destiny", year: 1997, state: "not-installed", platforms: ["PS1"] },
  { id: "eternia", icon: "E", name: "Tales of Eternia", year: 2000, state: "ready", platforms: ["PS1", "PSP"] },
  { id: "destiny2", icon: "D2", name: "Tales of Destiny 2", year: 2002, state: "not-installed", platforms: ["PS2"] },
  { id: "symphonia", icon: "S", name: "Tales of Symphonia", year: 2003, state: "not-installed", platforms: ["GameCube"] },
  { id: "rebirth", icon: "R", name: "Tales of Rebirth", year: 2004, state: "not-installed", platforms: ["PS2"] },
  { id: "legendia", icon: "L", name: "Tales of Legendia", year: 2005, state: "not-installed", platforms: ["PS2"] },
  { id: "abyss", icon: "A", name: "Tales of the Abyss", year: 2005, state: "installed", platforms: ["PS2"] },
  { id: "innocence", icon: "I", name: "Tales of Innocence", year: 2007, state: "not-installed", platforms: ["DS"] },
  { id: "vesperia", icon: "V", name: "Tales of Vesperia", year: 2008, state: "not-installed", platforms: ["Xbox 360"] },
  { id: "hearts", icon: "H", name: "Tales of Hearts", year: 2008, state: "not-installed", platforms: ["DS"] },
  { id: "graces", icon: "G", name: "Tales of Graces", year: 2009, state: "not-installed", platforms: ["Wii"] },
];

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
