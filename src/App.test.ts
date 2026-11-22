import { fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Catalog } from "./lib/catalog/types";

const api = vi.hoisted(() => ({ cachedCatalog: vi.fn(), refreshCatalog: vi.fn() }));
vi.mock("./lib/catalog/api", () => api);

const library = vi.hoisted(() => ({
  librarySnapshot: vi.fn(),
  instanceLocation: vi.fn(),
  onProgress: vi.fn(),
  onGameExited: vi.fn(),
}));
vi.mock("./lib/library/api", () => library);

import App from "./App.svelte";

const game = (id: string, name: string, year: number, platforms: Catalog["games"][0]["platforms"]) => ({
  id, name, japanReleaseYear: year, repository: `To-Recomp/${id}`, manifest: null, platforms,
});
const catalog: Catalog = {
  games: [
    game("toe", "Tales of Eternia", 2000, [{ platform: "PS1", region: "USA", status: "alpha", tested: true }]),
    game("tos", "Tales of Symphonia", 2003, [
      { platform: "GameCube", region: "France (PAL)", status: "unsupported", tested: false },
      { platform: "GameCube", region: "Japan", status: "beta", tested: false },
    ]),
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  api.cachedCatalog.mockResolvedValue(null);
  api.refreshCatalog.mockResolvedValue({ catalog, updated: true, error: null });
  library.librarySnapshot.mockResolvedValue([]);
  library.instanceLocation.mockResolvedValue(null);
  library.onProgress.mockResolvedValue(() => {});
  library.onGameExited.mockResolvedValue(() => {});
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

  it("shows each platform's status and tells regions apart in the selector", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /Tales of Symphonia/ }));
    expect(screen.getAllByTitle(/GameCube \(France \(PAL\)\): unsupported, untested/)).toHaveLength(1);
    expect(screen.getAllByTitle("GameCube (Japan): beta, untested")).toHaveLength(1);
    const platform = screen.getByRole("combobox", { name: "Platform" }) as HTMLSelectElement;
    expect([...platform.options].map((o) => o.value)).toEqual(["GameCube · France (PAL)", "GameCube · Japan"]);
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
        instances: [{ id: "gamecube-japan", platform: "GameCube", region: "Japan", state: "ready" }] },
    ]);
    library.instanceLocation.mockImplementation(async (game: string, platform: string, region: string) =>
      game === "tos" ? { installDir: `/lib/${game}/release`, dataDir: `/lib/${game}/${platform}-${region}` } : null);
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /Tales of Symphonia/ }));
    expect(screen.getByRole("img", { name: "Play" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Install" })).toBeInTheDocument(); // Eternia
    expect(library.instanceLocation).toHaveBeenCalledWith("tos", "GameCube", "France (PAL)");

    await fireEvent.change(screen.getByRole("combobox", { name: "Platform" }), { target: { value: "GameCube · Japan" } });
    await waitFor(() => expect(library.instanceLocation).toHaveBeenLastCalledWith("tos", "GameCube", "Japan"));
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

  it("always shows the launcher version and the non-affiliation notice", async () => {
    render(App);
    expect(within(screen.getByRole("complementary")).getByText("v0.1.0")).toBeInTheDocument();
    expect(screen.getByText(/Not affiliated with or endorsed by/)).toBeInTheDocument();
  });
});
