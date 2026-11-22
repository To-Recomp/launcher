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

/** "Nov 21, 2026", or "" when the release has no date. */
export function releaseDate(iso: string | null): string {
  const date = iso ? new Date(iso) : null;
  if (!date || Number.isNaN(date.getTime())) return "";
  return date.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric", timeZone: "UTC" });
}

/** The release body as one short plain-text paragraph. */
export function releaseSummary(body: string, max = 160): string {
  const text = body
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, " ")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/^\s{0,3}(#{1,6}|[-*+>]|\d+\.)\s+/gm, "")
    .replace(/[*_`~]/g, "")
    .replace(/\s+/g, " ")
    .trim();
  return text.length > max ? `${text.slice(0, max - 1).trimEnd()}…` : text;
}
