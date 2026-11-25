import type { Game, GameState, Platform } from "../catalog/games";
import { buildReport, dismissPanic, pendingPanic, type DiagnosticFile, type DiagnosticRequest } from "../diagnostics/api";
import { LAUNCHER_REPOSITORY } from "../data";
import { t } from "../i18n/index.svelte";
import { errorMessage } from "../settings/api";
import { rememberPlayed } from "./lastPlayed";
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
  type StageEvent,
  type TaskView,
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

/** How often a game is asked about a newer release (the backend keeps GitHub's answer as long), and the wait after a failure. */
const UPDATE_CHECK_MS = 15 * 60 * 1000;
const UPDATE_RETRY_MS = 60 * 1000;

/** Installations, instances and what the launcher is doing with them, for the screens. */
export class LibraryStore {
  installations = $state<InstallationView[]>([]);
  /** Progress of the running jobs, by job key. */
  progress = $state<Record<string, ProgressEvent>>({});
  /** The tasks a running step has reported, by job key, in the order they started. */
  tasks = $state<Record<string, TaskView[]>>({});
  status = $state<Status | null>(null);
  log = $state<string | null>(null);
  failure = $state<Failure | null>(null);
  diagnostic = $state<Diagnostic | null>(null);
  /** The file left by a launcher panic at the previous run, offered by a banner. */
  crash = $state<DiagnosticFile | null>(null);

  /** The newest release of each installed game's repository, by game id, once checked. */
  latest = $state<Record<string, string>>({});

  #unlisten: (() => void)[] = [];
  #checking = new Set<string>();
  /**
   * The install and preparation jobs this session is running. Events travel separately from the command's
   * answer, so one sent just before a job ended (a cancel) can arrive after it: without this it would
   * bring a finished job back, with its Cancel button and a progress bar that never goes away.
   */
  #running = new Set<string>();
  /** When each game's update check may run again (epoch ms). */
  #dueAt = new Map<string, number>();
  /** The games started by this session, to say which instance a non-zero exit belongs to. */
  #started: Record<string, { game: Game; platform: Platform }> = {};

