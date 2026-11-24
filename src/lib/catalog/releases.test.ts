import { afterEach, describe, expect, it, vi } from "vitest";

const tauri = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => tauri);

import { cachedReleases, gameReleases, releaseDate, releaseEntries, type ReleaseNote } from "./releases";

describe("releaseEntries", () => {
  it("makes one entry per list item, without headings, links or code", () => {
    const body = "## New\n- **Faster** startup, see [notes](http://x)\n\n```\ncode\n```\n- plain line";
    expect(releaseEntries(body)).toEqual({ entries: ["Faster startup, see notes", "plain line"], more: 0 });
  });
  it("drops what GitHub generates around the entries", () => {
    const body = [
      "## What's Changed",
      "* chore: add Dependabot, CODEOWNERS and PR template by @haegemonia76 in https://github.com/To-Recomp/tos-recomp/pull/5",
      "* docs: disclose AI assistance by @dependabot[bot] in https://github.com/To-Recomp/tos-recomp/pull/6",
      "",
      "**Full Changelog**: https://github.com/To-Recomp/tos-recomp/commits/v0.1.0",
    ].join("\n");
    expect(releaseEntries(body).entries).toEqual(["chore: add Dependabot, CODEOWNERS and PR template", "docs: disclose AI assistance"]);
  });
  it("keeps the first entries and counts the rest", () => {
    const body = Array.from({ length: 7 }, (_, i) => `- fix: item ${i + 1}`).join("\n");
    const { entries, more } = releaseEntries(body, 3);
    expect(entries).toEqual(["fix: item 1", "fix: item 2", "fix: item 3"]);
    expect(more).toBe(4);
    expect(releaseEntries("").entries).toEqual([]);
  });
});

describe("releaseDate", () => {
  it("formats an ISO date and tolerates a missing one", () => {
    expect(releaseDate("2026-11-21T10:00:00Z")).toBe("Nov 21, 2026");
    expect(releaseDate(null)).toBe("");
    expect(releaseDate("nope")).toBe("");
  });
});

describe("gameReleases", () => {
  const note = (tag: string): ReleaseNote => ({ tag, name: null, publishedAt: null, body: "", prerelease: false });
  afterEach(() => vi.useRealTimers());

  it("reads a repository once, then serves it (at once, too) until 15 minutes passed", async () => {
    vi.useFakeTimers();
    tauri.invoke.mockResolvedValue([note("v1")]);
    expect(cachedReleases("To-Recomp/a")).toBeUndefined();
    expect(await gameReleases("To-Recomp/a")).toEqual([note("v1")]);
    expect(cachedReleases("To-Recomp/a")).toEqual([note("v1")]);

    await gameReleases("To-Recomp/a");
    await gameReleases("To-Recomp/a");
    expect(tauri.invoke).toHaveBeenCalledTimes(1);

    tauri.invoke.mockResolvedValue([note("v2")]);
    vi.advanceTimersByTime(16 * 60 * 1000);
    expect(await gameReleases("To-Recomp/a")).toEqual([note("v2")]);
    expect(tauri.invoke).toHaveBeenCalledTimes(2);
  });

  it("doesn't keep a failure", async () => {
    tauri.invoke.mockRejectedValueOnce("offline").mockResolvedValue([note("v1")]);
    await expect(gameReleases("To-Recomp/b")).rejects.toBe("offline");
    expect(await gameReleases("To-Recomp/b")).toEqual([note("v1")]);
  });
});
