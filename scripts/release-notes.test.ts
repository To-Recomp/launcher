import { execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
// @ts-expect-error plain JS module, no type declarations
import { buildNotes, parseSubject, previousTag } from "./release-notes.mjs";

describe("parseSubject", () => {
  it("reads the conventional type and scope", () => {
    expect(parseSubject("feat(ui): add a page (#5)")).toMatchObject({ type: "feat", scope: "ui", text: "add a page (#5)" });
    expect(parseSubject("fix: crash")).toMatchObject({ type: "fix", scope: "" });
  });
  it("files Dependabot bumps under dependencies", () => {
    expect(parseSubject("chore(deps): Bump x from 1 to 2 (#3)").type).toBe("deps");
  });
  it("keeps unknown and free-form subjects", () => {
    expect(parseSubject("Show the real releases (#24)").type).toBe("other");
    expect(parseSubject("weird(x): y").type).toBe("other");
  });
  it("flags breaking changes", () => {
    expect(parseSubject("feat!: drop the old format").breaking).toBe(true);
  });
});

describe("buildNotes", () => {
  const commits = [
    { hash: "aaaaaaa1", subject: "feat: settings page (#38)" },
    { hash: "bbbbbbb2", subject: "fix: proxy port (#51)" },
    { hash: "ccccccc3", subject: "chore: initial repository skeleton" },
  ];

  it("lists every commit by section on a first release", () => {
    const notes = buildNotes({ tag: "v0.1.0", repo: "To-Recomp/launcher", commits });
    expect(notes).toContain("First release: all 3 commits");
    expect(notes).toContain("### New features\n\n- settings page (#38)");
    expect(notes).toContain("### Fixes\n\n- proxy port (#51)");
    // A commit without a PR number shows its short hash.
    expect(notes).toContain("- initial repository skeleton (ccccccc)");
    expect(notes).not.toContain("Full changelog");
  });

  it("links the comparison with the previous release", () => {
    const notes = buildNotes({ tag: "v0.2.0", repo: "To-Recomp/launcher", previous: "v0.1.0", commits: commits.slice(0, 1) });
    expect(notes).toContain("Changes since v0.1.0: 1 commit.");
    expect(notes).toContain("https://github.com/To-Recomp/launcher/compare/v0.1.0...v0.2.0");
  });
});

interface TmpRepo {
  dir: string;
  git: (...args: string[]) => string;
}

describe("with a real git history", () => {
  function repo(): TmpRepo {
    const dir = mkdtempSync(join(tmpdir(), "relnotes-"));
    const git = (...a: string[]) => execFileSync("git", ["-c", "user.name=t", "-c", "user.email=t@example.org", "-C", dir, ...a], { encoding: "utf8" });
    git("init", "-q");
    return { dir, git };
  }
  const commit = (r: TmpRepo, msg: string) => {
    writeFileSync(join(r.dir, "f"), msg);
    r.git("add", "f");
    r.git("commit", "-q", "-m", msg);
  };

  it("finds the previous tag and lists only the commits after it", () => {
    const r = repo();
    commit(r, "feat: one");
    r.git("tag", "v0.1.0");
    commit(r, "fix: two (#2)");
    commit(r, "docs: three");
    r.git("tag", "v0.2.0-rc.1");
    const out = execFileSync("node", [join(__dirname, "release-notes.mjs"), "--tag", "v0.2.0-rc.1", "--repo", "o/r"], {
      cwd: r.dir,
      encoding: "utf8",
    });
    expect(out).toContain("Changes since v0.1.0: 2 commits.");
    expect(out).toContain("- two (#2)");
    expect(out).toContain("- three");
    expect(out).not.toContain("one");
    process.chdir(r.dir);
    expect(previousTag("v0.2.0-rc.1", "v0.2.0-rc.1^")).toBe("v0.1.0");
  });

  it("lists the whole history when there is no previous tag", () => {
    const r = repo();
    commit(r, "feat: one");
    commit(r, "fix: two");
    r.git("tag", "v0.1.0");
    const out = execFileSync("node", [join(__dirname, "release-notes.mjs"), "--tag", "v0.1.0", "--repo", "o/r"], {
      cwd: r.dir,
      encoding: "utf8",
    });
    expect(out).toContain("First release: all 2 commits");
  });
});
