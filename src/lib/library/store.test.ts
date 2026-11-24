import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Game, Platform } from "../catalog/games";

const api = vi.hoisted(() => ({
  librarySnapshot: vi.fn(),
  installGame: vi.fn(),
  prepareInstance: vi.fn(),
  startGame: vi.fn(),
  cancelJob: vi.fn(),
  readInstanceLog: vi.fn(),
  onProgress: vi.fn(),
  onGameExited: vi.fn(),
  pickInput: vi.fn(),
}));
vi.mock("./api", () => api);

const diagnostics = vi.hoisted(() => ({ buildReport: vi.fn(), pendingPanic: vi.fn(), dismissPanic: vi.fn() }));
vi.mock("../diagnostics/api", () => diagnostics);

import { LibraryStore } from "./store.svelte";
import { CANCELLED, instanceId, type InstallationView, type InstanceState, type StageEvent } from "./types";

const platform: Platform = { platform: "GameCube", regions: ["France (PAL)", "Japan"], state: "playable", maturity: "beta", releaseYear: null, year: 2003 };
const game: Game = { id: "tos", icon: "S", name: "Tales of Symphonia", repository: "To-Recomp/tos-recomp", year: 2003, state: "not-installed", platforms: [platform] };

const disc = (id: string, optional = false) => ({ id, label: `Label ${id}`, role: "disc", optional, kinds: ["file"], extensions: ["iso"] });
function installation(state: InstanceState | null, extra: Partial<InstallationView> = {}): InstallationView {
  return {
    gameId: "tos", repository: game.repository, tag: "v1", inputs: [disc("disc1"), disc("disc2"), disc("dlc", true)],
    prepareLabel: "Building the game", canPrepare: true, keepsCopy: true,
    instances: state ? [{ id: "gamecube", platform: "GameCube", state, inputs: {} }] : [],
    ...extra,
  };
}

let progressListener: (e: { job: string; fraction: number | null; message: string; stage?: StageEvent | null }) => void;
let exitListener: (e: { job: string; code: number | null }) => Promise<void> | void;

beforeEach(() => {
  vi.clearAllMocks();
  api.librarySnapshot.mockResolvedValue([]);
  api.onProgress.mockImplementation(async (l) => ((progressListener = l), () => {}));
  api.onGameExited.mockImplementation(async (l) => ((exitListener = l), () => {}));
  diagnostics.pendingPanic.mockResolvedValue(null);
  diagnostics.dismissPanic.mockResolvedValue(undefined);
});

describe("instanceId", () => {
  it.each([
    ["GameCube", "gamecube"],
    ["PS1", "ps1"],
    ["Xbox Series X|S", "xbox-series-x-s"],
    ["../../etc", "etc"],
    ["日本", "x"],
  ])("%s -> %s (same as the backend)", (p, expected) => {
    expect(instanceId(p)).toBe(expected);
  });
});

