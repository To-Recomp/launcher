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
  region: string;
  state: InstanceState;
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

export interface ProgressEvent {
  job: string;
  fraction: number | null;
  message: string;
}

/** What the backend rejects with when the player cancels. */
export const CANCELLED = "Cancelled.";

/** `("GameCube", "France (PAL)")` -> `gamecube-france-pal`; mirrors the backend. */
export function instanceId(platform: string, region: string): string {
  const slug = (text: string) => {
    const out = text
      .replace(/[^A-Za-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .toLowerCase();
    return out || "x";
  };
  return `${slug(platform)}-${slug(region)}`;
}

export const installJob = (gameId: string) => `install:${gameId}`;
export const prepareJob = (gameId: string, instance: string) => `prepare:${gameId}/${instance}`;
export const gameJob = (gameId: string, instance: string) => `game:${gameId}/${instance}`;
