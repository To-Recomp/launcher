//! Key names of key-binding settings.

const NAMED: &[&str] = &[
    "Space",
    "Return",
    "Escape",
    "Tab",
    "Backspace",
    "Shift",
    "Ctrl",
    "Alt",
    "Up",
    "Down",
    "Left",
    "Right",
    "Mouse Left",
    "Mouse Right",
    "Mouse Middle",
    "Mouse 4",
    "Mouse 5",
    "Wheel Up",
    "Wheel Down",
    "Insert",
    "Delete",
    "Home",
    "End",
    "Page Up",
    "Page Down",
    "Caps Lock",
];

const PAD: &[&str] = &[
    "A",
    "B",
    "X",
    "Y",
    "Cross",
    "Circle",
    "Square",
    "Triangle",
    "LB",
    "RB",
    "LT",
    "RT",
    "Start",
    "Back",
    "Left Stick",
    "Right Stick",
    "D-pad Up",
    "D-pad Down",
    "D-pad Left",
    "D-pad Right",
    "L3",
    "R3",
];

pub fn is_valid(name: &str, pad: bool) -> bool {
    let mut chars = name.chars();
    let single =
        matches!((chars.next(), chars.next()), (Some(c), None) if c.is_ascii_alphanumeric());
    let function = name
        .strip_prefix('F')
        .and_then(|n| n.parse::<u8>().ok())
        .is_some_and(|n| (1..=12).contains(&n));
    single || function || NAMED.contains(&name) || (pad && PAD.contains(&name))
}
