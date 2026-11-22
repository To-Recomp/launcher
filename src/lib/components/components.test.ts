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
  const show = (value = "1") => {
    const onchange = vi.fn();
    render(Dropdown, { options: ["1", "2", "3"], value, label: "Size", onchange });
    return { onchange, box: screen.getByRole("combobox", { name: "Size" }) };
  };

  it("shows the value and opens a list of options on click, closing it on a choice", async () => {
    const { onchange, box } = show();
    expect(box).toHaveTextContent("1");
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    await fireEvent.click(box);
    expect(box).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("option", { name: "1" })).toHaveAttribute("aria-selected", "true");
    await fireEvent.click(screen.getByRole("option", { name: "2" }));
    expect(onchange).toHaveBeenCalledWith("2");
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  });

  it("does not report the option that is already selected", async () => {
    const { onchange, box } = show();
    await fireEvent.click(box);
    await fireEvent.click(screen.getByRole("option", { name: "1" }));
    expect(onchange).not.toHaveBeenCalled();
  });

  it("works from the keyboard: arrows move, Enter chooses, Escape cancels", async () => {
    const { onchange, box } = show();
    await fireEvent.keyDown(box, { key: "ArrowDown" }); // opens on the current value
    expect(box).toHaveAttribute("aria-expanded", "true");
    await fireEvent.keyDown(box, { key: "ArrowDown" });
    await fireEvent.keyDown(box, { key: "ArrowDown" });
    await fireEvent.keyDown(box, { key: "ArrowDown" }); // stops at the last option
    await fireEvent.keyDown(box, { key: "Enter" });
    expect(onchange).toHaveBeenCalledExactlyOnceWith("3");
    expect(box).toHaveAttribute("aria-expanded", "false");

    await fireEvent.keyDown(box, { key: "Enter" });
    await fireEvent.keyDown(box, { key: "Home" });
    await fireEvent.keyDown(box, { key: "Escape" });
    expect(onchange).toHaveBeenCalledOnce();
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  });

  it("closes when the pointer goes down elsewhere", async () => {
    const { box } = show();
    await fireEvent.click(box);
    await fireEvent.pointerDown(document.body);
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
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
