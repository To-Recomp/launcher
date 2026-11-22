import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";

// Mirrors src-tauri/src/diagnostics/mod.rs (serde camelCase).

export type DiagnosticContext = "install" | "prepare" | "game" | "manual" | "launcher";

export interface DiagnosticRequest {
  context: DiagnosticContext;
  gameId?: string;
  platform?: string;
  error?: string;
}

export interface DiagnosticFile {
  name: string;
  path: string;
  folder: string;
  /** Exactly the file's content. */
  text: string;
}

/** Writes the scrubbed diagnostic file; nothing leaves the machine. */
export const buildReport = (request: DiagnosticRequest) => invoke<DiagnosticFile>("build_report", { request });

/** The file a launcher panic left at the previous run, `null` when the last run ended normally. */
export const pendingPanic = () => invoke<DiagnosticFile | null>("pending_panic");

/** Stops offering that file; it stays on disk until rotation removes it. */
export const dismissPanic = () => invoke<void>("dismiss_panic");

/** Copies with the web API, and with the clipboard plugin where WebKitGTK refuses it. */
export async function copyText(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    await writeText(text);
  }
}

export const revealFile = (path: string) => revealItemInDir(path);

/** The page where the player pastes the file: a new issue of the repository (`owner/name`). */
export const issuesUrl = (repository: string) => `https://github.com/${repository}/issues/new`;

export const openIssues = (repository: string) => openUrl(issuesUrl(repository));
