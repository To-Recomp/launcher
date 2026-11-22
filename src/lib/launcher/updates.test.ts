import { beforeEach, describe, expect, it, vi } from "vitest";

const updater = vi.hoisted(() => ({
  updaterAvailable: vi.fn(),
  updateLauncher: vi.fn(),
  checkLauncherUpdate: vi.fn(),
  updatePreferences: vi.fn(),
}));
vi.mock("../updater", () => updater);

import { LauncherUpdates } from "./updates.svelte";

beforeEach(() => {
  vi.clearAllMocks();
  updater.updaterAvailable.mockResolvedValue(true);
  updater.updatePreferences.mockResolvedValue({ auto: true, prerelease: false });
  updater.checkLauncherUpdate.mockResolvedValue({ version: "0.2.0" });
});

describe("LauncherUpdates", () => {
  it("checks when it starts if the settings ask for it, and keeps what it found", async () => {
    const updates = new LauncherUpdates();
    await updates.start();
    expect(updates.available).toBe(true);
    expect(updates.found).toEqual({ version: "0.2.0" });
  });

  it("doesn't check at startup when automatic checks are off", async () => {
    updater.updatePreferences.mockResolvedValue({ auto: false, prerelease: false });
    const updates = new LauncherUpdates();
    await updates.start();
    expect(updater.checkLauncherUpdate).not.toHaveBeenCalled();
    expect(updates.found).toBeNull();
  });

  it("never checks where the launcher can't update itself", async () => {
    updater.updaterAvailable.mockResolvedValue(false);
    const updates = new LauncherUpdates();
    await updates.start();
    expect(updates.available).toBe(false);
    expect(updater.updatePreferences).not.toHaveBeenCalled();
    expect(updater.checkLauncherUpdate).not.toHaveBeenCalled();
  });

  it("stays quiet when the startup check fails, but a manual check says why", async () => {
    updater.checkLauncherUpdate.mockRejectedValue("Update check failed: offline");
    const updates = new LauncherUpdates();
    await expect(updates.start()).resolves.toBeUndefined();
    expect(updates.found).toBeNull();
    await expect(updates.check()).rejects.toBe("Update check failed: offline");
  });

  it("forgets a release found on the other channel", async () => {
    const updates = new LauncherUpdates();
    await updates.start();
    updates.forget();
    expect(updates.found).toBeNull();
  });
});
