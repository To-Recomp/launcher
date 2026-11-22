import { describe, expect, it } from "vitest";
import { settingsTabs, tabs } from "./data";

describe("static data", () => {
  it("keeps Mods hidden and About as the last settings tab", () => {
    expect(tabs.map((t) => t.id)).toEqual(["home", "settings"]);
    expect(settingsTabs.at(-1)).toBe("About");
  });
});
