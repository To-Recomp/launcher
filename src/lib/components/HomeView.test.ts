import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

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
vi.mock("../library/api", () => api);

import { LibraryStore } from "../library/store.svelte";
import type { InstallationView, InstanceState } from "../library/types";
import HomeView from "./HomeView.svelte";
import { eternia } from "./fixtures";

const platform = eternia.platforms[0];
const input = (id: string, kinds = ["file"], optional = false) => ({ id, label: id, role: "disc", optional, kinds, extensions: [] });

function installed(state: InstanceState | null, extra: Partial<InstallationView> = {}): InstallationView {
  return {
    gameId: "toe", repository: "To-Recomp/toe", tag: "v1", inputs: [input("disc1")], prepareLabel: null, canPrepare: true, keepsCopy: false,
    instances: state ? [{ id: "ps1-usa", platform: "PS1", region: "USA", state }] : [], ...extra,
  };
}

function show(installation: InstallationView | null) {
  const library = new LibraryStore();
  library.installations = installation ? [installation] : [];
  render(HomeView, { game: eternia, platform, library, onplatform: () => {} });
  return library;
}

beforeEach(() => vi.clearAllMocks());

describe("HomeView actions", () => {
  it("offers Install for a game that isn't installed", async () => {
    show(null);
    api.installGame.mockResolvedValue(installed(null));
    api.librarySnapshot.mockResolvedValue([]);
    expect(screen.getByText("Install the game to get started.")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Install" }));
    expect(api.installGame).toHaveBeenCalledWith("toe", "To-Recomp/toe");
  });

  it.each([
    [[input("disc1"), input("disc2")], "Choose game files"],
    [[input("disc1")], "Choose game file"],
    [[input("game", ["folder"])], "Choose game folder"],
    [[input("dlc", ["file"], true)], "Prepare"],
  ])("asks for the files of an unprepared instance: %#", (inputs, label) => {
    show(installed(null, { inputs }));
    expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
  });

  it("plays a ready instance and can prepare it again", async () => {
    show(installed("ready"));
    api.startGame.mockResolvedValue(undefined);
    expect(screen.getByText("Ready to play.")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Play" }));
    expect(api.startGame).toHaveBeenCalledWith("toe", "PS1", "USA");
    expect(screen.getByRole("button", { name: "Change game files" })).toBeInTheDocument();
  });

  it("says an update needs a new build, and offers Rebuild when the instance kept what it needs", () => {
    show(installed("needsRebuild", { keepsCopy: true }));
    expect(screen.getByText("The game needs a new build after the update.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Rebuild" })).toBeInTheDocument();
  });
});

describe("HomeView status", () => {
  it("shows progress with a way to cancel while a job runs", async () => {
    const library = show(installed(null));
    library.progress["prepare:toe/ps1-usa"] = { job: "prepare:toe/ps1-usa", fraction: 0.4, message: "Compiling" };
    expect(await screen.findByText("Compiling · 40%")).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "40");
    expect(screen.getByRole("button", { name: "Choose game file" })).toBeDisabled();
    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(api.cancelJob).toHaveBeenCalledWith("prepare:toe/ps1-usa");
  });

  it("shows the last result and the logs on demand", async () => {
    const library = show(installed("ready"));
    library.status = { tone: "error", text: "The preparation failed (exit code 7)." };
    expect(await screen.findByText("The preparation failed (exit code 7).")).toBeInTheDocument();

    api.readInstanceLog.mockResolvedValue("progress 0.1 Reading\ndisc is unreadable");
    await fireEvent.click(screen.getByRole("button", { name: "Preparation log" }));
    expect(await screen.findByLabelText("Log")).toHaveTextContent("disc is unreadable");
    expect(api.readInstanceLog).toHaveBeenCalledWith("toe", "PS1", "USA", "prepare");
    await fireEvent.click(screen.getByRole("button", { name: "Game log" }));
    expect(api.readInstanceLog).toHaveBeenLastCalledWith("toe", "PS1", "USA", "game");
  });
});

describe("HomeView diagnostic file", () => {
  it("offers Create diagnostic file after a failure of this game only, when nothing is running", async () => {
    const library = show(installed("ready"));
    expect(screen.queryByRole("button", { name: "Create diagnostic file" })).not.toBeInTheDocument();

    library.failure = { context: "game", gameId: "other", error: "x" };
    await Promise.resolve();
    expect(screen.queryByRole("button", { name: "Create diagnostic file" })).not.toBeInTheDocument();

    library.failure = { context: "game", gameId: "toe", platform, error: "The game stopped with exit code 3." };
    const button = await screen.findByRole("button", { name: "Create diagnostic file" });
    const report = vi.spyOn(library, "reportFailure").mockResolvedValue();
    await fireEvent.click(button);
    expect(report).toHaveBeenCalledWith(eternia, platform);

    library.progress["game:toe/ps1-usa"] = { job: "game:toe/ps1-usa", fraction: null, message: "running" };
    await waitFor(() => expect(screen.queryByRole("button", { name: "Create diagnostic file" })).not.toBeInTheDocument());
  });
});
