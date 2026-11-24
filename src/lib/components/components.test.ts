import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
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

  it("puts a small prefix before an option and keeps its width for the widest text it may show", async () => {
    render(Dropdown, {
      options: ["PS2", "PS3"], value: "PS2", label: "Platform",
      prefixes: { PS2: "2000", PS3: "2006" },
      sizing: [{ option: "Nintendo Switch 2", prefix: "2025" }],
      onchange: () => {},
    });
    const box = screen.getByRole("combobox", { name: "Platform" });
    expect(box.querySelector(".value")?.textContent?.trim()).toBe("2000 · PS2");
    // Hidden copies of every possible text, so the box never resizes when another option is chosen.
    const sizers = [...box.querySelectorAll(".sizer")];
    expect(sizers.map((s) => s.textContent?.trim())).toEqual(["2000 · PS2", "2006 · PS3", "2025 · Nintendo Switch 2"]);
    expect(sizers.every((s) => s.getAttribute("aria-hidden") === "true")).toBe(true);
    await fireEvent.click(box);
    expect(screen.getAllByRole("option").map((o) => o.textContent?.trim())).toEqual(["2000 · PS2", "2006 · PS3"]);
  });
});

describe("Dropdown with options that can't be chosen", () => {
  const show = () => {
    const onchange = vi.fn();
    render(Dropdown, {
      options: ["A", "B", "C", "D"], value: "A", label: "Platform", disabled: ["B", "D"], notes: { B: "N/A", D: "N/A" }, onchange,
    });
    return { onchange, box: screen.getByRole("combobox", { name: "Platform" }) };
  };

  it("lists them with their note, greyed out, and doesn't let a click choose them", async () => {
    const { onchange, box } = show();
    await fireEvent.click(box);
    expect(screen.getByRole("option", { name: /B/ })).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByRole("option", { name: /B/ })).toHaveTextContent("N/A");
    await fireEvent.click(screen.getByRole("option", { name: /B/ }));
    expect(onchange).not.toHaveBeenCalled();
    await fireEvent.click(screen.getByRole("option", { name: "C" }));
    expect(onchange).toHaveBeenCalledWith("C");
  });

  it("skips them with the keyboard", async () => {
    const { onchange, box } = show();
    await fireEvent.keyDown(box, { key: "ArrowDown" }); // opens on A
    await fireEvent.keyDown(box, { key: "ArrowDown" }); // B is skipped
    await fireEvent.keyDown(box, { key: "Enter" });
    expect(onchange).toHaveBeenCalledWith("C");
  });
});

describe("SidebarItem", () => {
  it("offers a separate quick play button only for a game that can be played at once", async () => {
    const onplay = vi.fn();
    const onselect = vi.fn();
    render(SidebarItem, { game: { ...eternia, state: "installed" }, onselect, onplay });
    expect(screen.queryByRole("button", { name: /^Play/ })).not.toBeInTheDocument();
    cleanup();
    render(SidebarItem, { game: { ...eternia, state: "not-installed" }, onselect, onplay });
    expect(screen.queryByRole("button", { name: /^Play/ })).not.toBeInTheDocument();
    cleanup();
    render(SidebarItem, { game: { ...eternia, state: "ready" }, onselect, onplay });
    await fireEvent.click(screen.getByRole("button", { name: "Play Tales of Eternia" }));
    expect(onplay).toHaveBeenCalledOnce();
    expect(onselect).not.toHaveBeenCalled(); // playing doesn't open the game's page by itself
  });
});
