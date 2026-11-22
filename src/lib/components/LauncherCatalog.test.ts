import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

vi.mock("../settings/api", () => ({ errorMessage: (e: unknown) => String(e) }));

import LauncherCatalog from "./LauncherCatalog.svelte";

const at = new Date(2026, 10, 22, 14, 32);

describe("LauncherCatalog", () => {
  it("refreshes on demand and is busy while it does", async () => {
    let finish: () => void = () => {};
    const onrefresh = vi.fn(() => new Promise<void>((resolve) => (finish = resolve)));
    render(LauncherCatalog, { status: { at, updated: false, error: null }, refreshing: false, onrefresh });
    expect(screen.getByText(/^Up to date as of/)).toBeInTheDocument();
    const button = screen.getByRole("button", { name: "Refresh" });
    await fireEvent.click(button);
    expect(onrefresh).toHaveBeenCalledOnce();
    expect(button).toBeDisabled();
    finish();
  });

  it("says why the last refresh failed and that the saved list is used", () => {
    render(LauncherCatalog, {
      status: { at, updated: false, error: "Can't reach the game catalog: offline" },
      refreshing: false,
      onrefresh: vi.fn(),
    });
    expect(screen.getByText(/The saved list is used\./)).toBeInTheDocument();
    expect(screen.getByText("Can't reach the game catalog: offline")).toBeInTheDocument();
  });
});
