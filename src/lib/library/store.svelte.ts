import type { Game, GameState, Platform } from "../catalog/games";
import { errorMessage } from "../settings/api";
import * as api from "./api";
import {
  CANCELLED,
  gameJob,
  installJob,
  instanceId,
  prepareJob,
  type InstallationView,
  type InstanceState,
  type ProgressEvent,
} from "./types";

export type Tone = "info" | "success" | "error";

export interface Status {
  tone: Tone;
  text: string;
}

/** Installations, instances and what the launcher is doing with them, for the screens. */
export class LibraryStore {
  installations = $state<InstallationView[]>([]);
  /** Progress of the running jobs, by job key. */
  progress = $state<Record<string, ProgressEvent>>({});
  status = $state<Status | null>(null);
  log = $state<string | null>(null);

  #unlisten: (() => void)[] = [];

  /** Starts listening to the backend's progress events and loads what is installed. */
  async start() {
    try {
      this.#unlisten.push(
        await api.onProgress((e) => (this.progress[e.job] = e)),
        await api.onGameExited(async (e) => {
          delete this.progress[e.job];
          this.status = e.code
            ? { tone: "error", text: `The game stopped with exit code ${e.code}. Check the log.` }
            : { tone: "info", text: "The game closed." };
          await this.refresh();
        }),
      );
    } catch {
      // No Tauri runtime (plain browser): nothing to listen to.
    }
    await this.refresh();
  }

  stop() {
    this.#unlisten.forEach((f) => f());
    this.#unlisten = [];
  }

  async refresh() {
    try {
      this.installations = await api.librarySnapshot();
    } catch {
      this.installations = [];
    }
  }

  installationOf(gameId: string): InstallationView | undefined {
    return this.installations.find((i) => i.gameId === gameId);
  }

  instanceState(gameId: string, platform: Platform): InstanceState {
    const id = instanceId(platform.platform, platform.region);
    return this.installationOf(gameId)?.instances.find((i) => i.id === id)?.state ?? "notPrepared";
  }

  /** The sidebar icon: download, settings once installed, play when an instance is ready. */
  gameState(gameId: string): GameState {
    const installation = this.installationOf(gameId);
    if (!installation) return "not-installed";
    return installation.instances.some((i) => i.state === "ready") ? "ready" : "installed";
  }

  isRunning(job: string): boolean {
    return job in this.progress;
  }

  cancel(job: string) {
    return api.cancelJob(job);
  }

  /** Runs one job: progress under its key while it lasts, the result in `status`. */
  async #run<T>(job: string, doing: string, work: () => Promise<T>, done: (result: T) => string): Promise<T | null> {
    this.progress[job] = { job, fraction: null, message: doing };
    this.status = null;
    this.log = null;
    try {
      const result = await work();
      this.status = { tone: "success", text: done(result) };
      return result;
    } catch (e) {
      const message = errorMessage(e);
      this.status = message === CANCELLED ? { tone: "info", text: "Cancelled." } : { tone: "error", text: message };
      return null;
    } finally {
      delete this.progress[job];
      await this.refresh();
    }
  }

  async install(game: Game) {
    await this.#run(
      installJob(game.id),
      "Looking for the latest release",
      () => api.installGame(game.id, game.repository),
      (installation) => `Installed ${game.name} ${installation.tag}.`,
    );
  }

  /**
   * Prepares an instance from the player's files. A rebuild first tries what the instance
   * already has (the files it kept or was given before); only if a file is missing are they
   * asked for.
   */
  async prepare(game: Game, platform: Platform) {
    const installation = this.installationOf(game.id);
    if (!installation) return;
    const job = prepareJob(game.id, instanceId(platform.platform, platform.region));
    const run = (inputs: Record<string, string>) =>
      this.#run(
        job,
        installation.prepareLabel ?? "Preparing the game",
        () => api.prepareInstance(game.id, platform.platform, platform.region, inputs),
        () => `${game.name} is ready to play.`,
      );

    if (this.instanceState(game.id, platform) === "needsRebuild") {
      await run({});
      if (!this.status?.text.startsWith("Missing file")) return;
    }
    const inputs: Record<string, string> = {};
    for (const input of installation.inputs.filter((i) => !i.optional)) {
      const picked = await api.pickInput(input);
      if (!picked) {
        this.status = { tone: "info", text: "Nothing was changed: no file was chosen." };
        return;
      }
      inputs[input.id] = picked;
    }
    await run(inputs);
  }

  async play(game: Game, platform: Platform) {
    const job = gameJob(game.id, instanceId(platform.platform, platform.region));
    this.progress[job] = { job, fraction: null, message: `${game.name} is running` };
    this.status = { tone: "info", text: `${game.name} is running.` };
    try {
      await api.startGame(game.id, platform.platform, platform.region);
    } catch (e) {
      delete this.progress[job];
      this.status = { tone: "error", text: errorMessage(e) };
    }
  }

  async showLog(game: Game, platform: Platform, kind: "prepare" | "game") {
    try {
      const text = await api.readInstanceLog(game.id, platform.platform, platform.region, kind);
      this.log = text || "The log is empty.";
    } catch (e) {
      this.log = errorMessage(e);
    }
  }
}
