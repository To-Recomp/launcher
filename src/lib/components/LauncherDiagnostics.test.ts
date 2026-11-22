import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({ diagnosticsFolder: vi.fn(), cleanDiagnostics: vi.fn(), revealFile: vi.fn() }));
vi.mock("../diagnostics/api", () => api);
vi.mock("../settings/api", () => ({ errorMessage: (e: unknown) => String(e) }));

import LauncherDiagnostics from "./LauncherDiagnostics.svelte";

const oncleaned = vi.fn();

beforeEach(() => vi.clearAllMocks());

describe("LauncherDiagnostics", () => {
  it("opens the reports folder", async () => {
    api.diagnosticsFolder.mockResolvedValue("/logs/launcher");
    render(LauncherDiagnostics, { oncleaned });
    await fireEvent.click(screen.getByRole("button", { name: "Open" }));
    await waitFor(() => expect(api.revealFile).toHaveBeenCalledWith("/logs/launcher"));
  });

  it("asks before deleting, then says what was deleted and tells the page", async () => {
    api.cleanDiagnostics.mockResolvedValue({ files: 3, bytes: 2048 });
    render(LauncherDiagnostics, { oncleaned });
    await fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    expect(api.cleanDiagnostics).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("button", { name: "Delete them" }));
    expect(await screen.findByRole("status")).toHaveTextContent("Deleted 3 files (2 KB).");
    expect(oncleaned).toHaveBeenCalledOnce();
  });

  it("says so when there was nothing to delete, and why a deletion failed", async () => {
    api.cleanDiagnostics.mockResolvedValueOnce({ files: 0, bytes: 0 });
    render(LauncherDiagnostics, { oncleaned });
    await fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    await fireEvent.click(screen.getByRole("button", { name: "Delete them" }));
    expect(await screen.findByRole("status")).toHaveTextContent("No reports to delete.");

    api.cleanDiagnostics.mockRejectedValueOnce("No place to keep the diagnostic files");
    await fireEvent.click(screen.getByRole("button", { name: "Delete…" }));
    await fireEvent.click(screen.getByRole("button", { name: "Delete them" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("No place to keep the diagnostic files");
  });
});
