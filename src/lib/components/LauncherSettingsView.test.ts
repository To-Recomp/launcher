import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SettingView, SettingsView as View } from "../settings/types";

const api = vi.hoisted(() => ({ loadLauncherSettings: vi.fn(), setLauncherSetting: vi.fn(), launcherFolders: vi.fn(), setGamesFolder: vi.fn(), launcherAppearance: vi.fn(), applyBehavior: vi.fn().mockResolvedValue(undefined), applyDownloadLimit: vi.fn().mockResolvedValue(undefined), proxySettings: vi.fn().mockResolvedValue({ settings: { mode: "system", protocol: "http", host: "", port: 8080, username: "", noProxy: "" }, hasPassword: false, keychainError: null }), setProxy: vi.fn(), testProxy: vi.fn(), autostartSupported: vi.fn().mockResolvedValue(false), autostartEnabled: vi.fn(), setAutostart: vi.fn() }));
vi.mock("../launcher/api", () => api);
vi.mock("../updater", () => ({ updaterAvailable: vi.fn(), updateLauncher: vi.fn(), checkLauncherUpdate: vi.fn(), updatePreferences: vi.fn() }));
vi.mock("../settings/api", () => ({ errorMessage: (e: unknown) => String(e) }));

import { appearance } from "../launcher/appearance.svelte";
import { restartNeeded } from "../launcher/restart.svelte";
import { launcherUpdates } from "../launcher/updates.svelte";
import LauncherSettingsView from "./LauncherSettingsView.svelte";

const toggle = (value: number): SettingView => ({
  id: "autoUpdate", label: "Check for updates", help: null, tab: "Game", group: "Updates", advanced: false, enabled: true,
  control: { type: "choices", style: "toggle", options: ["Off", "On"], value, custom: null },
});
const view = (...settings: SettingView[]): View => ({ settings });

const onreportbug = vi.fn();
const props = { catalogStatus: null, refreshingCatalog: false, onrefreshcatalog: vi.fn(), oncleaned: vi.fn(), onreportbug };

beforeEach(() => {
  vi.clearAllMocks();
  api.loadLauncherSettings.mockResolvedValue(view(toggle(1)));
  api.launcherFolders.mockResolvedValue({ configured: null, default: "/data/games", inUse: "/data/games" });
});

describe("LauncherSettingsView", () => {
  it("shows the launcher's settings under their group heading", async () => {
    render(LauncherSettingsView, props);
    expect(await screen.findByText("Check for updates")).toBeInTheDocument();
    expect(screen.getByText("Updates")).toBeInTheDocument();
    expect(screen.getByRole("switch")).toBeChecked();
  });

  it("writes a change and shows the refreshed value", async () => {
    api.setLauncherSetting.mockResolvedValue(view(toggle(0)));
    render(LauncherSettingsView, props);
    await fireEvent.click(await screen.findByRole("switch"));
    expect(api.setLauncherSetting).toHaveBeenCalledWith("autoUpdate", { type: "choice", index: 0 });
    await waitFor(() => expect(screen.getByRole("switch")).not.toBeChecked());
  });

  it("explains a rejected change and shows what is really saved", async () => {
    api.setLauncherSetting.mockRejectedValue("Check for updates: choice 7 doesn't exist.");
    render(LauncherSettingsView, props);
    await fireEvent.click(await screen.findByRole("switch"));
    expect(await screen.findByRole("alert")).toHaveTextContent("choice 7 doesn't exist");
    await waitFor(() => expect(api.loadLauncherSettings).toHaveBeenCalledTimes(2));
    expect(screen.getByRole("switch")).toBeChecked();
  });

  it("says why the settings can't be shown", async () => {
    api.loadLauncherSettings.mockRejectedValue("Can't read launcher.json: denied");
    render(LauncherSettingsView, props);
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
    render(LauncherSettingsView, props);
    await fireEvent.click(await screen.findByRole("radio", { name: "Pre-release" }));
    await waitFor(() => expect(launcherUpdates.found).toBeNull());
  });

  it("asks for a restart when the language changes, since it is chosen at startup", async () => {
    const language: SettingView = {
      id: "language", label: "Language", help: null, tab: "Game", group: "Interface", advanced: false, enabled: true,
      control: { type: "choices", style: "segmented", options: ["System", "English", "Français"], value: 0, custom: null },
    };
    restartNeeded.needed = false;
    api.loadLauncherSettings.mockResolvedValue(view(language));
    api.setLauncherSetting.mockResolvedValue(view(language));
    render(LauncherSettingsView, props);
    await fireEvent.click(await screen.findByRole("radio", { name: "Français" }));
    await waitFor(() => expect(restartNeeded.needed).toBe(true));
    restartNeeded.needed = false;
  });

  it("puts the notification area icon on or off when its setting changes, and says if it can't", async () => {
    const tray: SettingView = {
      id: "minimizeToTray", label: "Keep running in the notification area", help: null, tab: "Game", group: "Behavior", advanced: false, enabled: true,
      control: { type: "choices", style: "toggle", options: ["Off", "On"], value: 0, custom: null },
    };
    api.loadLauncherSettings.mockResolvedValue(view(tray));
    api.setLauncherSetting.mockResolvedValue(view(tray));
    render(LauncherSettingsView, props);
    await fireEvent.click(await screen.findByRole("switch"));
    await waitFor(() => expect(api.applyBehavior).toHaveBeenCalledWith({ show: "Show the launcher", quit: "Quit" }));

    api.applyBehavior.mockRejectedValueOnce("Can't put the launcher in the notification area: no host");
    await fireEvent.click(screen.getByRole("switch"));
    expect(await screen.findByRole("alert")).toHaveTextContent("no host");
  });

  it("applies a theme change at once, without a restart", async () => {
    const theme: SettingView = {
      id: "theme", label: "Theme", help: null, tab: "Game", group: "Appearance", advanced: false, enabled: true,
      control: { type: "choices", style: "segmented", options: ["Dark", "Light", "System"], value: 0, custom: null },
    };
    api.loadLauncherSettings.mockResolvedValue(view(theme));
    api.setLauncherSetting.mockResolvedValue(view(theme));
    api.launcherAppearance.mockResolvedValue({ theme: "light", gamePalette: true });
    restartNeeded.needed = false;
    render(LauncherSettingsView, props);
    await fireEvent.click(await screen.findByRole("radio", { name: "Light" }));
    await waitFor(() => expect(appearance.theme).toBe("light"));
    expect(restartNeeded.needed).toBe(false);
    document.documentElement.removeAttribute("data-theme");
  });

  it("applies a new download speed limit at once, so running downloads follow it", async () => {
    const limit: SettingView = {
      id: "downloadLimit", label: "Game download speed", help: null, tab: "Game", group: "Downloads", advanced: false, enabled: true,
      control: { type: "choices", style: "dropdown", options: ["Unlimited", "1 MB/s"], value: 0, custom: null },
    };
    api.loadLauncherSettings.mockResolvedValue(view(limit));
    api.setLauncherSetting.mockResolvedValue(view(limit));
    render(LauncherSettingsView, props);
    await fireEvent.click(await screen.findByRole("combobox", { name: "Game download speed" }));
    await fireEvent.click(await screen.findByRole("option", { name: "1 MB/s" }));
    await waitFor(() => expect(api.applyDownloadLimit).toHaveBeenCalledOnce());
    expect(api.setLauncherSetting).toHaveBeenCalledWith("downloadLimit", { type: "choice", index: 1 });
  });
});
