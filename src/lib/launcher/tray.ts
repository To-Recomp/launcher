import { t } from "../i18n/index.svelte";
import { applyBehavior } from "./api";

/** Makes the notification area icon match the settings, with its menu in the interface language. */
export const syncTray = () => applyBehavior({ show: t("launcher.tray.show"), quit: t("launcher.tray.quit") });
