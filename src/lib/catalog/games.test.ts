import { describe, expect, it } from "vitest";
import type { CatalogPlatform } from "./types";
import { defaultPlatform, initials, quickPlayPlatform, toGames } from "./games";

describe("initials", () => {
  it.each([
    ["Tales of Symphonia", "S"],
    ["Tales of Destiny 2", "D2"],
    ["Tales of the Abyss", "A"],
    ["tales of vesperia", "V"],
    ["Symphonia", "S"],
    ["Éternia", "É"],
    ["", "?"],
  ])("%s -> %s", (name, expected) => {
    expect(initials(name)).toBe(expected);
  });
});

describe("toGames", () => {
  it("keeps the catalog's order and lists one entry per console", () => {
    const games = toGames({
      games: [
        {
          id: "tos", name: "Tales of Symphonia", japanReleaseYear: 2003, repository: "To-Recomp/tos-recomp",
          platforms: [{ platform: "GameCube", regions: ["France (PAL)", "Japan"], state: "playable", maturity: "beta", releaseYear: null }],
        },
        { id: "tov", name: "Tales of Vesperia", japanReleaseYear: 2008, repository: "To-Recomp/tov", platforms: [] },
      ],
    });
    expect(games.map((g) => g.id)).toEqual(["tos", "tov"]);
    expect(games[0]).toMatchObject({ icon: "S", year: 2003, state: "not-installed" });
    expect(games[0].platforms).toEqual([
      { platform: "GameCube", regions: ["France (PAL)", "Japan"], state: "playable", maturity: "beta", releaseYear: null, year: 2003 },
    ]);
  });

  it("orders consoles from the most recent to the oldest, with the game's own year where the catalog has none", () => {
    const platform = (name: string, releaseYear: number | null) => ({ platform: name, regions: ["Japan"], state: "planned" as const, maturity: null, releaseYear });
    const [game] = toGames({
      games: [
        {
          id: "tos", name: "Tales of Symphonia", japanReleaseYear: 2003, repository: "To-Recomp/tos",
          platforms: [platform("PS2", 2004), platform("PS3", 2013), platform("GameCube", null), platform("Wii", 2013)],
        },
      ],
    });
    expect(game.platforms.map((p) => [p.platform, p.year])).toEqual([["PS3", 2013], ["Wii", 2013], ["PS2", 2004], ["GameCube", 2003]]);
  });
});

describe("defaultPlatform", () => {
  const platform = (name: string, state: "planned" | "playable", year: number): CatalogPlatform => ({
    platform: name, regions: ["Japan"], state, maturity: state === "playable" ? "beta" : null, releaseYear: year,
  });
  const game = (...platforms: CatalogPlatform[]) =>
    toGames({ games: [{ id: "tos", name: "Tales of Symphonia", japanReleaseYear: 2003, repository: "To-Recomp/tos", platforms }] })[0];

  it("is the first playable console, even when a more recent one is only planned", () => {
    expect(defaultPlatform(game(platform("Switch", "planned", 2023), platform("GameCube", "playable", 2003)))?.platform).toBe("GameCube");
  });

  it("is the most recent playable console when several are", () => {
    expect(defaultPlatform(game(platform("GameCube", "playable", 2003), platform("PS3", "playable", 2013)))?.platform).toBe("PS3");
  });

  it("is the most recent console when none is playable", () => {
    expect(defaultPlatform(game(platform("PS2", "planned", 2004), platform("Switch", "planned", 2023)))?.platform).toBe("Switch");
  });

  it("is the console the game was last played on, while it is still playable", () => {
    const g = game(platform("PS3", "playable", 2013), platform("GameCube", "playable", 2003), platform("Switch", "planned", 2023));
    expect(defaultPlatform(g, "GameCube")?.platform).toBe("GameCube");
    expect(defaultPlatform(g, "Switch")?.platform).toBe("PS3"); // no longer playable there, or never was
    expect(defaultPlatform(g, "Wii")?.platform).toBe("PS3"); // gone from the catalog
  });
});

describe("quickPlayPlatform", () => {
  const platform = (name: string, year: number): CatalogPlatform => ({ platform: name, regions: ["Japan"], state: "playable", maturity: "beta", releaseYear: year });
  const [game] = toGames({
    games: [{ id: "tos", name: "Tales of Symphonia", japanReleaseYear: 2003, repository: "To-Recomp/tos", platforms: [platform("GameCube", 2003), platform("PS3", 2013), platform("Wii", 2008)] }],
  });
  const ready = (...names: string[]) => (p: { platform: string }) => names.includes(p.platform);

  it("is the console last played on when it is ready", () => {
    expect(quickPlayPlatform(game, ready("GameCube", "PS3"), "GameCube")?.platform).toBe("GameCube");
  });

  it("is the most recent ready console when the last played one isn't ready, or nothing was played", () => {
    expect(quickPlayPlatform(game, ready("GameCube", "Wii"), "PS3")?.platform).toBe("Wii");
    expect(quickPlayPlatform(game, ready("GameCube", "Wii"), null)?.platform).toBe("Wii");
  });

  it("is nothing when no console is ready", () => {
    expect(quickPlayPlatform(game, ready(), "GameCube")).toBeUndefined();
  });
});
