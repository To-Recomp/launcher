import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({
  pickFolder: vi.fn(),
  restartLauncher: vi.fn(),
  loadLauncherSettings: vi.fn(),
  setLauncherSetting: vi.fn(),
  launcherFolders: vi.fn(),
  setGamesFolder: vi.fn(),
  applyBehavior: vi.fn().mockResolvedValue(undefined), proxySettings: vi.fn().mockResolvedValue({ settings: { mode: "system", protocol: "http", host: "", port: 8080, username: "", noProxy: "" }, hasPassword: false, keychainError: null }), setProxy: vi.fn(), testProxy: vi.fn(),
  autostartSupported: vi.fn().mockResolvedValue(false),
  autostartEnabled: vi.fn(),
  setAutostart: vi.fn(),
}));
vi.mock("../launcher/api", () => api);
vi.mock("../settings/api", () => ({ errorMessage: (e: unknown) => String(e) }));

import { restartNeeded } from "../launcher/restart.svelte";
import ActionRow from "./ActionRow.svelte";
import FolderRow from "./FolderRow.svelte";
import LauncherSettingsView from "./LauncherSettingsView.svelte";
import RestartBanner from "./RestartBanner.svelte";

const props = { catalogStatus: null, refreshingCatalog: false, onrefreshcatalog: vi.fn(), oncleaned: vi.fn(), onreportbug: vi.fn() };

beforeEach(() => {
  vi.clearAllMocks();
  restartNeeded.needed = false;
  api.loadLauncherSettings.mockResolvedValue({ settings: [] });
  api.launcherFolders.mockResolvedValue({ configured: null, default: "/data/games", inUse: "/data/games" });
});

describe("FolderRow", () => {
  it("shows the chosen folder, or Default, and reports the folder the player picks", async () => {
    api.pickFolder.mockResolvedValue("/games");
    const onpick = vi.fn();
    const { rerender } = render(FolderRow, { label: "Install folder", path: null, onpick });
    expect(screen.getByText("Default")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Choose…" }));
    expect(api.pickFolder).toHaveBeenCalledWith("Install folder");
    await waitFor(() => expect(onpick).toHaveBeenCalledWith("/games"));
    await rerender({ label: "Install folder", path: "/games", onpick });
    expect(screen.getByText("/games")).toBeInTheDocument();
  });

  it("does nothing when the dialog is closed", async () => {
    api.pickFolder.mockResolvedValue(null);
    const onpick = vi.fn();
    render(FolderRow, { label: "Install folder", path: "/games", onpick });
    await fireEvent.click(screen.getByRole("button", { name: "Choose…" }));
    await waitFor(() => expect(api.pickFolder).toHaveBeenCalled());
    expect(onpick).not.toHaveBeenCalled();
  });
});

describe("ActionRow", () => {
  it("is disabled while it runs, then shows the message it resolved with", async () => {
    let finish: (text: string) => void = () => {};
    const onrun = vi.fn(() => new Promise<string>((resolve) => (finish = resolve)));
    render(ActionRow, { label: "Catalog", action: "Refresh", onrun });
    const button = screen.getByRole("button", { name: "Refresh" });
    await fireEvent.click(button);
    expect(button).toBeDisabled();
    finish("Updated just now.");
    expect(await screen.findByRole("status")).toHaveTextContent("Updated just now.");
    expect(button).toBeEnabled();
  });

  it("shows why it failed", async () => {
    render(ActionRow, { label: "Catalog", action: "Refresh", onrun: () => Promise.reject("Can't reach the catalog") });
    await fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Can't reach the catalog");
  });
});

describe("RestartBanner", () => {
  it("restarts the launcher on demand and says why when it can't", async () => {
    api.restartLauncher.mockRejectedValue("Can't restart");
    render(RestartBanner);
    await fireEvent.click(screen.getByRole("button", { name: "Restart now" }));
    expect(api.restartLauncher).toHaveBeenCalledOnce();
    expect(await screen.findByText("Can't restart")).toBeInTheDocument();
  });
});

describe("LauncherSettingsView restart banner", () => {
  it("appears once a setting needs a restart, and not before", async () => {
    render(LauncherSettingsView, props);
    await screen.findByText("About");
    expect(screen.queryByText("Restart the launcher to apply these changes.")).not.toBeInTheDocument();
    restartNeeded.mark();
    expect(await screen.findByText("Restart the launcher to apply these changes.")).toBeInTheDocument();
  });
});
