import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";

const win = vi.hoisted(() => {
  (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  return { minimize: vi.fn(), toggleMaximize: vi.fn(), close: vi.fn() };
});

vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => win }));

import TitleBar from "./TitleBar.svelte";

describe("TitleBar", () => {
  it("drives the native window from the custom controls", async () => {
    render(TitleBar);
    await fireEvent.click(screen.getByRole("button", { name: "Minimize" }));
    await fireEvent.click(screen.getByRole("button", { name: "Maximize" }));
    await fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(win.minimize).toHaveBeenCalledOnce();
    expect(win.toggleMaximize).toHaveBeenCalledOnce();
    expect(win.close).toHaveBeenCalledOnce();
  });
});
