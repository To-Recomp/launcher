import { describe, expect, it } from "vitest";
import { releaseDate, releaseSummary } from "./releases";

describe("releaseSummary", () => {
  it("flattens markdown to a short plain line", () => {
    expect(releaseSummary("## New\n- **Faster** startup, see [notes](http://x)\n\n```\ncode\n```")).toBe("New Faster startup, see notes");
  });
  it("cuts long text with an ellipsis", () => {
    const out = releaseSummary("word ".repeat(100), 20);
    expect(out.length).toBeLessThanOrEqual(20);
    expect(out.endsWith("…")).toBe(true);
  });
});

describe("releaseDate", () => {
  it("formats an ISO date and tolerates a missing one", () => {
    expect(releaseDate("2026-11-21T23:30:00Z")).toBe("Nov 21, 2026");
    expect(releaseDate(null)).toBe("");
    expect(releaseDate("nope")).toBe("");
  });
});
