import { describe, expect, it } from "vitest";
import { version } from "../../package.json";
import { LAUNCHER_VERSION, settingsTabs } from "./data";

describe("static data", () => {
  it("keeps About as the last settings tab", () => {
    expect(settingsTabs.at(-1)).toBe("About");
  });

  it("shows the version of package.json, release candidate suffix included", () => {
    expect(LAUNCHER_VERSION).toBe(version);
  });
});
