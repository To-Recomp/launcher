import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const read = (path: string) => readFileSync(join(process.cwd(), path), "utf8");
const json = (path: string) => JSON.parse(read(path));

const stable = json("release-please-config.json");
const rc = json("release-please-config.rc.json");
const pkg = stable.packages["."];
const version: string = json("package.json").version;

describe("release-please configuration", () => {
  it("has a stable and an rc channel that differ only by `prerelease`", () => {
    expect(pkg.prerelease).toBe(false);
    expect(rc.packages["."].prerelease).toBe(true);
    expect({ ...rc, packages: { ".": { ...rc.packages["."], prerelease: false } } }).toEqual(stable);
  });

  it("starts from the version the app has now", () => {
    expect(json(".release-please-manifest.json")).toEqual({ ".": version });
  });

  // package.json and package-lock.json are handled by the `node` release type; the rest by extra-files.
  it("updates every other file that carries the version, each one holding it today", () => {
    const files: { path: string; type: string; jsonpath?: string }[] = pkg["extra-files"];
    expect(files.map((f) => f.path).sort()).toEqual([
      "packaging/rpm/to-recomp-launcher.spec",
      "src-tauri/Cargo.lock",
      "src-tauri/Cargo.toml",
      "src-tauri/tauri.conf.json",
    ]);
    expect(pkg["release-type"]).toBe("node");
    expect(json("package-lock.json").version).toBe(version);
    expect(json("package-lock.json").packages[""].version).toBe(version);
    expect(json("src-tauri/tauri.conf.json").version).toBe(version);
    expect(read("src-tauri/Cargo.toml")).toMatch(new RegExp(`^version = "${version}"$`, "m"));
    expect(read("src-tauri/Cargo.lock")).toContain(`name = "to-recomp-launcher"\nversion = "${version}"`);
    // The spec is edited through markers, not a path in the config.
    expect(read("packaging/rpm/to-recomp-launcher.spec")).toMatch(
      new RegExp(`# x-release-please-start-version\\n%global upstream_version ${version}\\n# x-release-please-end`),
    );
  });

  it("numbers release candidates like the existing tags (rc.1, rc.2…)", () => {
    expect(pkg.versioning).toBe("prerelease");
    expect(pkg["prerelease-type"]).toBe("rc.1");
    expect(pkg["include-component-in-tag"]).toBe(false);
  });
});
