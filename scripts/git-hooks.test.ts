import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

const hooks = join(process.cwd(), ".githooks");
let repo: string;

const git = (...args: string[]) =>
  execFileSync("git", args, { cwd: repo, encoding: "utf8", env: { ...process.env, GIT_EDITOR: ":" } }).trim();
const message = () => git("log", "-1", "--format=%B");
const signoffs = () => message().split("\n").filter((l) => l.startsWith("Signed-off-by:"));

beforeEach(() => {
  repo = mkdtempSync(join(tmpdir(), "hook-"));
  git("init", "-q");
  git("config", "user.name", "Ada Lovelace");
  git("config", "user.email", "ada@example.org");
  git("config", "commit.gpgsign", "false");
  git("config", "core.hooksPath", hooks);
});
afterEach(() => rmSync(repo, { recursive: true, force: true }));

describe(".githooks/prepare-commit-msg", () => {
  it("signs off a commit made without -s", () => {
    git("commit", "-q", "--allow-empty", "-m", "feat: x");
    expect(signoffs()).toEqual(["Signed-off-by: Ada Lovelace <ada@example.org>"]);
    expect(message()).toMatch(/^feat: x\n\nSigned-off-by:/);
  });

  it("doesn't duplicate the line of `git commit -s`", () => {
    git("commit", "-q", "-s", "--allow-empty", "-m", "feat: x");
    expect(signoffs()).toHaveLength(1);
  });

  it("keeps someone else's sign-off and adds yours", () => {
    git("commit", "-q", "--allow-empty", "-m", "feat: x\n\nSigned-off-by: Bob <bob@example.org>");
    expect(signoffs()).toEqual(["Signed-off-by: Bob <bob@example.org>", "Signed-off-by: Ada Lovelace <ada@example.org>"]);
  });

  it("signs off a message typed in the editor, above the comment lines", () => {
    writeFileSync(join(repo, "msg.sh"), '#!/bin/sh\nprintf "fix: y\\n\\n%s" "$(cat "$1")" > "$1"\n', { mode: 0o755 });
    execFileSync("git", ["commit", "-q", "--allow-empty"], {
      cwd: repo,
      env: { ...process.env, GIT_EDITOR: join(repo, "msg.sh") },
    });
    expect(signoffs()).toEqual(["Signed-off-by: Ada Lovelace <ada@example.org>"]);
    expect(message()).toMatch(/^fix: y\n/);
  });
});
