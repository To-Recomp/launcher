import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SettingView, SettingsView as View } from "../settings/types";

const api = vi.hoisted(() => ({ loadLauncherSettings: vi.fn(), setLauncherSetting: vi.fn(), launcherFolders: vi.fn(), setGamesFolder: vi.fn() }));
vi.mock("../launcher/api", () => api);
vi.mock("../updater", () => ({ updaterAvailable: vi.fn(), updateLauncher: vi.fn(), checkLauncherUpdate: vi.fn(), updatePreferences: vi.fn() }));
vi.mock("../settings/api", () => ({ errorMessage: (e: unknown) => String(e) }));

import { launcherUpdates } from "../launcher/updates.svelte";
import LauncherSettingsView from "./LauncherSettingsView.svelte";

const toggle = (value: number): SettingView => ({
  id: "autoUpdate", label: "Check for updates", help: null, tab: "Game", group: "Updates", advanced: false, enabled: true,
  control: { type: "choices", style: "toggle", options: ["Off", "On"], value, custom: null },
});
const view = (...settings: SettingView[]): View => ({ settings });

const onreportbug = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  api.loadLauncherSettings.mockResolvedValue(view(toggle(1)));
  api.launcherFolders.mockResolvedValue({ configured: null, default: "/data/games", inUse: "/data/games" });
});

describe("LauncherSettingsView", () => {
  it("shows the launcher's settings under their group heading", async () => {
    render(LauncherSettingsView, { onreportbug });
    expect(await screen.findByText("Check for updates")).toBeInTheDocument();
    expect(screen.getByText("Updates")).toBeInTheDocument();
    expect(screen.getByRole("switch")).toBeChecked();
  });

  it("writes a change and shows the refreshed value", async () => {
    api.setLauncherSetting.mockResolvedValue(view(toggle(0)));
    render(LauncherSettingsView, { onreportbug });
    await fireEvent.click(await screen.findByRole("switch"));
    expect(api.setLauncherSetting).toHaveBeenCalledWith("autoUpdate", { type: "choice", index: 0 });
    await waitFor(() => expect(screen.getByRole("switch")).not.toBeChecked());
  });

  it("explains a rejected change and shows what is really saved", async () => {
    api.setLauncherSetting.mockRejectedValue("Check for updates: choice 7 doesn't exist.");
    render(LauncherSettingsView, { onreportbug });
    await fireEvent.click(await screen.findByRole("switch"));
    expect(await screen.findByRole("alert")).toHaveTextContent("choice 7 doesn't exist");
    await waitFor(() => expect(api.loadLauncherSettings).toHaveBeenCalledTimes(2));
    expect(screen.getByRole("switch")).toBeChecked();
  });

  it("says why the settings can't be shown", async () => {
    api.loadLauncherSettings.mockRejectedValue("Can't read launcher.json: denied");
    render(LauncherSettingsView, { onreportbug });
    expect(await screen.findByRole("alert")).toHaveTextContent("Can't read launcher.json: denied");
  });

  it("forgets a release found on the other channel when the channel changes", async () => {
    const channel: SettingView = {
      id: "updateChannel", label: "Releases", help: null, tab: "Game", group: "Updates", advanced: false, enabled: true,
      control: { type: "choices", style: "segmented", options: ["Stable", "Pre-release"], value: 0, custom: null },
    };
    api.loadLauncherSettings.mockResolvedValue(view(channel));
    api.setLauncherSetting.mockResolvedValue(view(channel));
    launcherUpdates.found = { version: "0.3.0-beta.1" };
    render(LauncherSettingsView, { onreportbug });
    await fireEvent.click(await screen.findByRole("radio", { name: "Pre-release" }));
    await waitFor(() => expect(launcherUpdates.found).toBeNull());
  });
});
