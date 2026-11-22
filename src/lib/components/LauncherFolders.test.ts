import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Folders } from "../launcher/api";

const api = vi.hoisted(() => ({ launcherFolders: vi.fn(), setGamesFolder: vi.fn(), pickFolder: vi.fn() }));
vi.mock("../launcher/api", () => api);
vi.mock("../settings/api", () => ({ errorMessage: (e: unknown) => String(e) }));

import { restartNeeded } from "../launcher/restart.svelte";
import LauncherFolders from "./LauncherFolders.svelte";

const inUse = (configured: string | null): Folders => ({ configured, default: "/data/games", inUse: "/data/games" });

beforeEach(() => {
  vi.clearAllMocks();
  restartNeeded.needed = false;
  api.launcherFolders.mockResolvedValue(inUse(null));
});

describe("LauncherFolders", () => {
  it("shows the default folder, and no restart is asked for", async () => {
    render(LauncherFolders);
    expect(await screen.findByText("In use: /data/games")).toBeInTheDocument();
    expect(screen.getByText("Default")).toBeInTheDocument();
    expect(restartNeeded.needed).toBe(false);
  });

  it("saves the picked folder and asks for a restart, since games are found at startup", async () => {
    api.pickFolder.mockResolvedValue("/mnt/big");
    api.setGamesFolder.mockResolvedValue(inUse("/mnt/big"));
    render(LauncherFolders);
    await fireEvent.click(await screen.findByRole("button", { name: "Choose…" }));
    expect(await screen.findByText("/mnt/big")).toBeInTheDocument();
    expect(api.setGamesFolder).toHaveBeenCalledWith("/mnt/big");
    expect(restartNeeded.needed).toBe(true);
  });

  it("goes back to the default, and needs no restart when that is what is in use", async () => {
    api.launcherFolders.mockResolvedValue(inUse("/data/games"));
    api.setGamesFolder.mockResolvedValue(inUse(null));
    render(LauncherFolders);
    await fireEvent.click(await screen.findByRole("button", { name: "Use default" }));
    expect(await screen.findByText("Default")).toBeInTheDocument();
    expect(api.setGamesFolder).toHaveBeenCalledWith(null);
    expect(restartNeeded.needed).toBe(false);
  });

  it("says why a folder was refused and keeps the previous choice", async () => {
    api.pickFolder.mockResolvedValue("/nope");
    api.setGamesFolder.mockRejectedValue("/nope isn't an existing folder.");
    render(LauncherFolders);
    await fireEvent.click(await screen.findByRole("button", { name: "Choose…" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("/nope isn't an existing folder.");
    expect(screen.getByText("Default")).toBeInTheDocument();
    await waitFor(() => expect(restartNeeded.needed).toBe(false));
  });
});
