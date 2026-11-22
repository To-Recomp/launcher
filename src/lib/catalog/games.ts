import type { Catalog, CatalogPlatform } from "./types";

export type GameState = "not-installed" | "installed" | "ready";

export interface Platform extends CatalogPlatform {
  /** What the platform selector shows: the platform and the region it was tested for. */
  label: string;
}

export interface Game {
  id: string;
  /** Letters shown in place of artwork. */
  icon: string;
  name: string;
  year: number;
  /** Installations arrive with #3; until then no game is installed. */
  state: GameState;
  platforms: Platform[];
}

/** "Tales of Symphonia" -> "S", "Tales of Destiny 2" -> "D2", "Tales of the Abyss" -> "A". */
export function initials(name: string): string {
  const words = name.replace(/^Tales of (the )?/i, "").trim().split(/\s+/);
  const first = Array.from(words[0] ?? "")[0] ?? "?";
  const last = words.length > 1 ? words[words.length - 1] : "";
  return (first + (/^\d+$/.test(last) ? last : "")).toUpperCase();
}

export function toGames(catalog: Catalog): Game[] {
  return catalog.games.map((g) => ({
    id: g.id,
    icon: initials(g.name),
    name: g.name,
    year: g.japanReleaseYear,
    state: "not-installed",
    platforms: g.platforms.map((p) => ({ ...p, label: `${p.platform} · ${p.region}` })),
  }));
}
