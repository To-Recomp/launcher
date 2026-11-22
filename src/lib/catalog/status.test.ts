import { describe, expect, it } from "vitest";
import { describeCatalog } from "./status";

const at = new Date(2026, 10, 22, 14, 32);

describe("describeCatalog", () => {
  it("says a refresh is under way, whatever happened before", () => {
    expect(describeCatalog({ at, updated: true, error: null }, true)).toBe("Refreshing…");
  });

  it("tells a first run from a finished refresh", () => {
    expect(describeCatalog(null, false)).toBe("Not refreshed yet");
    expect(describeCatalog({ at, updated: true, error: null }, false)).toMatch(/^Updated at \d{1,2}:\d{2}/);
    expect(describeCatalog({ at, updated: false, error: null }, false)).toMatch(/^Up to date as of \d{1,2}:\d{2}/);
  });

  it("says a failed refresh kept the saved list", () => {
    expect(describeCatalog({ at, updated: false, error: "offline" }, false)).toMatch(
      /^Couldn't refresh at \d{1,2}:\d{2}.*\. The saved list is used\.$/,
    );
  });
});
