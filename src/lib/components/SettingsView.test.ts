import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SettingView, SettingsView as View } from "../settings/types";

const api = vi.hoisted(() => ({ loadSettings: vi.fn(), setSetting: vi.fn() }));
vi.mock("../settings/api", () => ({
  loadSettings: api.loadSettings,
  setSetting: api.setSetting,
  errorMessage: (e: unknown) => String(e),
}));

const releases = vi.hoisted(() => ({ gameLicense: vi.fn() }));
vi.mock("../catalog/releases", () => releases);

const links = vi.hoisted(() => ({ openLink: vi.fn(), revealFile: vi.fn() }));
vi.mock("../diagnostics/api", () => ({
  ...links,
  repositoryUrl: (repository: string) => `https://github.com/${repository}`,
}));

import SettingsView from "./SettingsView.svelte";

const base = { help: null, group: null, advanced: false, enabled: true };
const toggle: SettingView = { ...base, id: "vsync", label: "VSync", tab: "Display", group: "Frame rate",
  control: { type: "choices", style: "toggle", options: ["Off", "On"], value: 1, custom: null } };
const segmented: SettingView = { ...base, id: "window", label: "Window", tab: "Display", group: "Picture",
  control: { type: "choices", style: "segmented", options: ["Windowed", "Borderless", "Fullscreen"], value: 0, custom: null } };
const dropdown: SettingView = { ...base, id: "size", label: "Window size", tab: "Display", group: "Picture", enabled: false,
  control: { type: "choices", style: "dropdown", options: ["1280 × 720", "1600 × 900"], value: null, custom: "1234" } };
const slider: SettingView = { ...base, id: "volume", label: "Volume", tab: "Audio",
  control: { type: "slider", min: 0, max: 1, step: 0.05, value: 0.25, format: "%.2f" } };
const text: SettingView = { ...base, id: "name", label: "Player name", tab: "Game", help: "Shown in saves.",
  control: { type: "text", value: "Yuri" } };
const keys: SettingView = { ...base, id: "keys", label: "Keyboard", tab: "Controls",
  control: { type: "bindings", max: 2, actions: [{ label: "Cross", key: "key.cross", keys: ["Space"], default: ["Space"], pad: false }] } };
const hidden: SettingView = { ...base, id: "debug", label: "Debug overlay", tab: "Game", advanced: true,
  control: { type: "choices", style: "toggle", options: ["Off", "On"], value: 0, custom: null } };

const view = (...settings: SettingView[]): View => ({ settings });
const all = view(toggle, segmented, dropdown, slider, text, keys, hidden);
const location = { installDir: "/games/eternia", dataDir: "/games/eternia/data" };
const onreportbug = vi.fn();
const props = {
  gameId: "eternia",
  gameName: "Tales of Eternia",
  repository: "To-Recomp/toe",
  platform: "PS1",
  tag: "v0.3.0",
  location,
  onreportbug,
  files: { hasInputs: false, current: [], busy: false, onchange: vi.fn() },
  onback: vi.fn(),
};

async function openTab(name: string) {
  await fireEvent.click(screen.getByRole("tab", { name }));
}

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  api.loadSettings.mockResolvedValue(all);
  releases.gameLicense.mockResolvedValue({ name: "GNU General Public License v3.0", url: "https://github.com/To-Recomp/toe/blob/main/LICENSE" });
});

