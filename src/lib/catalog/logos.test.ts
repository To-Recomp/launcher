import { describe, expect, it } from "vitest";
import { logoFor, logoKey } from "./logos";

describe("logoKey", () => {
  it.each([
    ["Tales of Symphonia", "symphonia"],
    ["Tales of the Abyss", "abyss"],
    ["Tales of Destiny 2", "destiny2"],
    ["Tales of Xillia  2", "xillia2"],
  ])("%s -> %s", (name, key) => expect(logoKey(name)).toBe(key));
});

describe("logoFor", () => {
  it("finds the bundled logo, with or without the article and the number", () => {
    for (const name of ["Tales of Symphonia", "Tales of the Abyss", "Tales of Destiny 2", "Tales of Xillia 2"]) {
      expect(logoFor(name), name).toMatch(/\.png/);
    }
  });

  it("returns nothing for a game without a logo", () => {
    expect(logoFor("Tales of Unknown")).toBeUndefined();
  });
});
