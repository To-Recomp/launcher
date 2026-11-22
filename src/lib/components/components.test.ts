import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import Segmented from "./Segmented.svelte";
import Toggle from "./Toggle.svelte";
import Dropdown from "./Dropdown.svelte";
import SidebarItem from "./SidebarItem.svelte";
import { eternia } from "./fixtures";

describe("Segmented", () => {
  it("marks the current option and reports a new choice", async () => {
    const onchange = vi.fn();
    render(Segmented, { options: ["A", "B"], value: "A", label: "Mode", onchange });
    expect(screen.getByRole("radio", { name: "A" })).toHaveAttribute("aria-checked", "true");
    await fireEvent.click(screen.getByRole("radio", { name: "B" }));
    expect(onchange).toHaveBeenCalledWith("B");
  });
});

describe("Toggle", () => {
  it("reports the inverted state on click", async () => {
    const onchange = vi.fn();
    render(Toggle, { checked: true, label: "VSync", onchange });
    const sw = screen.getByRole("switch", { name: "VSync" });
    expect(sw).toHaveAttribute("aria-checked", "true");
    await fireEvent.click(sw);
    expect(onchange).toHaveBeenCalledWith(false);
  });
});

describe("Dropdown", () => {
  it("reports the selected option", async () => {
    const onchange = vi.fn();
    render(Dropdown, { options: ["1", "2"], value: "1", label: "Size", onchange });
    await fireEvent.change(screen.getByRole("combobox", { name: "Size" }), { target: { value: "2" } });
    expect(onchange).toHaveBeenCalledWith("2");
  });
});

describe("SidebarItem", () => {
  it.each([
    ["not-installed", "Install"],
    ["installed", "Settings"],
    ["ready", "Play"],
  ] as const)("shows the %s action as %s", (state, label) => {
    render(SidebarItem, { game: { ...eternia, state }, onselect: () => {} });
    expect(screen.getByRole("img", { name: label })).toBeInTheDocument();
  });
});
