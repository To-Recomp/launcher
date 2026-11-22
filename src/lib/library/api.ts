import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { Location } from "../settings/types";
import type { InputView, InstallationView, InstanceView, ProgressEvent } from "./types";

export const librarySnapshot = () => invoke<InstallationView[]>("library_snapshot");

export const installGame = (gameId: string, repository: string) =>
  invoke<InstallationView>("install_game", { gameId, repository });

export const prepareInstance = (gameId: string, platform: string, inputs: Record<string, string>) =>
  invoke<InstanceView>("prepare_instance", { gameId, platform, inputs });

export const startGame = (gameId: string, platform: string) => invoke<void>("start_game", { gameId, platform });

export const cancelJob = (job: string) => invoke<void>("cancel_job", { job });

export const readInstanceLog = (gameId: string, platform: string, kind: "prepare" | "game") =>
  invoke<string>("read_instance_log", { gameId, platform, kind });

/** Where the settings of an instance live; `null` while the game isn't installed. */
export const instanceLocation = (gameId: string, platform: string) =>
  invoke<Location | null>("instance_location", { gameId, platform });

export const onProgress = (listener: (event: ProgressEvent) => void) =>
  listen<ProgressEvent>("library-progress", (e) => listener(e.payload));

export const onGameExited = (listener: (event: { job: string; code: number | null }) => void) =>
  listen<{ job: string; code: number | null }>("library-game-exited", (e) => listener(e.payload));

/** Asks the player for one input; `null` when they close the dialog. */
export async function pickInput(input: InputView): Promise<string | null> {
  const folderOnly = input.kinds.length === 1 && input.kinds[0] === "folder";
  const picked = await open({
    title: input.label,
    multiple: false,
    directory: folderOnly,
    filters: !folderOnly && input.extensions.length ? [{ name: input.label, extensions: input.extensions }] : undefined,
  });
  return typeof picked === "string" ? picked : null;
}
