import { describe, expect, it } from "vitest";
import { keyName, mouseName, wheelName } from "./keys";
import { formatNumber } from "./format";

describe("keyName", () => {
  it.each([
    ["KeyA", "A"],
    ["Digit7", "7"],
    ["F12", "F12"],
    ["Space", "Space"],
    ["Enter", "Return"],
    ["NumpadEnter", "Return"],
    ["ControlRight", "Ctrl"],
    ["ArrowLeft", "Left"],
    ["PageDown", "Page Down"],
  ])("%s is %s", (code, name) => {
    expect(keyName({ code })).toBe(name);
  });

  it.each(["F13", "MediaPlayPause", "IntlBackslash", ""])("rejects %s", (code) => {
    expect(keyName({ code })).toBeNull();
  });
});

describe("pointer names", () => {
  it("maps mouse buttons and the wheel", () => {
    expect([0, 1, 2, 3, 4, 5].map((button) => mouseName({ button }))).toEqual([
      "Mouse Left", "Mouse Middle", "Mouse Right", "Mouse 4", "Mouse 5", null,
    ]);
    expect(wheelName({ deltaY: -3 })).toBe("Wheel Up");
    expect(wheelName({ deltaY: 3 })).toBe("Wheel Down");
    expect(wheelName({ deltaY: 0 })).toBeNull();
  });
});

describe("formatNumber", () => {
  it("follows the slider's printf format", () => {
    expect(formatNumber("%.2f", 1)).toBe("1.00");
    expect(formatNumber("%d", 2.6)).toBe("3");
    expect(formatNumber("%g", 0.25)).toBe("0.25");
    expect(formatNumber(null, 5)).toBe("5");
  });
});
