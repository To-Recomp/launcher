// Builds the release description from every commit since the previous release.
//
//   node scripts/release-notes.mjs --tag v0.2.0 --repo To-Recomp/launcher [--previous v0.1.0]
//
// The previous release is the newest `v*` tag reachable from the tag's parent (or --previous);
// without one (the first release) every commit of the history is listed. Commits are grouped by
// their conventional-commit type; `(#123)` pull request numbers are kept, GitHub links them.
// Merge commits are skipped. Run from a checkout with the tags and the full history
// (`actions/checkout` with `fetch-depth: 0`).

import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";

/** Section title and order for each conventional-commit type. */
export const SECTIONS = [
  ["feat", "New features"],
  ["fix", "Fixes"],
  ["perf", "Performance"],
  ["docs", "Documentation"],
  ["deps", "Dependencies"],
  ["chore", "Maintenance"],
  ["other", "Other changes"],
];

/** @param {string} subject */
export function parseSubject(subject) {
  const m = /^(\w+)(?:\(([^)]+)\))?(!)?:\s*(.+)$/.exec(subject);
  if (!m) return { type: "other", scope: "", breaking: false, text: subject };
  let [, type, scope, bang, text] = m;
  type = type.toLowerCase();
  // Dependabot: "chore(deps): Bump x from 1 to 2".
  if (type === "chore" && scope === "deps") type = "deps";
  if (!SECTIONS.some(([t]) => t === type)) type = "other";
  return { type, scope: scope ?? "", breaking: Boolean(bang), text };
}

/**
 * @param {{ tag: string, repo: string, previous?: string | null, commits: { hash: string, subject: string }[] }} input
 */
export function buildNotes({ tag, repo, previous = null, commits }) {
  const lines = [];
  const intro = previous
    ? `Changes since ${previous}: ${commits.length} commit${commits.length === 1 ? "" : "s"}.`
    : `First release: all ${commits.length} commits of the history.`;
  lines.push(intro, "");

  const groups = new Map(SECTIONS.map(([type]) => [type, []]));
  for (const { hash, subject } of commits) {
    const parsed = parseSubject(subject);
    groups.get(parsed.type).push({ ...parsed, hash });
  }
  for (const [type, title] of SECTIONS) {
    const items = groups.get(type);
    if (items.length === 0) continue;
    lines.push(`### ${title}`, "");
    for (const { scope, breaking, text, hash } of items) {
      const prefix = scope && type !== "deps" ? `**${scope}**: ` : "";
      const flag = breaking ? " **(breaking)**" : "";
      const hasPr = /\(#\d+\)\s*$/.test(text);
      lines.push(`- ${prefix}${text}${flag}${hasPr ? "" : ` (${hash.slice(0, 7)})`}`);
    }
    lines.push("");
  }

  if (previous) lines.push(`**Full changelog**: https://github.com/${repo}/compare/${previous}...${tag}`, "");
  lines.push(
    "Verify a download with `sha256sum -c SHA256SUMS --ignore-missing`. The binaries are not signed for the",
    "operating system; see the releasing guide in the repository for what each asset is for.",
    "",
  );
  return lines.join("\n");
}

/** @param {string[]} args */
function git(args) {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}

/**
 * Newest `v*` tag reachable from `rev` (excluding `tag` itself), or null. A stable release (no `-`) is
 * compared with the previous stable one, not with the last release candidate: its notes cover everything
 * the candidates brought.
 */
export function previousTag(tag, rev) {
  const stable = !tag.includes("-");
  const tags = git(["-c", "versionsort.suffix=-", "tag", "--merged", rev, "--list", "v*", "--sort=-v:refname"]) // v1.0.0-rc.1 sorts before v1.0.0
    .split("\n")
    .filter((t) => t && t !== tag && (!stable || !t.includes("-")));
  return tags[0] ?? null;
}

function main(argv) {
  const args = Object.fromEntries(
    argv.flatMap((arg, i) => (arg.startsWith("--") ? [[arg.slice(2), argv[i + 1]]] : [])),
  );
  for (const required of ["tag", "repo"]) {
    if (!args[required]) throw new Error(`missing --${required}`);
  }
  const rev = `${args.tag}^{commit}`;
  const previous = args.previous ?? previousTag(args.tag, `${args.tag}^`);
  const range = previous ? `${previous}..${rev}` : rev;
  const log = git(["log", "--no-merges", "--format=%H%x09%s", range]);
  const commits = log
    ? log.split("\n").map((l) => {
        const [hash, ...rest] = l.split("\t");
        return { hash, subject: rest.join("\t") };
      })
    : [];
  process.stdout.write(buildNotes({ tag: args.tag, repo: args.repo, previous, commits }));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main(process.argv.slice(2));
