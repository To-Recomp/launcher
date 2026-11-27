// Mirrors src-tauri/src/library/mod.rs (serde camelCase).

export type InstanceState = "notPrepared" | "ready" | "needsRebuild";

export interface InputView {
  id: string;
  label: string;
  role: string;
  optional: boolean;
  /** `file` and/or `folder`. */
  kinds: string[];
  /** File extensions without the dot; empty for any. */
  extensions: string[];
}

export interface InstanceView {
  id: string;
  platform: string;
  state: InstanceState;
  /** The files the instance was prepared from, by input id. */
  inputs: Record<string, string>;
}

export interface InstallationView {
  gameId: string;
  repository: string;
  tag: string;
  inputs: InputView[];
  prepareLabel: string | null;
  canPrepare: boolean;
  keepsCopy: boolean;
  instances: InstanceView[];
}

/** A task of a step's own list, as the backend forwards the step's `stage-*` lines. */
export type StageEvent = { id: string } & (
  | { event: "start"; index: number; count: number; title: string }
  | { event: "progress"; done: number; total: number; text: string }
  | { event: "end"; status: string; seconds: number }
);

export interface ProgressEvent {
  job: string;
  fraction: number | null;
  message: string;
  /** Set when the event is about one task of the step's list; then `fraction` and `message` are empty. */
  stage?: StageEvent | null;
}

/** One task of a running step, for the list under the progress bar. */
export interface TaskView {
  id: string;
  title: string;
  /** 1-based position and number of tasks of the step. */
  index: number;
  count: number;
  state: "running" | "ok" | "skipped" | "failed" | "cancelled";
  /** 0 to 1 while running, when the task counts its work. */
  fraction: number | null;
  /** What the task is doing right now. */
  text: string;
  /** How long it took, once ended. */
  seconds: number | null;
}

/** What the backend rejects with when the player cancels. */
export const CANCELLED = "Cancelled.";

/** What a preparation rejects with when it stopped after its verification to ask the player to go on (the discs are not the tested ones). */
export const NEEDS_CONFIRMATION = "Needs confirmation.";


/** `"GameCube"` -> `gamecube`, `"Xbox Series X|S"` -> `xbox-series-x-s`; mirrors the backend. */
export function instanceId(platform: string): string {
  const out = platform
    .replace(/[^A-Za-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .toLowerCase();
  return out || "x";
}

export const installJob = (gameId: string) => `install:${gameId}`;
export const prepareJob = (gameId: string, instance: string) => `prepare:${gameId}/${instance}`;
export const gameJob = (gameId: string, instance: string) => `game:${gameId}/${instance}`;
