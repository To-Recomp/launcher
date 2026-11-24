import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { eternia } from "./fixtures";

const updater = vi.hoisted(() => ({
  updaterAvailable: vi.fn(),
  updateLauncher: vi.fn(),
  checkLauncherUpdate: vi.fn(),
  updatePreferences: vi.fn(),
}));
vi.mock("../updater", () => updater);

import { LauncherUpdates } from "../launcher/updates.svelte";
import Sidebar from "./Sidebar.svelte";

const onlauncher = vi.fn();
const onplay = vi.fn();
const props = { games: [eternia], selectedId: "toe", onselect: () => {}, onplay, onlauncher };

async function started() {
  const updates = new LauncherUpdates();
  await updates.start();
  return { ...props, updates };
}

beforeEach(() => {
  vi.clearAllMocks();
  updater.updatePreferences.mockResolvedValue({ auto: false, prerelease: false });
});

describe("Sidebar update button", () => {
  it("is not offered when this installation can't update itself", async () => {
    updater.updaterAvailable.mockResolvedValue(false);
    render(Sidebar, await started());
    expect(screen.queryByRole("button", { name: "Update launcher" })).not.toBeInTheDocument();
  });

  it("reports an up-to-date launcher", async () => {
    updater.updaterAvailable.mockResolvedValue(true);
    updater.updateLauncher.mockResolvedValue(undefined);
    render(Sidebar, await started());
    await fireEvent.click(await screen.findByRole("button", { name: "Update launcher" }));
    expect(await screen.findByRole("button", { name: "Up to date" })).toBeInTheDocument();
  });

  it("shows why an update failed", async () => {
    updater.updaterAvailable.mockResolvedValue(true);
    updater.updateLauncher.mockRejectedValue("Update check failed: offline");
    render(Sidebar, await started());
    await fireEvent.click(await screen.findByRole("button", { name: "Update launcher" }));
    expect(await screen.findByRole("button", { name: "Update failed" })).toHaveAttribute("title", "Update check failed: offline");
  });

  it("names the newer release the startup check found", async () => {
    updater.updaterAvailable.mockResolvedValue(true);
    updater.updatePreferences.mockResolvedValue({ auto: true, prerelease: false });
    updater.checkLauncherUpdate.mockResolvedValue({ version: "0.2.0" });
    render(Sidebar, await started());
    expect(await screen.findByRole("button", { name: "Update to v0.2.0" })).toBeInTheDocument();
  });
});

describe("Sidebar footer and list", () => {
  it("opens the launcher's settings from the gear next to the version", async () => {
    updater.updaterAvailable.mockResolvedValue(false);
    render(Sidebar, await started());
    await fireEvent.click(screen.getByRole("button", { name: "Launcher settings" }));
    expect(onlauncher).toHaveBeenCalledOnce();
  });

  it("gives a quick play button only to a game that can be played at once", async () => {
    updater.updaterAvailable.mockResolvedValue(false);
    const ready = { ...eternia, id: "tos", name: "Tales of Symphonia", state: "ready" as const };
    render(Sidebar, { ...(await started()), games: [eternia, { ...eternia, id: "tov", name: "Tales of Vesperia", state: "installed" as const }, ready] });
    expect(screen.getAllByRole("button", { name: /^Play / })).toHaveLength(1);
    await fireEvent.click(screen.getByRole("button", { name: "Play Tales of Symphonia" }));
    expect(onplay).toHaveBeenCalledWith("tos");
  });
});
