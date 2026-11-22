/** Launcher key names of the DOM events a binding captures (see the backend's `keys.rs`). */

const BY_CODE: Record<string, string> = {
  Space: "Space",
  Enter: "Return",
  NumpadEnter: "Return",
  Escape: "Escape",
  Tab: "Tab",
  Backspace: "Backspace",
  ShiftLeft: "Shift",
  ShiftRight: "Shift",
  ControlLeft: "Ctrl",
  ControlRight: "Ctrl",
  AltLeft: "Alt",
  AltRight: "Alt",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Insert: "Insert",
  Delete: "Delete",
  Home: "Home",
  End: "End",
  PageUp: "Page Up",
  PageDown: "Page Down",
  CapsLock: "Caps Lock",
};

/** `null` for keys a binding can't hold. */
export function keyName(event: Pick<KeyboardEvent, "code">): string | null {
  const { code } = event;
  const letter = code.match(/^Key([A-Z])$/);
  if (letter) return letter[1];
  const digit = code.match(/^Digit(\d)$/);
  if (digit) return digit[1];
  if (/^F([1-9]|1[0-2])$/.test(code)) return code;
  return BY_CODE[code] ?? null;
}

const MOUSE = ["Mouse Left", "Mouse Middle", "Mouse Right", "Mouse 4", "Mouse 5"];

export function mouseName(event: Pick<MouseEvent, "button">): string | null {
  return MOUSE[event.button] ?? null;
}

export function wheelName(event: Pick<WheelEvent, "deltaY">): string | null {
  if (event.deltaY < 0) return "Wheel Up";
  if (event.deltaY > 0) return "Wheel Down";
  return null;
}
