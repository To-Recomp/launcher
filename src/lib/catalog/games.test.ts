import { describe, expect, it } from "vitest";
import { initials, toGames } from "./games";

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
          platforms: [{ platform: "GameCube", regions: ["France (PAL)", "Japan"], state: "playable", maturity: "beta" }],
        },
        { id: "tov", name: "Tales of Vesperia", japanReleaseYear: 2008, repository: "To-Recomp/tov", platforms: [] },
      ],
    });
    expect(games.map((g) => g.id)).toEqual(["tos", "tov"]);
    expect(games[0]).toMatchObject({ icon: "S", year: 2003, state: "not-installed" });
    expect(games[0].platforms).toEqual([
      { platform: "GameCube", regions: ["France (PAL)", "Japan"], state: "playable", maturity: "beta" },
    ]);
  });
});
