import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import KeyBindings from "./KeyBindings.svelte";

const actions = [
  { label: "Cross", key: "key.cross", keys: ["Space"], default: ["Space"], pad: false },
  { label: "Options", key: "key.options", keys: ["Escape", "Return"], default: ["Escape"], pad: false },
];

describe("KeyBindings", () => {
  it("replaces a key with the next one pressed", async () => {
    const onchange = vi.fn();
    render(KeyBindings, { actions, max: 2, onchange });
    const chip = screen.getByRole("button", { name: "Cross: Space" });
    await fireEvent.click(chip);
    await fireEvent.keyDown(screen.getByRole("button", { name: /Press a key/ }), { code: "KeyZ" });
    expect(onchange).toHaveBeenCalledWith("key.cross", ["Z"]);
  });

  it("captures a mouse button and the wheel", async () => {
    const onchange = vi.fn();
    render(KeyBindings, { actions, max: 2, onchange });
    await fireEvent.click(screen.getByRole("button", { name: "Add a key to Cross" }));
    await fireEvent.mouseDown(screen.getByRole("button", { name: /Press a key/ }), { button: 2 });
    expect(onchange).toHaveBeenLastCalledWith("key.cross", ["Space", "Mouse Right"]);

    await fireEvent.click(screen.getByRole("button", { name: "Add a key to Cross" }));
    await fireEvent.wheel(screen.getByRole("button", { name: /Press a key/ }), { deltaY: -100 });
    expect(onchange).toHaveBeenLastCalledWith("key.cross", ["Space", "Wheel Up"]);
  });

  it("ignores keys it can't hold and a key already bound", async () => {
    const onchange = vi.fn();
    render(KeyBindings, { actions, max: 2, onchange });
    await fireEvent.click(screen.getByRole("button", { name: "Add a key to Cross" }));
    await fireEvent.keyDown(screen.getByRole("button", { name: /Press a key/ }), { code: "MediaPlayPause" });
    await fireEvent.click(screen.getByRole("button", { name: "Add a key to Cross" }));
    await fireEvent.keyDown(screen.getByRole("button", { name: /Press a key/ }), { code: "Space" });
    expect(onchange).not.toHaveBeenCalled();
  });

  it("offers add only below the maximum, and reset only when changed", () => {
    render(KeyBindings, { actions, max: 2, onchange: () => {} });
    expect(screen.getByRole("button", { name: "Add a key to Cross" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Add a key to Options" })).not.toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Reset" })).toHaveLength(1);
  });

  it("removes a key and resets to the default", async () => {
    const onchange = vi.fn();
    render(KeyBindings, { actions, max: 2, onchange });
    await fireEvent.click(screen.getByRole("button", { name: "Remove Return from Options" }));
    expect(onchange).toHaveBeenLastCalledWith("key.options", ["Escape"]);
    await fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    expect(onchange).toHaveBeenLastCalledWith("key.options", ["Escape"]);
  });
});
