import { describe, expect, it } from "vitest";
import { releaseDate, releaseSummary } from "./releases";

describe("releaseSummary", () => {
  it("flattens markdown to a short plain line", () => {
    expect(releaseSummary("## New\n- **Faster** startup, see [notes](http://x)\n\n```\ncode\n```")).toBe("New · Faster startup, see notes");
  });
  it("drops what GitHub generates around the entries", () => {
    const body = [
      "## What's Changed",
      "* chore: add Dependabot, CODEOWNERS and PR template by @haegemonia76 in https://github.com/To-Recomp/tos-recomp/pull/5",
      "* docs: disclose AI assistance by @dependabot[bot] in https://github.com/To-Recomp/tos-recomp/pull/6",
      "",
      "**Full Changelog**: https://github.com/To-Recomp/tos-recomp/commits/v0.1.0",
    ].join("\n");
    expect(releaseSummary(body)).toBe("chore: add Dependabot, CODEOWNERS and PR template · docs: disclose AI assistance");
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