  /** Starts listening to the backend's progress events and loads what is installed. */
  async start() {
    try {
      this.#unlisten.push(
        await api.onProgress((e) => {
          if (!this.#running.has(e.job)) return;
          if (e.stage) this.#task(e.job, e.stage);
          else this.progress[e.job] = e;
        }),
        await api.onGameExited(async (e) => {
          delete this.progress[e.job];
          const started = this.#started[e.job];
          this.failure =
            e.code && started
              ? { context: "game", gameId: started.game.id, platform: started.platform, error: t("shell.library.exitCode", { code: e.code }) }
              : null;
          this.status = e.code
            ? { tone: "error", text: t("shell.library.exitCodeCheckLog", { code: e.code }) }
            : { tone: "info", text: t("shell.library.closed") };
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

  /**
   * Asks GitHub for the newest release of an installed game, at most every 15 minutes (a minute after a
   * failure). The backend answers from its own cache within that time, so asking often costs nothing.
   */
  async checkUpdate(game: Game) {
    if (this.#checking.has(game.id) || !this.installationOf(game.id)) return;
    if (Date.now() < (this.#dueAt.get(game.id) ?? 0)) return;
    this.#checking.add(game.id);
    try {
      this.latest[game.id] = await api.latestReleaseTag(game.repository);
      this.#dueAt.set(game.id, Date.now() + UPDATE_CHECK_MS);
    } catch {
      this.#dueAt.set(game.id, Date.now() + UPDATE_RETRY_MS); // offline or limited by GitHub
    } finally {
      this.#checking.delete(game.id);
    }
  }

  /** The release to update to, when the installed one isn't the newest. */
  updateAvailable(gameId: string): string | null {
    const installed = this.installationOf(gameId)?.tag;
    const newest = this.latest[gameId];
    return installed && newest && newest !== installed ? newest : null;
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

  /** Applies one `stage-*` line of a running step to the list of its tasks. */
  #task(job: string, e: StageEvent) {
    this.tasks[job] ??= [];
    const list = this.tasks[job];
    if (e.event === "start") {
      const view: TaskView = { id: e.id, title: e.title, index: e.index, count: e.count, state: "running", fraction: null, text: "", seconds: null };
      const at = list.findIndex((task) => task.id === e.id);
      if (at >= 0) list[at] = view;
      else list.push(view);
      return;
    }
    const task = list.find((candidate) => candidate.id === e.id);
    if (!task) return;
    if (e.event === "progress") {
      task.fraction = e.total > 0 ? Math.min(1, e.done / e.total) : null;
      task.text = e.text;
    } else {
      task.state = (["ok", "skipped", "failed", "cancelled"] as const).find((s) => s === e.status) ?? "ok";
      task.fraction = task.state === "ok" ? 1 : task.fraction;
      task.seconds = e.seconds;
    }
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
    delete this.tasks[job];
    this.status = null;
    this.failure = null;
    this.log = null;
    this.#running.add(job);
    try {
      const result = await work();
      this.status = { tone: "success", text: done(result) };
      return result;
    } catch (e) {
      const message = errorMessage(e);
      if (message === CANCELLED) {
        this.status = { tone: "info", text: t("shell.library.cancelled") };
      } else {
        this.status = { tone: "error", text: message };
        const context = job.startsWith("install:") ? "install" : "prepare";
        this.failure = { context, gameId: target.game.id, platform: target.platform, error: message };
      }
      return null;
    } finally {
      this.#running.delete(job);
      delete this.progress[job];
      delete this.tasks[job];
      await this.refresh();
    }
  }

  async install(game: Game) {
    await this.#run(
      installJob(game.id),
      { game },
      t("shell.library.lookingForRelease"),
      () => api.installGame(game.id, game.repository),
      (installation) => t("shell.library.installed", { game: game.name, tag: installation.tag }),
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
        installation.prepareLabel ?? t("shell.library.preparing"),
        () => api.prepareInstance(game.id, platform.platform, inputs),
        () => t("shell.library.ready", { game: game.name }),
      );

    if (this.instanceState(game.id, platform) === "needsRebuild") {
      await run({});
      if (!this.status?.text.startsWith("Missing file")) return;
    }
    const inputs: Record<string, string> = {};
    for (const input of installation.inputs.filter((i) => !i.optional)) {
      const picked = await api.pickInput(input);
      if (!picked) {
        this.status = { tone: "info", text: t("shell.library.nothingChanged") };
        return;
      }
      inputs[input.id] = picked;
    }
    await run(inputs);
  }

  /**
   * Chooses another file for one input of an instance. Nothing is built: the choice is kept, and the game then
   * offers a rebuild that uses it, when the player wants it. The same file as the one in use changes nothing.
   */
  async changeInput(game: Game, platform: Platform, inputId: string) {
    const installation = this.installationOf(game.id);
    const input = installation?.inputs.find((i) => i.id === inputId);
    if (!installation || !input) return;
    const picked = await api.pickInput(input);
    if (!picked) {
      this.status = { tone: "info", text: t("shell.library.nothingChanged") };
      return;
    }
    const id = instanceId(platform.platform);
    if (installation.instances.find((i) => i.id === id)?.inputs[inputId] === picked) {
      this.status = { tone: "info", text: t("shell.library.sameFile") };
      return;
    }
    try {
      const view = await api.setInstanceInput(game.id, platform.platform, inputId, picked);
      // The backend also knows the same file reached by another path: it kept the one it had.
      this.status =
        view.inputs[inputId] === picked
          ? { tone: "success", text: t("shell.library.fileKept") }
          : { tone: "info", text: t("shell.library.sameFile") };
    } catch (e) {
      this.status = { tone: "error", text: errorMessage(e) };
    }
    await this.refresh();
  }

  async play(game: Game, platform: Platform) {
    const job = gameJob(game.id, instanceId(platform.platform));
    this.progress[job] = { job, fraction: null, message: t("shell.library.runningProgress", { game: game.name }) };
    this.status = { tone: "info", text: t("shell.library.running", { game: game.name }) };
    this.failure = null;
    this.#started[job] = { game, platform };
    try {
      await api.startGame(game.id, platform.platform);
      rememberPlayed(game.id, platform.platform);
    } catch (e) {
      delete this.progress[job];
      const error = errorMessage(e);
      this.status = { tone: "error", text: error };
      this.failure = { context: "game", gameId: game.id, platform, error };
    }
  }

  /** Deletes the build of an instance after the player confirmed; the saves are kept. */
  async deleteBuild(game: Game, platform: Platform) {
    const confirmed = await api.confirmDeletion(
      t("shell.library.deleteBuildConfirm", { game: game.name }),
      t("shell.library.deleteBuildTitle"),
    );
    if (!confirmed) return;
    this.log = null;
    this.failure = null;
    try {
      await api.deleteBuild(game.id, platform.platform);
      this.status = { tone: "success", text: t("shell.library.buildDeleted", { game: game.name }) };
    } catch (e) {
      this.status = { tone: "error", text: errorMessage(e) };
    }
    await this.refresh();
  }

  async showLog(game: Game, platform: Platform, kind: "prepare" | "game") {
    this.log = await this.#readLog(game, platform, kind);
  }

  /**
   * Shows a log and re-reads it every second, to watch a step while it runs. Returns the function that
   * stops it; a read still in flight when it is called is dropped.
   */
  followLog(game: Game, platform: Platform, kind: "prepare" | "game"): () => void {
    let active = true;
    const read = async () => {
      const text = await this.#readLog(game, platform, kind);
      if (active) this.log = text;
    };
    void read();
    const timer = setInterval(read, 1000);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }

  async #readLog(game: Game, platform: Platform, kind: "prepare" | "game"): Promise<string> {
    try {
      return (await api.readInstanceLog(game.id, platform.platform, kind)) || t("shell.library.logEmpty");
    } catch (e) {
      return errorMessage(e);
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

  /** Game → Settings → About → Report a bug: this instance, for the game's own repository. */
  reportGameBug(game: Game, platform: Platform | undefined) {
    return this.#report(game.repository, { context: "manual", gameId: game.id, platform: platform?.platform });
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
