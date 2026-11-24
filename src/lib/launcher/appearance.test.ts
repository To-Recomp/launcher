import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({ launcherAppearance: vi.fn() }));
vi.mock("./api", () => api);

import { AppearanceStore } from "./appearance.svelte";

let systemDark = true;
let notify: (e: { matches: boolean }) => void = () => {};

beforeEach(() => {
  vi.clearAllMocks();
  systemDark = true;
  window.matchMedia = ((query: string) => ({
    matches: systemDark,
    media: query,
    addEventListener: (_: string, listener: typeof notify) => (notify = listener),
    removeEventListener: () => {},
  })) as unknown as typeof window.matchMedia;
});
afterEach(() => document.documentElement.removeAttribute("data-theme"));

describe("AppearanceStore", () => {
  it("is dark when nothing is saved or the settings can't be read", async () => {
    api.launcherAppearance.mockRejectedValue("no runtime");
    const appearance = new AppearanceStore();
    await appearance.load();
    expect(appearance.theme).toBe("dark");
    expect(appearance.usesGamePalette).toBe(true);
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("shows the chosen theme on the page, and never tints a light one with the game's colors", async () => {
    api.launcherAppearance.mockResolvedValue({ theme: "light" });
    const appearance = new AppearanceStore();
    await appearance.load();
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(appearance.usesGamePalette).toBe(false);
  });

  it("follows the system's theme, including when it changes while the launcher runs", async () => {
    systemDark = false;
    api.launcherAppearance.mockResolvedValue({ theme: "system" });
    const appearance = new AppearanceStore();
    await appearance.load();
    expect(appearance.theme).toBe("light");
    notify({ matches: true });
    expect(appearance.theme).toBe("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("applies a change made in the settings at once", async () => {
    api.launcherAppearance.mockResolvedValue({ theme: "dark" });
    const appearance = new AppearanceStore();
    await appearance.load();
    api.launcherAppearance.mockResolvedValue({ theme: "light" });
    await appearance.reload();
    expect(document.documentElement.dataset.theme).toBe("light");
  });
});