describe("LibraryStore state", () => {
  it("maps installations to the sidebar state and the instance state", async () => {
    const store = new LibraryStore();
    api.librarySnapshot.mockResolvedValue([installation("needsRebuild")]);
    await store.start();
    expect(store.gameState("tos")).toBe("installed");
    expect(store.gameState("other")).toBe("not-installed");
    expect(store.instanceState("tos", platform)).toBe("needsRebuild");
    expect(store.instanceState("tos", { ...platform, platform: "PlayStation 2" })).toBe("notPrepared");

    api.librarySnapshot.mockResolvedValue([installation("ready")]);
    await store.refresh();
    expect(store.gameState("tos")).toBe("ready");
  });

  it("follows the backend's progress and the end of a game", async () => {
    const store = new LibraryStore();
    await store.start();
    progressListener({ job: "prepare:tos/x", fraction: 0.5, message: "Compiling" });
    expect(store.isRunning("prepare:tos/x")).toBe(true);
    expect(store.progress["prepare:tos/x"].message).toBe("Compiling");

    progressListener({ job: "game:tos/x", fraction: null, message: "running" });
    api.librarySnapshot.mockResolvedValue([installation("ready")]);
    await exitListener({ job: "game:tos/x", code: 3 });
    expect(store.isRunning("game:tos/x")).toBe(false);
    expect(store.status).toEqual({ tone: "error", text: "The game stopped with exit code 3. Check the log." });
    expect(store.gameState("tos")).toBe("ready");
  });

  it("keeps the list of tasks a step reports, apart from the overall progress", async () => {
    const store = new LibraryStore();
    await store.start();
    const job = "prepare:tos/x";
    progressListener({ job, fraction: 0.2, message: "Copying", stage: null });
    progressListener({ job, fraction: null, message: "", stage: { id: "verify", event: "start", index: 1, count: 2, title: "Checking the discs" } });
    progressListener({ job, fraction: null, message: "", stage: { id: "verify", event: "end", status: "ok", seconds: 3.5 } });
    progressListener({ job, fraction: null, message: "", stage: { id: "compile", event: "start", index: 2, count: 2, title: "Compiling" } });
    progressListener({ job, fraction: null, message: "", stage: { id: "compile", event: "progress", done: 12, total: 48, text: "12/48 files" } });
    progressListener({ job, fraction: null, message: "", stage: { id: "other", event: "end", status: "ok", seconds: 1 } }); // never started: ignored

    expect(store.progress[job]).toMatchObject({ fraction: 0.2, message: "Copying" });
    expect(store.tasks[job]).toEqual([
      { id: "verify", title: "Checking the discs", index: 1, count: 2, state: "ok", fraction: 1, text: "", seconds: 3.5 },
      { id: "compile", title: "Compiling", index: 2, count: 2, state: "running", fraction: 0.25, text: "12/48 files", seconds: null },
    ]);
  });

  it("re-reads a log every second until stopped, and drops a read that finishes after that", async () => {
    vi.useFakeTimers();
    try {
      const store = new LibraryStore();
      api.readInstanceLog.mockResolvedValueOnce("line 1").mockResolvedValueOnce("line 1\nline 2");
      const stop = store.followLog(game, platform, "prepare");
      await vi.advanceTimersByTimeAsync(0);
      expect(store.log).toBe("line 1");
      await vi.advanceTimersByTimeAsync(1000);
      expect(store.log).toBe("line 1\nline 2");
      expect(api.readInstanceLog).toHaveBeenLastCalledWith(game.id, platform.platform, "prepare");

      const late = Promise.withResolvers<string>();
      api.readInstanceLog.mockReturnValueOnce(late.promise);
      await vi.advanceTimersByTimeAsync(1000);
      stop();
      late.resolve("late line");
      await vi.advanceTimersByTimeAsync(5000);
      expect(store.log).toBe("line 1\nline 2");
      expect(api.readInstanceLog).toHaveBeenCalledTimes(3);
    } finally {
      vi.useRealTimers();
    }
  });
});

