import { describe, expect, it } from "vitest";
import { games, settingsTabs, tabs, updates } from "./data";

describe("static data", () => {
  it("orders games by first Japanese release year with unique ids", () => {
    const years = games.map((g) => g.year);
    expect(years).toEqual([...years].sort((a, b) => a - b));
    expect(new Set(games.map((g) => g.id)).size).toBe(games.length);
  });

  it("gives every game at least one platform", () => {
    for (const g of games) expect(g.platforms.length).toBeGreaterThan(0);
  });

  it("keeps Mods hidden and About as the last settings tab", () => {
    expect(tabs.map((t) => t.id)).toEqual(["home", "settings"]);
    expect(settingsTabs.at(-1)).toBe("About");
  });

  it("provides the three release-note cards of the Home strip", () => {
    expect(updates).toHaveLength(3);
  });
});
