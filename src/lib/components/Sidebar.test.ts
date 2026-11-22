import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { eternia } from "./fixtures";

const updater = vi.hoisted(() => ({ updaterAvailable: vi.fn(), updateLauncher: vi.fn() }));
vi.mock("../updater", () => updater);

import Sidebar from "./Sidebar.svelte";

const props = { games: [eternia], selectedId: "toe", onselect: () => {} };

beforeEach(() => vi.clearAllMocks());

describe("Sidebar update button", () => {
  it("is not offered when this installation can't update itself", async () => {
    updater.updaterAvailable.mockResolvedValue(false);
    render(Sidebar, props);
    await vi.waitFor(() => expect(updater.updaterAvailable).toHaveBeenCalled());
    expect(screen.queryByRole("button", { name: "Update launcher" })).not.toBeInTheDocument();
  });

  it("reports an up-to-date launcher", async () => {
    updater.updaterAvailable.mockResolvedValue(true);
    updater.updateLauncher.mockResolvedValue(undefined);
    render(Sidebar, props);
    await fireEvent.click(await screen.findByRole("button", { name: "Update launcher" }));
    expect(await screen.findByRole("button", { name: "Up to date" })).toBeInTheDocument();
  });

  it("shows why an update failed", async () => {
    updater.updaterAvailable.mockResolvedValue(true);
    updater.updateLauncher.mockRejectedValue("Update check failed: offline");
    render(Sidebar, props);
    await fireEvent.click(await screen.findByRole("button", { name: "Update launcher" }));
    expect(await screen.findByRole("button", { name: "Update failed" })).toHaveAttribute("title", "Update check failed: offline");
  });
});