describe("SettingsView", () => {
  it("asks for an installation when the game has none, without calling the backend", () => {
    render(SettingsView, { ...props, location: null });
    expect(screen.getByText("Install and prepare Tales of Eternia to configure it.")).toBeInTheDocument();
    expect(api.loadSettings).not.toHaveBeenCalled();
  });

  it("renders each control type in the sub-tab the manifest names", async () => {
    render(SettingsView, props);
    expect(await screen.findByRole("switch", { name: "VSync" })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: "Windowed" })).toHaveAttribute("aria-checked", "true");
    // Group headings, in manifest order.
    expect(screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toEqual(["Frame rate", "Picture"]);

    await openTab("Audio");
    expect(screen.getByRole("slider", { name: "Volume" })).toHaveValue("0.25");
    expect(screen.getByText("0.25")).toBeInTheDocument();
    await openTab("Game");
    expect(screen.getByRole("textbox", { name: "Player name" })).toHaveValue("Yuri");
    expect(screen.getByText("Shown in saves.")).toBeInTheDocument();
    await openTab("Controls");
    expect(screen.getByRole("button", { name: "Cross: Space" })).toBeInTheDocument();
    await openTab("Saves");
    expect(screen.getByText("No settings in this section.")).toBeInTheDocument();
  });

  it("shows a value that matches no choice as it is, and greys out unavailable settings", async () => {
    render(SettingsView, props);
    const select = await screen.findByRole("combobox", { name: "Window size" });
    expect(select).toHaveTextContent("1234");
    expect(select.closest("[aria-disabled=true]")).not.toBeNull();
  });

  it("sends the typed change for every control and shows the refreshed view", async () => {
    api.setSetting.mockImplementation(async () => view({ ...toggle, control: { ...toggle.control, value: 0 } as SettingView["control"] }, segmented, slider, text, keys));
    render(SettingsView, props);
    await fireEvent.click(await screen.findByRole("switch", { name: "VSync" }));
    expect(api.setSetting).toHaveBeenLastCalledWith(location, "vsync", { type: "choice", index: 0 });
    await waitFor(() => expect(screen.getByRole("switch", { name: "VSync" })).toHaveAttribute("aria-checked", "false"));

    await fireEvent.click(screen.getByRole("radio", { name: "Fullscreen" }));
    expect(api.setSetting).toHaveBeenLastCalledWith(location, "window", { type: "choice", index: 2 });

    api.setSetting.mockResolvedValue(all);
    await openTab("Audio");
    await fireEvent.change(screen.getByRole("slider", { name: "Volume" }), { target: { value: "0.7" } });
    expect(api.setSetting).toHaveBeenLastCalledWith(location, "volume", { type: "number", value: 0.7 });
    await openTab("Game");
    await fireEvent.change(screen.getByRole("textbox", { name: "Player name" }), { target: { value: "Flynn" } });
    expect(api.setSetting).toHaveBeenLastCalledWith(location, "name", { type: "text", value: "Flynn" });
    await openTab("Controls");
    await fireEvent.click(screen.getByRole("button", { name: "Cross: Space" }));
    await fireEvent.keyDown(screen.getByRole("button", { name: /Press a key/ }), { code: "KeyX" });
    expect(api.setSetting).toHaveBeenLastCalledWith(location, "keys", { type: "bindings", action: "key.cross", keys: ["X"] });
  });

  it("shows the backend's message for a rejected value and reloads the real state", async () => {
    api.setSetting.mockRejectedValue("Volume must be between 0 and 1.");
    render(SettingsView, props);
    await openTab("Audio");
    await fireEvent.change(await screen.findByRole("slider", { name: "Volume" }), { target: { value: "0.5" } });
    expect(await screen.findByRole("alert")).toHaveTextContent("Volume must be between 0 and 1.");
    expect(api.loadSettings).toHaveBeenCalledTimes(2);
    await fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows why the manifest couldn't be loaded", async () => {
    api.loadSettings.mockRejectedValue("Invalid recomplauncher.json: setting `a`: `choices` is empty");
    render(SettingsView, props);
    expect(await screen.findByRole("alert")).toHaveTextContent("`choices` is empty");
  });

  it("hides advanced settings until asked, and remembers the choice per game", async () => {
    render(SettingsView, props);
    await openTab("Game");
    expect(screen.queryByRole("switch", { name: "Debug overlay" })).not.toBeInTheDocument();
    await fireEvent.click(await screen.findByRole("button", { name: "Show advanced options" }));
    expect(screen.getByRole("switch", { name: "Debug overlay" })).toBeInTheDocument();
    cleanup();

    render(SettingsView, props);
    await openTab("Game");
    expect(await screen.findByRole("switch", { name: "Debug overlay" })).toBeInTheDocument();
    cleanup();

    render(SettingsView, { ...props, gameId: "symphonia" });
    await openTab("Game");
    await screen.findByRole("textbox", { name: "Player name" });
    expect(screen.queryByRole("switch", { name: "Debug overlay" })).not.toBeInTheDocument();
  });

  it("describes the installed game on About", async () => {
    render(SettingsView, props);
    await openTab("About");
    expect(screen.getByText("Tales of Eternia")).toBeInTheDocument();
    expect(screen.getByText("PS1")).toBeInTheDocument();
    expect(screen.getByText("v0.3.0")).toBeInTheDocument();
    expect(screen.getByText("To-Recomp/toe")).toBeInTheDocument();
    expect(await screen.findByText("GNU General Public License v3.0")).toBeInTheDocument();
    expect(screen.getByText("/games/eternia")).toBeInTheDocument();
    expect(screen.getByText("/games/eternia/data")).toBeInTheDocument();
  });

  it("opens the repository, the license and the folders from About", async () => {
    render(SettingsView, props);
    await openTab("About");
    await screen.findByText("GNU General Public License v3.0");
    const open = (label: string) => fireEvent.click(within(screen.getByText(label).closest(".row") as HTMLElement).getByRole("button", { name: "Open" }));
    await open("Repository");
    await open("License");
    await open("Install folder");
    await open("Data folder");
    expect(links.openLink).toHaveBeenNthCalledWith(1, "https://github.com/To-Recomp/toe");
    expect(links.openLink).toHaveBeenNthCalledWith(2, "https://github.com/To-Recomp/toe/blob/main/LICENSE");
    expect(links.revealFile).toHaveBeenNthCalledWith(1, "/games/eternia");
    expect(links.revealFile).toHaveBeenNthCalledWith(2, "/games/eternia/data");
  });

  it("still describes a game that isn't installed, without folders", async () => {
    releases.gameLicense.mockResolvedValue(null);
    render(SettingsView, { ...props, location: null, tag: null });
    await openTab("About");
    expect(screen.getByText("Not installed")).toBeInTheDocument();
    expect(await screen.findByText("Not published")).toBeInTheDocument();
    expect(screen.queryByText("Install folder")).not.toBeInTheDocument();
  });

  it("offers Report a bug for the game on About, even without an installation", async () => {
    render(SettingsView, { ...props, location: null });
    await openTab("About");
    await fireEvent.click(screen.getByRole("button", { name: "Report a bug" }));
    expect(onreportbug).toHaveBeenCalledOnce();
  });

  it("changes one file at a time from the top of the Game tab, even before the game is prepared, when nothing is running", async () => {
    const onchange = vi.fn();
    // Two inputs may share a label (a manifest can describe both discs the same way): they are told apart by id.
    const current = [{ id: "disc1", label: "Disc", path: "/iso/eternia.iso" }, { id: "disc2", label: "Disc", path: null }];
    render(SettingsView, { ...props, location: null, files: { hasInputs: true, current, busy: false, onchange } });
    expect(screen.queryByRole("tab", { name: "Files" })).not.toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Files" })).not.toBeInTheDocument(); // not on Display
    await openTab("Game");
    expect(screen.getByRole("heading", { level: 2, name: "Files" })).toBeInTheDocument();
    // A short path is all "end"; a long one is cut between its start and its end.
    const path = screen.getByTitle("/iso/eternia.iso");
    expect(path.querySelector(".head")?.textContent).toBe("");
    expect(path.querySelector(".tail")?.textContent).toBe("/iso/eternia.iso");
    expect(screen.getAllByText("Disc")).toHaveLength(2);
    expect(screen.getByText("Not chosen")).toBeInTheDocument();
    // The files come first; the settings (here: none yet) follow.
    expect(screen.getByRole("heading", { name: "Files" }).compareDocumentPosition(screen.getByText("Install and prepare Tales of Eternia to configure it."))).toBe(Node.DOCUMENT_POSITION_FOLLOWING);

    const buttons = screen.getAllByRole("button", { name: "Change Disc" }); // one per file
    expect(buttons).toHaveLength(2);
    await fireEvent.click(buttons[1]);
    expect(onchange).toHaveBeenCalledExactlyOnceWith("disc2");

    cleanup();
    render(SettingsView, { ...props, files: { hasInputs: true, current, busy: true, onchange } });
    await openTab("Game");
    for (const button of screen.getAllByRole("button", { name: "Change Disc" })) expect(button).toBeDisabled();
  });

  it("has no Files section for a game that uses no files, and goes back to the game", async () => {
    const onback = vi.fn();
    render(SettingsView, { ...props, onback });
    await openTab("Game");
    expect(screen.queryByRole("heading", { name: "Files" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /^Change / })).not.toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Back" }));
    expect(onback).toHaveBeenCalledOnce();
  });
});

describe("a path in the Files section", () => {
  const shown = async (path: string) => {
    render(SettingsView, { ...props, files: { hasInputs: true, current: [{ id: "d", label: "Disc", path }], busy: false, onchange: vi.fn() } });
    await openTab("Game");
    const el = screen.getByTitle(path);
    return [el.querySelector(".head")?.textContent, el.querySelector(".tail")?.textContent];
  };

  it("keeps the last 22 characters (the extension with them) apart from the start", async () => {
    const path = "/mnt/library/games/Tales of/Symphonia/Tales of Symphonia (France) (Disc 1) (2).iso";
    const [head, tail] = await shown(path);
    expect(tail).toBe(path.slice(-22));
    expect(tail).toMatch(/\.iso$/);
    expect(`${head}${tail}`).toBe(path);
    expect(head?.startsWith("/mnt/library/")).toBe(true);
    cleanup();
    expect(await shown("D:\\Games\\Disc 1.iso")).toEqual(["", "D:\\Games\\Disc 1.iso"]); // short: all end
  });
});