describe("LibraryStore actions", () => {
  it("installs a game from its repository and reports it", async () => {
    const store = new LibraryStore();
    api.installGame.mockResolvedValue(installation(null, { tag: "v1.2.0" }));
    api.librarySnapshot.mockResolvedValue([installation(null, { tag: "v1.2.0" })]);
    await store.install(game);
    expect(api.installGame).toHaveBeenCalledWith("tos", "To-Recomp/tos-recomp");
    expect(store.status).toEqual({ tone: "success", text: "Installed Tales of Symphonia v1.2.0." });
    expect(store.isRunning("install:tos")).toBe(false);
    expect(store.gameState("tos")).toBe("installed");
  });

  it("shows a failure, and a cancel as information", async () => {
    const store = new LibraryStore();
    api.installGame.mockRejectedValue("The downloaded file doesn't match the checksum GitHub publishes");
    await store.install(game);
    expect(store.status).toEqual({ tone: "error", text: "The downloaded file doesn't match the checksum GitHub publishes" });
    api.installGame.mockRejectedValue(CANCELLED);
    await store.install(game);
    expect(store.status).toEqual({ tone: "info", text: "Cancelled." });
    expect(store.isRunning("install:tos")).toBe(false);
  });

  it("asks only for the files the game requires, then prepares", async () => {
    const store = new LibraryStore();
    api.librarySnapshot.mockResolvedValue([installation(null)]);
    await store.refresh();
    api.pickInput.mockImplementation(async (input) => `/discs/${input.id}.iso`);
    api.prepareInstance.mockResolvedValue({ id: "x", platform: "GameCube", state: "ready" });
    await store.prepare(game, platform);
    expect(api.pickInput).toHaveBeenCalledTimes(2);
    expect(api.prepareInstance).toHaveBeenCalledWith("tos", "GameCube", { disc1: "/discs/disc1.iso", disc2: "/discs/disc2.iso" });
    expect(store.status).toEqual({ tone: "success", text: "Tales of Symphonia is ready to play." });
  });

  it("changes nothing when the player closes the file dialog", async () => {
    const store = new LibraryStore();
    api.librarySnapshot.mockResolvedValue([installation(null)]);
    await store.refresh();
    api.pickInput.mockResolvedValueOnce("/discs/disc1.iso").mockResolvedValueOnce(null);
    await store.prepare(game, platform);
    expect(api.prepareInstance).not.toHaveBeenCalled();
    expect(store.status?.tone).toBe("info");
  });

  it("rebuilds after an update without asking, and asks only if a file is missing", async () => {
    const store = new LibraryStore();
    api.librarySnapshot.mockResolvedValue([installation("needsRebuild")]);
    await store.refresh();
    api.prepareInstance.mockResolvedValueOnce({ state: "ready" });
    await store.prepare(game, platform);
    expect(api.prepareInstance).toHaveBeenCalledWith("tos", "GameCube", {});
    expect(api.pickInput).not.toHaveBeenCalled();

    api.prepareInstance.mockRejectedValueOnce("Missing file: Label disc2.").mockResolvedValueOnce({ state: "ready" });
    api.pickInput.mockImplementation(async (input) => `/discs/${input.id}.iso`);
    await store.prepare(game, platform);
    expect(api.pickInput).toHaveBeenCalledTimes(2);
    expect(api.prepareInstance).toHaveBeenLastCalledWith("tos", "GameCube", { disc1: "/discs/disc1.iso", disc2: "/discs/disc2.iso" });
    expect(store.status?.tone).toBe("success");
  });

  it("starts the game and reports a failure to start", async () => {
    const store = new LibraryStore();
    api.startGame.mockResolvedValue(undefined);
    await store.play(game, platform);
    expect(api.startGame).toHaveBeenCalledWith("tos", "GameCube");
    expect(store.isRunning("game:tos/gamecube")).toBe(true);

    api.startGame.mockRejectedValue("This game isn't ready: prepare it first.");
    await store.play(game, platform);
    expect(store.isRunning("game:tos/gamecube")).toBe(false);
    expect(store.status).toEqual({ tone: "error", text: "This game isn't ready: prepare it first." });
  });

  it("reads a log", async () => {
    const store = new LibraryStore();
    api.readInstanceLog.mockResolvedValue("line 1\nline 2");
    await store.showLog(game, platform, "prepare");
    expect(api.readInstanceLog).toHaveBeenCalledWith("tos", "GameCube", "prepare");
    expect(store.log).toBe("line 1\nline 2");
    api.readInstanceLog.mockResolvedValue("");
    await store.showLog(game, platform, "game");
    expect(store.log).toBe("The log is empty.");
  });
});

