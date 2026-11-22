import type { Game, GameState, Platform } from "../catalog/games";
import { buildReport, dismissPanic, pendingPanic, type DiagnosticFile, type DiagnosticRequest } from "../diagnostics/api";
import { LAUNCHER_REPOSITORY } from "../data";
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

/** What a diagnostic file can be made about: the last install, preparation or game run that failed. */
export interface Failure {
  context: "install" | "prepare" | "game";
  gameId: string;
  /** The instance that failed; none for an install, which belongs to the whole game. */
  platform?: Platform;
  error: string;
}

/** The diagnostic file the player is asked to review, or why it could not be made. */
export interface Diagnostic {
  /** `owner/name` whose issues the file is meant for. */
  repository: string;
  file: DiagnosticFile | null;
  error: string | null;
}

/** Installations, instances and what the launcher is doing with them, for the screens. */
export class LibraryStore {
  installations = $state<InstallationView[]>([]);
  /** Progress of the running jobs, by job key. */
  progress = $state<Record<string, ProgressEvent>>({});
  status = $state<Status | null>(null);
  log = $state<string | null>(null);
  failure = $state<Failure | null>(null);
  diagnostic = $state<Diagnostic | null>(null);
  /** The file left by a launcher panic at the previous run, offered by a banner. */
  crash = $state<DiagnosticFile | null>(null);

  #unlisten: (() => void)[] = [];
  /** The games started by this session, to say which instance a non-zero exit belongs to. */
  #started: Record<string, { game: Game; platform: Platform }> = {};

  /** Starts listening to the backend's progress events and loads what is installed. */
  async start() {
    try {
      this.#unlisten.push(
        await api.onProgress((e) => (this.progress[e.job] = e)),
        await api.onGameExited(async (e) => {
          delete this.progress[e.job];
          const started = this.#started[e.job];
          this.failure =
            e.code && started
              ? { context: "game", gameId: started.game.id, platform: started.platform, error: `The game stopped with exit code ${e.code}.` }
              : null;
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
    try {
      this.crash = await pendingPanic();
    } catch {
      // No Tauri runtime: no previous run to report.
    }
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
    const id = instanceId(platform.platform);
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
  async #run<T>(
    job: string,
    target: { game: Game; platform?: Platform },
    doing: string,
    work: () => Promise<T>,
    done: (result: T) => string,
  ): Promise<T | null> {
    this.progress[job] = { job, fraction: null, message: doing };
    this.status = null;
    this.failure = null;
    this.log = null;
    try {
      const result = await work();
      this.status = { tone: "success", text: done(result) };
      return result;
    } catch (e) {
      const message = errorMessage(e);
      if (message === CANCELLED) {
        this.status = { tone: "info", text: "Cancelled." };
      } else {
        this.status = { tone: "error", text: message };
        const context = job.startsWith("install:") ? "install" : "prepare";
        this.failure = { context, gameId: target.game.id, platform: target.platform, error: message };
      }
      return null;
    } finally {
      delete this.progress[job];
      await this.refresh();
    }
  }

  async install(game: Game) {
    await this.#run(
      installJob(game.id),
      { game },
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
    const job = prepareJob(game.id, instanceId(platform.platform));
    const run = (inputs: Record<string, string>) =>
      this.#run(
        job,
        { game, platform },
        installation.prepareLabel ?? "Preparing the game",
        () => api.prepareInstance(game.id, platform.platform, inputs),
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
    const job = gameJob(game.id, instanceId(platform.platform));
    this.progress[job] = { job, fraction: null, message: `${game.name} is running` };
    this.status = { tone: "info", text: `${game.name} is running.` };
    this.failure = null;
    this.#started[job] = { game, platform };
    try {
      await api.startGame(game.id, platform.platform);
    } catch (e) {
      delete this.progress[job];
      const error = errorMessage(e);
      this.status = { tone: "error", text: error };
      this.failure = { context: "game", gameId: game.id, platform, error };
    }
  }

  async showLog(game: Game, platform: Platform, kind: "prepare" | "game") {
    try {
      const text = await api.readInstanceLog(game.id, platform.platform, kind);
      this.log = text || "The log is empty.";
    } catch (e) {
      this.log = errorMessage(e);
    }
  }

  /** Writes the diagnostic file of the last failure of `game` and shows it for review. */
  async reportFailure(game: Game, current: Platform | undefined) {
    const failure = this.failure;
    if (!failure || failure.gameId !== game.id) return;
    const platform = failure.platform ?? current;
    await this.#report(game.repository, {
      context: failure.context,
      gameId: game.id,
      platform: platform?.platform,
      error: failure.error,
    });
  }

  /** Settings → About → Report a bug: no game, the launcher's own issues. */
  reportBug() {
    return this.#report(LAUNCHER_REPOSITORY, { context: "manual" });
  }

  closeDiagnostic() {
    this.diagnostic = null;
  }

  async #report(repository: string, request: DiagnosticRequest) {
    try {
      this.diagnostic = { repository, file: await buildReport(request), error: null };
    } catch (e) {
      this.diagnostic = { repository, file: null, error: errorMessage(e) };
    }
  }

  /** The banner's *View file*: the same review dialog, for the launcher's own issues. */
  viewCrash() {
    if (this.crash) this.diagnostic = { repository: LAUNCHER_REPOSITORY, file: this.crash, error: null };
  }

  async dismissCrash() {
    this.crash = null;
    try {
      await dismissPanic();
    } catch {
      // The banner is gone either way; at worst it shows once more.
    }
  }
}
