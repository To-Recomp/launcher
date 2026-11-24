import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({
  librarySnapshot: vi.fn(),
  installGame: vi.fn(),
  prepareInstance: vi.fn(),
  startGame: vi.fn(),
  cancelJob: vi.fn(),
  deleteBuild: vi.fn(),
  confirmDeletion: vi.fn(),
  readInstanceLog: vi.fn(),
  onProgress: vi.fn(),
  onGameExited: vi.fn(),
  pickInput: vi.fn(),
}));
vi.mock("../library/api", () => api);

const releases = vi.hoisted(() => ({ gameReleases: vi.fn() }));
vi.mock("../catalog/releases", async (orig) => ({ ...(await orig<object>()), ...releases }));

import { LibraryStore } from "../library/store.svelte";
import type { InstallationView, InstanceState } from "../library/types";
import HomeView from "./HomeView.svelte";
import { eternia } from "./fixtures";

const platform = eternia.platforms[0];
const input = (id: string, kinds = ["file"], optional = false) => ({ id, label: id, role: "disc", optional, kinds, extensions: [] });

function installed(state: InstanceState | null, extra: Partial<InstallationView> = {}): InstallationView {
  return {
    gameId: "toe", repository: "To-Recomp/toe", tag: "v1", inputs: [input("disc1")], prepareLabel: null, canPrepare: true, keepsCopy: false,
    instances: state ? [{ id: "ps1", platform: "PS1", state, inputs: {} }] : [], ...extra,
  };
}

const onsettings = vi.fn();

function show(installation: InstallationView | null) {
  const library = new LibraryStore();
  library.installations = installation ? [installation] : [];
  render(HomeView, { game: eternia, platform, library, platformSizing: [], onplatform: () => {}, onsettings });
  return library;
}

beforeEach(() => {
  vi.clearAllMocks();
  releases.gameReleases.mockResolvedValue([]);
});

describe("latest releases", () => {
  it("shows the project's releases, newest first as the featured card", async () => {
    releases.gameReleases.mockResolvedValue([
      { tag: "v1.2.0", name: "Library", publishedAt: "2026-11-21T10:00:00Z", body: "## What's Changed\n* feat(ui): Faster startup by @me in https://github.com/x/y/pull/1\n* fix: Crisper text", prerelease: false },
      { tag: "v1.1.0", name: null, publishedAt: null, body: "", prerelease: false },
    ]);
    show(null);
    expect(await screen.findByText("v1.2.0 · Nov 21, 2026")).toBeInTheDocument();
    expect(screen.getByRole("heading", { level: 3, name: "Library" })).toBeInTheDocument();
    expect(screen.getByText("feat(ui): Faster startup")).toBeInTheDocument();
    expect(screen.getByText("fix: Crisper text")).toBeInTheDocument();
    expect(screen.getByRole("heading", { level: 3, name: "v1.1.0" })).toBeInTheDocument();
    expect(releases.gameReleases).toHaveBeenCalledWith(eternia.repository);
  });

  it("says so when there is no release or GitHub can't be read", async () => {
    show(null);
    expect(await screen.findByText("No release has been published yet.")).toBeInTheDocument();
    cleanup();
    releases.gameReleases.mockRejectedValue("offline");
    show(null);
    expect(await screen.findByText("The releases can't be read right now.")).toBeInTheDocument();
  });
});

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
    expect(api.startGame).toHaveBeenCalledWith("toe", "PS1");
  });

  it("opens the game's settings once the game is installed, not before", async () => {
    show(null);
    expect(screen.getByRole("button", { name: "Game settings" })).toBeDisabled();
    cleanup();
    show(installed(null));
    await fireEvent.click(screen.getByRole("button", { name: "Game settings" }));
    expect(onsettings).toHaveBeenCalledOnce();
  });

  it("says an update needs a new build, and offers Rebuild when the instance kept what it needs", () => {
    show(installed("needsRebuild", { keepsCopy: true }));
    expect(screen.getByText("The game needs a new build after the update.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Rebuild" })).toBeInTheDocument();
  });

  it("deletes the build of a prepared instance from the main button's menu, only after the player confirms", async () => {
    const library = show(installed("ready"));
    api.librarySnapshot.mockResolvedValue([installed(null)]);
    api.deleteBuild.mockResolvedValue(undefined);

    api.confirmDeletion.mockResolvedValue(false);
    await fireEvent.click(screen.getByRole("button", { name: "More actions" }));
    await fireEvent.click(screen.getByRole("menuitem", { name: "Delete the build" }));
    await waitFor(() => expect(api.confirmDeletion).toHaveBeenCalledTimes(1));
    expect(api.deleteBuild).not.toHaveBeenCalled();

    api.confirmDeletion.mockResolvedValue(true);
    await fireEvent.click(screen.getByRole("button", { name: "More actions" }));
    await fireEvent.click(screen.getByRole("menuitem", { name: "Delete the build" }));
    await waitFor(() => expect(api.deleteBuild).toHaveBeenCalledWith("toe", "PS1"));
    await waitFor(() => expect(library.status?.text).toMatch(/build of .* was deleted/));
  });

  it("offers no build to delete before the game is prepared", () => {
    show(installed(null));
    expect(screen.queryByRole("button", { name: "More actions" })).not.toBeInTheDocument();
  });
});

