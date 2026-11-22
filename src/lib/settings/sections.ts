import type { SettingView } from "./types";

export interface Section {
  group: string | null;
  items: SettingView[];
}

/** Consecutive settings of the same group share a card under one heading. */
export function sectionsOf(settings: SettingView[]): Section[] {
  return settings.reduce<Section[]>((acc, setting) => {
    const last = acc.at(-1);
    if (last && last.group === setting.group) last.items.push(setting);
    else acc.push({ group: setting.group, items: [setting] });
    return acc;
  }, []);
}
