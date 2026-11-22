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
  if (refreshing) return "Refreshing…";
  if (!status) return "Not refreshed yet";
  if (status.error) return `Couldn't refresh at ${clock(status.at)}. The saved list is used.`;
  return status.updated ? `Updated at ${clock(status.at)}` : `Up to date as of ${clock(status.at)}`;
}
