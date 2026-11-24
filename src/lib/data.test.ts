import { describe, expect, it } from "vitest";
import { settingsTabs } from "./data";

describe("static data", () => {
  it("keeps About as the last settings tab", () => {
    expect(settingsTabs.at(-1)).toBe("About");
  });
});
