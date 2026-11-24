import { locale } from "../i18n/index.svelte";
import { invoke } from "@tauri-apps/api/core";

/** One published release of a game's repository. */
export interface ReleaseNote {
  tag: string;
  name: string | null;
  /** ISO 8601; null for a release that was never published. */
  publishedAt: string | null;
  /** Markdown, as written on GitHub. */
  body: string;
  prerelease: boolean;
}

/** The newest releases of `owner/name`, newest first; empty when there are none. */
export const gameReleases = (repository: string) => invoke<ReleaseNote[]>("game_releases", { repository });

/** The license GitHub detected in a repository. */
export interface RepositoryLicense {
  name: string;
  /** The license file on GitHub. */
  url: string;
}

/** The license of `owner/name`; `null` when it has none. */
export const gameLicense = (repository: string) => invoke<RepositoryLicense | null>("game_license", { repository });

/** "Nov 21, 2026", or "" when the release has no date. */
export function releaseDate(iso: string | null): string {
  const date = iso ? new Date(iso) : null;
  if (!date || Number.isNaN(date.getTime())) return "";
  return date.toLocaleDateString(locale(), { month: "short", day: "numeric", year: "numeric", timeZone: "UTC" });
}

/**
 * The first `max` entries of a release's notes, one per markdown list item or line, and how many more
 * there are. Headings, links, URLs and what GitHub generates around the entries ("What's Changed",
 * "by @user in <pull request>", "Full Changelog") are left out.
 */
export function releaseEntries(body: string, max = 4): { entries: string[]; more: number } {
  const all = body
    .replace(/```[\s\S]*?```/g, "\n")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, " ")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .split("\n")
    .filter((line) => !/^\s{0,3}#{1,6}\s/.test(line))
    .map((line) =>
      line
        .replace(/^\s{0,3}([-*+>]|\d+\.)\s+/, "")
        .replace(/\s+by @[\w-]+(\[bot\])?\s+in\s+https?:\/\/\S+/gi, "")
        .replace(/https?:\/\/\S+/g, "")
        .replace(/[*_`~]/g, "")
        .replace(/\s+/g, " ")
        .trim(),
    )
    .filter((line) => line && !/^full changelog\b/i.test(line));
  return { entries: all.slice(0, max), more: Math.max(0, all.length - max) };
}
