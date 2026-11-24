import { hasMessage, t } from "../i18n/index.svelte";
import type { SettingView } from "../settings/types";

const known = (key: string, fallback: string) => (hasMessage(key) ? t(key) : fallback);

/**
 * The launcher's own settings are described in English by a manifest built into the launcher;
 * this shows them in the interface language. What has no message stays as the manifest says it.
 * A note that only concerns Linux (GNOME has no notification area) is added there only.
 */
export function localizeSetting(
  setting: SettingView,
  linux: boolean = typeof navigator !== "undefined" && navigator.userAgent.includes("Linux"),
): SettingView {
  const control =
    setting.control.type === "choices"
      ? {
          ...setting.control,
          options: setting.control.options.map((option) => known(`launcher.option.${option}`, option)),
        }
      : setting.control;
  const help = setting.help === null ? null : known(`launcher.setting.${setting.id}.help`, setting.help);
  const linuxNote = setting.id === "minimizeToTray" && linux ? ` ${t("launcher.setting.minimizeToTray.helpLinux")}` : "";
  return {
    ...setting,
    label: known(`launcher.setting.${setting.id}.label`, setting.label),
    help: help === null ? null : help + linuxNote,
    group: setting.group === null ? null : known(`launcher.group.${setting.group}`, setting.group),
    control,
  };
}
