// Checks that a pull request title is a conventional commit: `type(scope)!: description`.
//
//   node scripts/check-pr-title.mjs "feat(ui): add a page"
//
// Exits 1 with an explanation otherwise. The title becomes the squashed commit on main and
// release-please reads it to pick the next version, so a title it can't parse is silently ignored.

import { pathToFileURL } from "node:url";

export const TYPES = ["feat", "fix", "perf", "docs", "chore", "ci", "build", "test", "refactor", "style", "revert"];

/** @param {string} title @returns {string | null} the problem, or null when the title is fine */
export function titleProblem(title) {
  const m = /^(\w+)(?:\(([^)]+)\))?(!)?: (\S.*)$/.exec(title.trim());
  if (!m) return "expected `type(scope)!: description` (scope and `!` are optional)";
  if (!TYPES.includes(m[1])) return `unknown type "${m[1]}", use one of: ${TYPES.join(", ")}`;
  return null;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const title = process.argv[2] ?? "";
  const problem = titleProblem(title);
  if (problem) {
    console.error(`::error::Pull request title "${title}": ${problem}`);
    process.exit(1);
  }
}
