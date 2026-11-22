import { t } from "../i18n/index.svelte";

/** How the last refresh of the game list went. */
export interface CatalogStatus {
  at: Date;
  /** A newer list was downloaded. */
  updated: boolean;
  /** Why the refresh failed; the saved list, if any, stays in use. */
  error: string | null;
}

const clock = (at: Date) => at.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });

/** One line for the settings: what happened at the last refresh, or that one is under way. */
export function describeCatalog(status: CatalogStatus | null, refreshing: boolean): string {
  if (refreshing) return t("launcher.catalog.refreshing");
  if (!status) return t("launcher.catalog.never");
  const time = clock(status.at);
  if (status.error) return t("launcher.catalog.failed", { time });
  return status.updated ? t("launcher.catalog.updated", { time }) : t("launcher.catalog.upToDate", { time });
}
