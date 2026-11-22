import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const updater = vi.hoisted(() => ({
  updaterAvailable: vi.fn(),
  updateLauncher: vi.fn(),
  checkLauncherUpdate: vi.fn(),
  updatePreferences: vi.fn(),
}));
vi.mock("../updater", () => updater);
vi.mock("../settings/api", () => ({ errorMessage: (e: unknown) => String(e) }));

import { launcherUpdates } from "../launcher/updates.svelte";
import LauncherUpdates from "./LauncherUpdates.svelte";

beforeEach(() => {
  vi.clearAllMocks();
  launcherUpdates.available = true;
  launcherUpdates.found = null;
});

describe("LauncherUpdates", () => {
  it("is not shown where the launcher can't update itself", () => {
    launcherUpdates.available = false;
    render(LauncherUpdates);
    expect(screen.queryByRole("button", { name: "Check now" })).not.toBeInTheDocument();
  });

  it("says the launcher is up to date", async () => {
    updater.checkLauncherUpdate.mockResolvedValue(null);
    render(LauncherUpdates);
    await fireEvent.click(screen.getByRole("button", { name: "Check now" }));
    expect(await screen.findByRole("status")).toHaveTextContent("You have the latest version.");
    expect(screen.queryByRole("button", { name: "Install and restart" })).not.toBeInTheDocument();
  });

  it("offers to install the release it found", async () => {
    updater.checkLauncherUpdate.mockResolvedValue({ version: "0.2.0" });
    updater.updateLauncher.mockResolvedValue(undefined);
    render(LauncherUpdates);
    await fireEvent.click(screen.getByRole("button", { name: "Check now" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Install and restart" }));
    expect(updater.updateLauncher).toHaveBeenCalledOnce();
  });

  it("shows why a check failed", async () => {
    updater.checkLauncherUpdate.mockRejectedValue("Update check failed: offline");
    render(LauncherUpdates);
    await fireEvent.click(screen.getByRole("button", { name: "Check now" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Update check failed: offline");
  });
});
