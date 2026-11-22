import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Catalog } from "./lib/catalog/types";

const api = vi.hoisted(() => ({ cachedCatalog: vi.fn(), refreshCatalog: vi.fn() }));
vi.mock("./lib/catalog/api", () => api);

const releases = vi.hoisted(() => ({ gameReleases: vi.fn() }));
vi.mock("./lib/catalog/releases", async (orig) => ({ ...(await orig<object>()), ...releases }));

const library = vi.hoisted(() => ({
  librarySnapshot: vi.fn(),
  instanceLocation: vi.fn(),
  onProgress: vi.fn(),
  onGameExited: vi.fn(),
}));
vi.mock("./lib/library/api", () => library);

const diagnostics = vi.hoisted(() => ({
  pendingPanic: vi.fn(),
  dismissPanic: vi.fn(),
  copyText: vi.fn(),
  buildReport: vi.fn(),
  revealFile: vi.fn(),
  openIssues: vi.fn(),
}));
vi.mock("./lib/diagnostics/api", () => diagnostics);

const launcher = vi.hoisted(() => ({ loadLauncherSettings: vi.fn(), setLauncherSetting: vi.fn(), launcherFolders: vi.fn(), setGamesFolder: vi.fn() }));
vi.mock("./lib/launcher/api", () => launcher);

import App from "./App.svelte";

const game = (id: string, name: string, year: number, platforms: Catalog["games"][0]["platforms"]) => ({
  id, name, japanReleaseYear: year, repository: `To-Recomp/${id}`, platforms,
});
const catalog: Catalog = {
  games: [
    game("toe", "Tales of Eternia", 2000, [{ platform: "PS1", regions: ["USA"], state: "playable", maturity: "alpha" }]),
    game("tos", "Tales of Symphonia", 2003, [
      { platform: "GameCube", regions: ["France (PAL)", "Japan"], state: "playable", maturity: "beta" },
      { platform: "PlayStation 2", regions: ["Japan"], state: "planned", maturity: null },
    ]),
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  releases.gameReleases.mockResolvedValue([]);
  api.cachedCatalog.mockResolvedValue(null);
  api.refreshCatalog.mockResolvedValue({ catalog, updated: true, error: null });
  library.librarySnapshot.mockResolvedValue([]);
  library.instanceLocation.mockResolvedValue(null);
  library.onProgress.mockResolvedValue(() => {});
  library.onGameExited.mockResolvedValue(() => {});
  diagnostics.pendingPanic.mockResolvedValue(null);
  diagnostics.dismissPanic.mockResolvedValue(undefined);
});

