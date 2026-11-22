// Mirrors src-tauri/src/settings/mod.rs (serde camelCase).

export type ChoiceStyle = "toggle" | "segmented" | "dropdown";

export interface ActionView {
  label: string;
  key: string;
  keys: string[];
  default: string[];
  pad: boolean;
}

export type Control =
  | { type: "choices"; style: ChoiceStyle; options: string[]; value: number | null; custom: string | null }
  | { type: "slider"; min: number; max: number; step: number; value: number; format: string | null }
  | { type: "text"; value: string }
  | { type: "bindings"; max: number; actions: ActionView[] };

export interface SettingView {
  id: string;
  label: string;
  help: string | null;
  tab: string;
  group: string | null;
  advanced: boolean;
  /** False when the setting's `when` condition doesn't hold: shown greyed out. */
  enabled: boolean;
  control: Control;
}

export interface SettingsView {
  settings: SettingView[];
}

export type SettingInput =
  | { type: "choice"; index: number }
  | { type: "number"; value: number }
  | { type: "text"; value: string }
  | { type: "bindings"; action: string; keys: string[] };

/** Where one installed game lives. */
export interface Location {
  installDir: string;
  dataDir?: string;
}
