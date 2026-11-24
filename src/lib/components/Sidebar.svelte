<script lang="ts">
  import Icon from "./Icon.svelte";
  import SidebarItem from "./SidebarItem.svelte";
  import type { Game } from "../catalog/games";
  import { LAUNCHER_VERSION } from "../data";
  import { launcherUpdates, type LauncherUpdates } from "../launcher/updates.svelte";
  import { errorMessage } from "../settings/api";
  import { t } from "../i18n/index.svelte";

  let {
    games,
    selectedId,
    onselect,
    onplay,
    launcherActive = false,
    onlauncher,
    loading = false,
    catalogError = null,
    onretry = () => {},
    updates = launcherUpdates,
  }: {
    games: Game[];
    selectedId: string;
    onselect: (id: string) => void;
    /** Quick play of a game: its last played console. */
    onplay: (id: string) => void;
    /** The launcher's own settings page is open. */
    launcherActive?: boolean;
    /** Opens the launcher's own settings page. */
    onlauncher: () => void;
    loading?: boolean;
    /** Why the catalog couldn't be refreshed (the saved list, if any, is shown). */
    catalogError?: string | null;
    onretry?: () => void;
    updates?: LauncherUpdates;
  } = $props();

  let status = $state<"idle" | "checking" | "current" | "failed">("idle");
  let failure = $state("");

  async function update() {
    status = "checking";
    try {
      await updates.install(); // restarts into the new version when there is one
      status = "current";
    } catch (e) {
      failure = errorMessage(e);
      status = "failed";
    }
  }

  const labels = $derived({
    idle: t("shell.update.idle"),
    checking: t("shell.update.checking"),
    current: t("shell.update.current"),
    failed: t("shell.update.failed"),
  });
  const label = $derived(
    status === "idle" && updates.found ? t("shell.update.to", { version: updates.found.version }) : labels[status],
  );
</script>

<aside class="sidebar">
  <nav class="list" aria-label={t("shell.sidebar.games")}>
    {#each games as game (game.id)}
      <SidebarItem {game} selected={game.id === selectedId} onselect={() => onselect(game.id)} onplay={() => onplay(game.id)} />
    {/each}
    {#if games.length === 0}
      <p class="empty">{loading ? t("shell.sidebar.loading") : t("shell.sidebar.unavailable")}</p>
    {/if}
  </nav>
  {#if catalogError && !loading}
    <div class="note" role="status">
      <span>{games.length ? t("shell.sidebar.savedList") : ""}{catalogError}</span>
      <button onclick={onretry}>{t("shell.sidebar.retry")}</button>
    </div>
  {/if}
  <footer>
    <span class="version">v{LAUNCHER_VERSION}</span>
    <span class="tools">
      {#if updates.available}
        <button class="update" disabled={status === "checking"} title={status === "failed" ? failure : undefined} onclick={update}>
          {label}
        </button>
      {/if}
      <button
        class="gear"
        class:active={launcherActive}
        aria-label={t("shell.sidebar.launcherSettings")}
        title={t("shell.sidebar.launcherSettings")}
        aria-pressed={launcherActive}
        onclick={onlauncher}
      >
        <Icon name="gear" size={16} />
      </button>
    </span>
  </footer>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: max-content;
    min-width: var(--sidebar-min-width);
    max-width: var(--sidebar-max-width);
    flex: none;
    background: var(--color-bg);
    border-right: 1px solid var(--color-border);
  }
  .list {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-2);
    overflow-y: auto;
  }
  .empty { margin: var(--space-4); color: var(--color-text-muted); }
  .note {
    /* Its text wraps to the width the game names give: it never widens the sidebar. */
    display: flex;
    width: 0;
    min-width: 100%;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-top: 1px solid var(--color-border);
    font-size: 11px;
    color: var(--color-text-muted);
  }
  .note button {
    flex: none;
    padding: 2px 8px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-sm);
    background: none;
    color: var(--color-accent);
    font-size: 11px;
  }
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-2) var(--space-2) var(--space-4);
    border-top: 1px solid var(--color-border);
  }
  .version { font-size: 12px; color: var(--color-text-muted); }
  .tools { display: flex; align-items: center; gap: var(--space-2); }
  .gear {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--color-text-muted);
  }
  .gear:hover, .gear.active { background: var(--color-surface-alt); color: var(--color-text); }
  .update {
    padding: 6px 12px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-sm);
    background: var(--color-surface-alt);
    color: var(--color-accent);
    font-size: 12px;
    font-weight: 600;
  }
  .update:disabled { opacity: 0.6; cursor: default; }
</style>
