import type { Catalog, CatalogPlatform } from "./types";

export type GameState = "not-installed" | "installed" | "ready";

/**
 * One console of a game: the selector lists these, and an instance is one of them. `year` is when
 * the game came out on it (the game's first Japanese release when the catalog doesn't say).
 */
export type Platform = CatalogPlatform & { year: number };

export interface Game {
  id: string;
  /** Letters shown in place of artwork. */
  icon: string;
  name: string;
  /** `owner/name` of the game's recomp repository, where its releases are. */
  repository: string;
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

/**
 * The console a game opens on: the one it was last played on (if it is still playable), else the
 * first playable one, else the most recent (the list is ordered that way).
 */
export function defaultPlatform(game: Game, lastPlayed: string | null = null): Platform | undefined {
  const playable = game.platforms.filter((p) => p.state === "playable");
  return playable.find((p) => p.platform === lastPlayed) ?? playable[0] ?? game.platforms[0];
}

export function toGames(catalog: Catalog): Game[] {
  return catalog.games.map((g) => ({
    id: g.id,
    icon: initials(g.name),
    name: g.name,
    repository: g.repository,
    year: g.japanReleaseYear,
    state: "not-installed",
    // Most recent first; the catalog's order breaks ties.
    platforms: g.platforms
      .map((p): Platform => ({ ...p, year: p.releaseYear ?? g.japanReleaseYear }))
      .sort((a, b) => b.year - a.year),
  }));
}
