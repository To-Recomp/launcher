import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import SettingsView from "./SettingsView.svelte";
import { settingsTabs } from "../data";

describe("SettingsView", () => {
  it("lists every sub-tab with Display selected first", () => {
    render(SettingsView);
    for (const name of settingsTabs) {
      expect(screen.getByRole("tab", { name })).toBeInTheDocument();
    }
    expect(screen.getByRole("tab", { name: "Display" })).toHaveAttribute("aria-selected", "true");
  });

  it("updates a segmented control, a dropdown and a toggle", async () => {
    render(SettingsView);
    await fireEvent.click(screen.getByRole("radio", { name: "Fullscreen" }));
    expect(screen.getByRole("radio", { name: "Fullscreen" })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: "Windowed" })).toHaveAttribute("aria-checked", "false");

    await fireEvent.change(screen.getByRole("combobox", { name: "Aspect ratio" }), { target: { value: "16:9" } });
    expect((screen.getByRole("combobox", { name: "Aspect ratio" }) as HTMLSelectElement).value).toBe("16:9");

    const vsync = screen.getByRole("switch", { name: "VSync" });
    expect(vsync).toHaveAttribute("aria-checked", "true");
    await fireEvent.click(vsync);
    expect(vsync).toHaveAttribute("aria-checked", "false");
  });

  it("shows a placeholder for sub-tabs without rows yet", async () => {
    render(SettingsView);
    await fireEvent.click(screen.getByRole("tab", { name: "Audio" }));
    expect(screen.getByText(/Audio settings will be generated/)).toBeInTheDocument();
    expect(screen.queryByRole("switch", { name: "VSync" })).not.toBeInTheDocument();
  });
});
