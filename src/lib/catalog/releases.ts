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

/** The release body as one short plain-text line: one entry per markdown line, joined by " · ". */
export function releaseSummary(body: string, max = 160): string {
  const lines = body
    .replace(/```[\s\S]*?```/g, "\n")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, " ")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .split("\n")
    .map((line) =>
      line
        .replace(/^\s{0,3}(#{1,6}|[-*+>]|\d+\.)\s+/, "")
        .replace(/\s+by @[\w-]+(\[bot\])?\s+in\s+https?:\/\/\S+/gi, "") // GitHub's generated "by @user in <pull request>"
        .replace(/https?:\/\/\S+/g, "")
        .replace(/[*_`~]/g, "")
        .replace(/\s+/g, " ")
        .trim(),
    )
    .filter((line) => line && !/^(what's changed|full changelog\b.*)$/i.test(line));
  const text = lines.join(" · ");
  return text.length > max ? `${text.slice(0, max - 1).trimEnd()}…` : text;
}
