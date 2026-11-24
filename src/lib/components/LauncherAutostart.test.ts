import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({ autostartSupported: vi.fn(), autostartEnabled: vi.fn(), setAutostart: vi.fn() }));
vi.mock("../launcher/api", () => api);
vi.mock("../settings/api", () => ({ errorMessage: (e: unknown) => String(e) }));

import LauncherAutostart from "./LauncherAutostart.svelte";

beforeEach(() => {
  vi.clearAllMocks();
  api.autostartSupported.mockResolvedValue(true);
  api.autostartEnabled.mockResolvedValue(false);
});

describe("LauncherAutostart", () => {
  it("offers nothing where the system can't start the launcher", async () => {
    api.autostartSupported.mockResolvedValue(false);
    render(LauncherAutostart);
    await waitFor(() => expect(api.autostartSupported).toHaveBeenCalled());
    expect(screen.queryByRole("switch")).not.toBeInTheDocument();
  });

  it("shows what the system says and follows what it answers after a change", async () => {
    api.autostartEnabled.mockResolvedValue(true);
    api.setAutostart.mockResolvedValue(false);
    render(LauncherAutostart);
    const toggle = await screen.findByRole("switch", { name: "Start with the system" });
    await waitFor(() => expect(toggle).toBeChecked());
    await fireEvent.click(toggle);
    expect(api.setAutostart).toHaveBeenCalledWith(false);
    await waitFor(() => expect(toggle).not.toBeChecked());
  });

  it("says why the system refused and keeps the real state", async () => {
    api.setAutostart.mockRejectedValue("Can't change whether the launcher starts with the system: denied");
    render(LauncherAutostart);
    await fireEvent.click(await screen.findByRole("switch", { name: "Start with the system" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("denied");
    expect(screen.getByRole("switch")).not.toBeChecked();
  });
});
