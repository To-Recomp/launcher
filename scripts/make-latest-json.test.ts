import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
// @ts-expect-error plain JS module, no type declarations
import { buildLatest } from "./make-latest-json.mjs";

const base = { version: "0.2.0", tag: "v0.2.0", repo: "To-Recomp/launcher", pubDate: "2026-11-22T12:00:00.000Z" };

describe("buildLatest", () => {
  it("lists each signed platform with its download URL and signature", () => {
    const latest = buildLatest({
      ...base,
      signatures: {
        "To-Recomp-Launcher_0.2.0_linux-x86_64.AppImage": "sig-linux\n",
        "To-Recomp-Launcher_0.2.0_windows-x86_64-setup.exe": "sig-win\n",
        "something-else.zip": "ignored",
      },
    });
    expect(latest).toEqual({
      version: "0.2.0",
      notes: "https://github.com/To-Recomp/launcher/releases/tag/v0.2.0",
      pub_date: "2026-11-22T12:00:00.000Z",
      platforms: {
        "linux-x86_64": {
          signature: "sig-linux",
          url: "https://github.com/To-Recomp/launcher/releases/download/v0.2.0/To-Recomp-Launcher_0.2.0_linux-x86_64.AppImage",
        },
        "windows-x86_64": {
          signature: "sig-win",
          url: "https://github.com/To-Recomp/launcher/releases/download/v0.2.0/To-Recomp-Launcher_0.2.0_windows-x86_64-setup.exe",
        },
      },
    });
  });

  it("returns nothing when no asset is signed", () => {
    expect(buildLatest({ ...base, signatures: {} })).toBeNull();
  });

  it("fails loudly from the command line instead of writing an empty latest.json", () => {
    const dir = mkdtempSync(join(tmpdir(), "assets-"));
    writeFileSync(join(dir, "To-Recomp-Launcher_0.2.0_linux-x86_64.AppImage"), "binary");
    const run = spawnSync(
      "node",
      ["scripts/make-latest-json.mjs", "--version", "0.2.0", "--tag", "v0.2.0", "--repo", "a/b", "--dir", dir],
      { encoding: "utf8" },
    );
    expect(run.status).toBe(3);
    expect(run.stdout).toBe("");

    writeFileSync(join(dir, "To-Recomp-Launcher_0.2.0_linux-x86_64.AppImage.sig"), "sig\n");
    const out = execFileSync(
      "node",
      ["scripts/make-latest-json.mjs", "--version", "0.2.0", "--tag", "v0.2.0", "--repo", "a/b", "--dir", dir],
      { encoding: "utf8" },
    );
    expect(JSON.parse(out).platforms["linux-x86_64"].signature).toBe("sig");
  });
});