describe("LibraryStore diagnostic files", () => {
  const file = { name: "tos-diagnostic-x.md", path: "/logs/tos-diagnostic-x.md", folder: "/logs", text: "Diagnostic format: 1" };

  it("offers a file for a failed preparation, about that instance, for the game's repository", async () => {
    const store = new LibraryStore();
    api.librarySnapshot.mockResolvedValue([installation(null)]);
    await store.start();
    api.pickInput.mockResolvedValue("/discs/a.iso");
    api.prepareInstance.mockRejectedValue("The preparation failed (exit code 7).");
    await store.prepare(game, platform);
    expect(store.failure).toMatchObject({ context: "prepare", gameId: "tos", platform });

    diagnostics.buildReport.mockResolvedValue(file);
    await store.reportFailure(game, undefined);
    expect(diagnostics.buildReport).toHaveBeenCalledWith({
      context: "prepare", gameId: "tos", platform: "GameCube", error: "The preparation failed (exit code 7).",
    });
    expect(store.diagnostic).toEqual({ repository: "To-Recomp/tos-recomp", file, error: null });
    store.closeDiagnostic();
    expect(store.diagnostic).toBeNull();
  });

  it("offers a file for a failed install and for a game that exits with an error, not for a cancel or a clean exit", async () => {
    const store = new LibraryStore();
    api.installGame.mockRejectedValue("Can't download the release.");
    await store.install(game);
    expect(store.failure).toMatchObject({ context: "install", gameId: "tos", platform: undefined });
    // An install has no instance: the page's platform stands in.
    diagnostics.buildReport.mockResolvedValue(file);
    await store.reportFailure(game, platform);
    expect(diagnostics.buildReport).toHaveBeenCalledWith(expect.objectContaining({ context: "install", platform: "GameCube" }));

    api.installGame.mockRejectedValue(CANCELLED);
    await store.install(game);
    expect(store.failure).toBeNull();

    api.librarySnapshot.mockResolvedValue([installation("ready")]);
    await store.start();
    api.startGame.mockResolvedValue(undefined);
    await store.play(game, platform);
    await exitListener({ job: "game:tos/gamecube", code: 3 });
    expect(store.failure).toMatchObject({ context: "game", gameId: "tos", platform, error: "The game stopped with exit code 3." });
    await store.play(game, platform);
    await exitListener({ job: "game:tos/gamecube", code: 0 });
    expect(store.failure).toBeNull();
  });

  it("ignores a failure of another game, and a Report a bug is about no game and the launcher's repository", async () => {
    const store = new LibraryStore();
    api.installGame.mockRejectedValue("nope");
    await store.install(game);
    await store.reportFailure({ ...game, id: "other" }, platform);
    expect(diagnostics.buildReport).not.toHaveBeenCalled();

    diagnostics.buildReport.mockResolvedValue(file);
    await store.reportBug();
    expect(diagnostics.buildReport).toHaveBeenCalledWith({ context: "manual" });
    expect(store.diagnostic?.repository).toBe("To-Recomp/launcher");
  });

  it("reports a game's About bug for that instance and the game's own repository", async () => {
    const store = new LibraryStore();
    diagnostics.buildReport.mockResolvedValue(file);
    await store.reportGameBug(game, platform);
    expect(diagnostics.buildReport).toHaveBeenCalledWith({ context: "manual", gameId: game.id, platform: platform.platform });
    expect(store.diagnostic?.repository).toBe(game.repository);
  });

  it("keeps why the file could not be written", async () => {
    const store = new LibraryStore();
    diagnostics.buildReport.mockRejectedValue("Can't write the diagnostic file in /logs: denied");
    await store.reportBug();
    expect(store.diagnostic).toEqual({ repository: "To-Recomp/launcher", file: null, error: "Can't write the diagnostic file in /logs: denied" });
  });
});

describe("LibraryStore launcher crash", () => {
  const file = { name: "launcher-diagnostic-x.md", path: "/logs/launcher-diagnostic-x.md", folder: "/logs", text: "Diagnostic format: 1" };

  it("offers the file of a panic at the previous run, for the launcher's repository, until dismissed", async () => {
    const store = new LibraryStore();
    diagnostics.pendingPanic.mockResolvedValue(file);
    await store.start();
    expect(store.crash).toEqual(file);

    store.viewCrash();
    expect(store.diagnostic).toEqual({ repository: "To-Recomp/launcher", file, error: null });

    await store.dismissCrash();
    expect(store.crash).toBeNull();
    expect(diagnostics.dismissPanic).toHaveBeenCalledOnce();
  });

  it("shows nothing after a normal run, or without a backend", async () => {
    const store = new LibraryStore();
    await store.start();
    expect(store.crash).toBeNull();
    diagnostics.pendingPanic.mockRejectedValue("no runtime");
    await store.start();
    expect(store.crash).toBeNull();
    store.viewCrash();
    expect(store.diagnostic).toBeNull();
  });
});