describe("App", () => {
  it("builds the sidebar from the catalog, in its order, and opens on the first game", async () => {
    render(App);
    const nav = await screen.findByRole("navigation", { name: "Games" });
    const items = await within(nav).findAllByRole("button");
    expect(items.map((b) => b.querySelector(".name")?.textContent)).toEqual(["Tales of Eternia", "Tales of Symphonia"]);
    expect(screen.getByRole("heading", { level: 1, name: "Eternia" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Tales of Eternia/ })).toHaveAttribute("aria-current", "true");
  });

  it("lists consoles in the selector, not their regions", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /Tales of Symphonia/ }));
    await fireEvent.click(screen.getByRole("combobox", { name: "Platform" }));
    expect(screen.getAllByRole("option").map((o) => o.textContent?.trim())).toEqual(["GameCube", "PlayStation 2"]);
  });

  it("shows the saved list at once, then the downloaded one", async () => {
    api.cachedCatalog.mockResolvedValue({ games: [catalog.games[0]] });
    let release: (v: unknown) => void = () => {};
    api.refreshCatalog.mockReturnValue(new Promise((resolve) => (release = resolve)));
    render(App);
    expect(await screen.findByRole("button", { name: /Tales of Eternia/ })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Tales of Symphonia/ })).not.toBeInTheDocument();
    release({ catalog, updated: true, error: null });
    expect(await screen.findByRole("button", { name: /Tales of Symphonia/ })).toBeInTheDocument();
  });

  it("keeps the saved list and says why when the download fails", async () => {
    api.cachedCatalog.mockResolvedValue(catalog);
    api.refreshCatalog.mockResolvedValue({ catalog, updated: false, error: "Can't reach the game catalog: offline" });
    render(App);
    const sidebar = within(screen.getByRole("complementary"));
    expect(await sidebar.findByRole("status")).toHaveTextContent("Showing the saved game list. Can't reach the game catalog: offline");
    expect(screen.getByRole("button", { name: /Tales of Eternia/ })).toBeInTheDocument();
  });

  it("explains an empty first run and retries on demand", async () => {
    api.refreshCatalog.mockResolvedValueOnce({ catalog: null, updated: false, error: "Can't reach the game catalog: offline" });
    render(App);
    expect(await screen.findByText("The game list isn't available yet.")).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("Can't reach the game catalog: offline");
    await fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(await screen.findByRole("button", { name: /Tales of Symphonia/ })).toBeInTheDocument();
    expect(api.refreshCatalog).toHaveBeenCalledTimes(2);
  });

  it("falls back to the first game when the selected one leaves the catalog", async () => {
    api.refreshCatalog.mockResolvedValueOnce({ catalog, updated: false, error: "Can't reach the game catalog: offline" });
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /Tales of Symphonia/ }));
    expect(screen.getByRole("heading", { level: 1, name: "Symphonia" })).toBeInTheDocument();

    api.refreshCatalog.mockResolvedValue({ catalog: { games: [catalog.games[0]] }, updated: true, error: null });
    await fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Eternia" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Tales of Symphonia/ })).not.toBeInTheDocument();
  });

  it("shows what is installed in the sidebar and finds the settings of the selected platform", async () => {
    library.librarySnapshot.mockResolvedValue([
      { gameId: "tos", repository: "To-Recomp/tos", tag: "v1", inputs: [], prepareLabel: null, canPrepare: false, keepsCopy: false,
        instances: [{ id: "gamecube", platform: "GameCube", state: "ready" }] },
    ]);
    library.instanceLocation.mockImplementation(async (game: string, platform: string) =>
      game === "tos" ? { installDir: `/lib/${game}/release`, dataDir: `/lib/${game}/${platform}` } : null);
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /Tales of Symphonia/ }));
    expect(screen.getByRole("img", { name: "Play" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Install" })).toBeInTheDocument(); // Eternia
    expect(library.instanceLocation).toHaveBeenCalledWith("tos", "GameCube");

    await fireEvent.click(screen.getByRole("combobox", { name: "Platform" }));
    await fireEvent.click(screen.getByRole("option", { name: "PlayStation 2" }));
    await waitFor(() => expect(library.instanceLocation).toHaveBeenLastCalledWith("tos", "PlayStation 2"));
  });

  it("does not expose the hidden Mods tab and switches between Home and Settings", async () => {
    render(App);
    await screen.findByRole("heading", { level: 1, name: "Eternia" });
    expect(screen.queryByRole("tab", { name: "Mods" })).not.toBeInTheDocument();
    await fireEvent.click(screen.getByRole("tab", { name: "Settings" }));
    expect(screen.getByRole("heading", { level: 1, name: "Settings" })).toBeInTheDocument();
    expect(screen.getByText("Install and prepare Tales of Eternia to configure it.")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("tab", { name: "Home" }));
    expect(screen.getByText("Action status")).toBeInTheDocument();
  });

  it("opens the launcher's own settings from the gear, even before the game list loads, and leaves them when a game is picked", async () => {
    api.refreshCatalog.mockResolvedValue({ catalog: null, updated: false, error: "Can't reach the game catalog: offline" });
    launcher.loadLauncherSettings.mockResolvedValue({ settings: [] });
    launcher.launcherFolders.mockResolvedValue({ configured: null, default: "/data/games", inUse: "/data/games" });
    render(App);
    const gear = await screen.findByRole("button", { name: "Launcher settings" });
    await fireEvent.click(gear);
    expect(screen.getByRole("heading", { level: 1, name: "Launcher settings" })).toBeInTheDocument();
    expect(gear).toHaveAttribute("aria-pressed", "true");

    api.refreshCatalog.mockResolvedValue({ catalog, updated: true, error: null });
    await fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    await fireEvent.click(await screen.findByRole("button", { name: /Tales of Symphonia/ }));
    expect(screen.getByRole("heading", { level: 1, name: "Symphonia" })).toBeInTheDocument();
    expect(gear).toHaveAttribute("aria-pressed", "false");
  });

  it("always shows the launcher version and the non-affiliation notice", async () => {
    render(App);
    expect(within(screen.getByRole("complementary")).getByText("v0.1.0")).toBeInTheDocument();
    expect(screen.getByText(/Not affiliated with or endorsed by/)).toBeInTheDocument();
  });
});

describe("launcher crash banner", () => {
  const file = { name: "launcher-diagnostic-x.md", path: "/logs/x.md", folder: "/logs", text: "Diagnostic format: 1\n" };

  it("is absent after a normal run", async () => {
    render(App);
    await screen.findByText("Tales of Eternia", { selector: "h1" }).catch(() => {});
    expect(screen.queryByText("The launcher stopped unexpectedly")).not.toBeInTheDocument();
  });

  it("offers the file of the previous run's panic, opens it for review and goes away when dismissed", async () => {
    diagnostics.pendingPanic.mockResolvedValue(file);
    render(App);
    expect(await screen.findByText("The launcher stopped unexpectedly")).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "View file" }));
    expect(screen.getByRole("dialog")).toHaveTextContent("Review this file before you publish it: it will be public.");
    expect(screen.getByLabelText("File content").textContent).toBe(file.text);
    await fireEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Open issues" }));
    expect(diagnostics.openIssues).toHaveBeenCalledWith("To-Recomp/launcher");
    await fireEvent.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Close" }));

    await fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(diagnostics.dismissPanic).toHaveBeenCalledOnce();
    expect(screen.queryByText("The launcher stopped unexpectedly")).not.toBeInTheDocument();
  });
});
