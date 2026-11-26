import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const saves = vi.hoisted(() => ({
  listSaves: vi.fn(),
  exportSaves: vi.fn(),
  importSaves: vi.fn(),
  deleteSave: vi.fn(),
  pickBackupTarget: vi.fn(),
  pickBackup: vi.fn(),
}));
vi.mock("../settings/saves", () => saves);

const links = vi.hoisted(() => ({ revealFile: vi.fn() }));
vi.mock("../diagnostics/api", () => links);

import GameSaves from "./GameSaves.svelte";

const location = { installDir: "/games/tos", dataDir: "/games/tos/data" };
const folder = {
  path: "/games/tos/data/saves",
  exists: true,
  files: [
    { name: "card.raw", size: 2048, modified: 1_700_000_000 },
    { name: "sub/slot.raw", size: 12, modified: null },
  ],
};

beforeEach(() => {
  vi.clearAllMocks();
  saves.listSaves.mockResolvedValue({ folders: [folder] });
});

describe("GameSaves", () => {
  it("lists each save file with its size and opens the folder", async () => {
    render(GameSaves, { location, gameId: "tos" });
    expect(await screen.findByText("card.raw")).toBeInTheDocument();
    expect(screen.getByText(/^2\.0 KB · /)).toBeInTheDocument();
    expect(screen.getByText("12 B")).toBeInTheDocument();
    await fireEvent.click(screen.getAllByRole("button", { name: "Open" })[0]);
    expect(links.revealFile).toHaveBeenCalledWith("/games/tos/data/saves");
  });

  it("deletes a file only after a second click, then refreshes the list", async () => {
    render(GameSaves, { location, gameId: "tos" });
    await screen.findByText("card.raw");
    saves.listSaves.mockResolvedValue({ folders: [{ ...folder, files: [folder.files[1]] }] });
    const [first] = screen.getAllByRole("button", { name: "Delete" });
    await fireEvent.click(first);
    expect(saves.deleteSave).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("button", { name: /^Delete\?/ }));
    expect(saves.deleteSave).toHaveBeenCalledWith(location, 0, "card.raw");
    await waitFor(() => expect(screen.queryByText("card.raw")).not.toBeInTheDocument());
  });

  it("backs up to the chosen file and says how many files it holds; closing the dialog does nothing", async () => {
    render(GameSaves, { location, gameId: "tos" });
    await screen.findByText("card.raw");
    saves.pickBackupTarget.mockResolvedValueOnce(null);
    await fireEvent.click(screen.getByRole("button", { name: "Back up…" }));
    await waitFor(() => expect(saves.pickBackupTarget).toHaveBeenCalled());
    expect(saves.exportSaves).not.toHaveBeenCalled();

    saves.pickBackupTarget.mockResolvedValueOnce("/tmp/b.tar.gz");
    saves.exportSaves.mockResolvedValueOnce(2);
    await fireEvent.click(screen.getByRole("button", { name: "Back up…" }));
    expect(await screen.findByText("2 files saved.")).toBeInTheDocument();
    expect(saves.exportSaves).toHaveBeenCalledWith(location, "/tmp/b.tar.gz");
  });

  it("restores a backup, refreshes the list, and shows a refusal", async () => {
    render(GameSaves, { location, gameId: "tos" });
    await screen.findByText("card.raw");
    saves.pickBackup.mockResolvedValue("/tmp/b.tar.gz");
    saves.importSaves.mockRejectedValueOnce("This isn't a valid backup: nope");
    await fireEvent.click(screen.getByRole("button", { name: "Restore…" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("This isn't a valid backup");

    saves.importSaves.mockResolvedValueOnce(2);
    await fireEvent.click(screen.getByRole("button", { name: "Restore…" }));
    expect(await screen.findByText("2 files restored.")).toBeInTheDocument();
    expect(saves.listSaves).toHaveBeenCalledTimes(2);
  });

  it("says there are no saves yet", async () => {
    saves.listSaves.mockResolvedValue({ folders: [{ ...folder, exists: false, files: [] }] });
    render(GameSaves, { location, gameId: "tos" });
    expect(await screen.findByText(/No saves yet/)).toBeInTheDocument();
  });
});
