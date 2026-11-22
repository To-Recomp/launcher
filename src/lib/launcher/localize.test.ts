import { afterEach, describe, expect, it } from "vitest";
import manifest from "../../../src-tauri/src/launcher_settings/manifest.json";
import { catalogs, FALLBACK_LOCALE, hasMessage, setLocale } from "../i18n/index.svelte";
import type { SettingView } from "../settings/types";
import { localizeSetting } from "./localize";

afterEach(() => setLocale(FALLBACK_LOCALE));

const view = (setting: (typeof manifest.settings)[number]): SettingView => ({
  id: setting.id,
  label: setting.label,
  help: setting.help ?? null,
  tab: "Game",
  group: setting.group ?? null,
  advanced: false,
  enabled: true,
  control: { type: "choices", style: "segmented", options: setting.choices.map((c) => c.label), value: 0, custom: null },
});

describe("the launcher's built-in settings", () => {
  it("each have a message for their label, help, group and options, in every shipped language", () => {
    for (const setting of manifest.settings) {
      const keys = [
        `launcher.setting.${setting.id}.label`,
        ...(setting.help ? [`launcher.setting.${setting.id}.help`] : []),
        ...(setting.group ? [`launcher.group.${setting.group}`] : []),
        ...setting.choices.map((c) => `launcher.option.${c.label}`),
      ];
      for (const key of keys) {
        expect(hasMessage(key), `${key} is missing from the English messages`).toBe(true);
        for (const [code, catalog] of Object.entries(catalogs)) {
          expect((catalog as Record<string, string>)[key], `${code}: ${key}`).toBeDefined();
        }
      }
    }
  });

  it("are shown in the interface language", () => {
    const language = manifest.settings.find((s) => s.id === "language")!;
    setLocale("fr");
    const shown = localizeSetting(view(language));
    expect(shown.label).toBe("Langue");
    expect(shown.group).toBe("Interface");
    expect(shown.control.type === "choices" && shown.control.options).toEqual(["Système", "English", "Français"]);
  });

  it("keep what the manifest says when there is no message for it", () => {
    const shown = localizeSetting({ ...view(manifest.settings[0]), id: "unknown", label: "Whatever", help: "Help", group: "Other" });
    expect([shown.label, shown.help, shown.group]).toEqual(["Whatever", "Help", "Other"]);
  });
});