describe("HomeView status", () => {
  it("shows progress with a way to cancel while a job runs", async () => {
    const library = show(installed(null));
    library.progress["prepare:toe/ps1"] = { job: "prepare:toe/ps1", fraction: 0.4, message: "Compiling" };
    expect(await screen.findByText("Compiling · 40%")).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "40");
    expect(screen.getByRole("button", { name: "Choose game file" })).toBeDisabled();
    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(api.cancelJob).toHaveBeenCalledWith("prepare:toe/ps1");
  });

  it("lists the tasks of the running step with their progress", async () => {
    const library = show(installed(null));
    const job = "prepare:toe/ps1";
    library.progress[job] = { job, fraction: 0.6, message: "Compiling the game" };
    library.tasks[job] = [
      { id: "verify", title: "Checking the discs", index: 1, count: 3, state: "ok", fraction: 1, text: "", seconds: 36.6 },
      { id: "extract", title: "Copying the game files", index: 2, count: 3, state: "skipped", fraction: null, text: "", seconds: 0 },
      { id: "compile", title: "Compiling the game", index: 3, count: 3, state: "running", fraction: 0.25, text: "12/48 files", seconds: null },
    ];
    const list = await screen.findByRole("list", { name: "Tasks" });
    const items = Array.from(list.querySelectorAll("li"));
    expect(items).toHaveLength(3);
    expect(items[0]).toHaveTextContent("1/3 Checking the discs");
    expect(items[0]).toHaveTextContent("37 s");
    expect(items[1]).toHaveTextContent("Skipped");
    expect(items[2]).toHaveTextContent("25%");
    expect(items[2]).toHaveTextContent("12/48 files");
    expect(screen.getByRole("progressbar", { name: "Compiling the game" })).toHaveAttribute("aria-valuenow", "25");
  });

  it("shows the preparation's log live while it runs, and keeps the last lines when it ends", async () => {
    const library = show(installed(null));
    const job = "prepare:toe/ps1";
    library.progress[job] = { job, fraction: 0.4, message: "Compiling" };
    api.readInstanceLog.mockResolvedValue("stage-start compile 4/5 Building the game");
    await fireEvent.click(await screen.findByRole("button", { name: "Live log" }));
    expect(await screen.findByLabelText("Log")).toHaveTextContent("stage-start compile");
    expect(api.readInstanceLog).toHaveBeenCalledWith("toe", "PS1", "prepare");
    expect(screen.getByRole("button", { name: "Hide live log" })).toHaveAttribute("aria-pressed", "true");

    api.readInstanceLog.mockResolvedValue("stage-start compile 4/5 Building the game\nresult ready 0 done");
    delete library.progress[job];
    await waitFor(() => expect(screen.getByLabelText("Log")).toHaveTextContent("result ready 0 done"));
    expect(screen.queryByRole("button", { name: /live log/i })).not.toBeInTheDocument();
  });

  it("shows the last result and the logs on demand", async () => {
    const library = show(installed("ready"));
    library.status = { tone: "error", text: "The preparation failed (exit code 7)." };
    expect(await screen.findByText("The preparation failed (exit code 7).")).toBeInTheDocument();

    api.readInstanceLog.mockResolvedValue("progress 0.1 Reading\ndisc is unreadable");
    await fireEvent.click(screen.getByRole("button", { name: "Preparation log" }));
    expect(await screen.findByLabelText("Log")).toHaveTextContent("disc is unreadable");
    expect(api.readInstanceLog).toHaveBeenCalledWith("toe", "PS1", "prepare");
    await fireEvent.click(screen.getByRole("button", { name: "Game log" }));
    expect(api.readInstanceLog).toHaveBeenLastCalledWith("toe", "PS1", "game");
  });
});

describe("HomeView availability", () => {
  it.each([
    ["planned", "This console is planned: nothing to install yet."],
    ["in-progress", "This console is in progress: there is nothing to install yet."],
    ["unsupported", "This console isn't supported."],
  ] as const)("can't install or prepare a %s console, and says why", (state, hint) => {
    const library = new LibraryStore();
    library.installations = [installed(null)];
    render(HomeView, { game: eternia, platform: { ...platform, state, maturity: null }, library, platformSizing: [], onplatform: () => {}, onsettings });
    expect(screen.getByText(hint)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose game file" })).toBeDisabled();
  });

  it("lists consoles only in the selector, each with the year the game came out on it", async () => {
    const library = new LibraryStore();
    const game = { ...eternia, platforms: [platform, { platform: "Windows", regions: ["Worldwide"], state: "planned" as const, maturity: null, releaseYear: 2012, year: 2012 }] };
    render(HomeView, { game, platform, library, platformSizing: [], onplatform: () => {}, onsettings });
    await fireEvent.click(screen.getByRole("combobox", { name: "Platform" }));
    expect(screen.getAllByRole("option").map((o) => o.textContent?.trim())).toEqual(["2000 · PS1", "2012 · Windows"]);
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

    library.progress["game:toe/ps1"] = { job: "game:toe/ps1", fraction: null, message: "running" };
    await waitFor(() => expect(screen.queryByRole("button", { name: "Create diagnostic file" })).not.toBeInTheDocument());
  });
});
