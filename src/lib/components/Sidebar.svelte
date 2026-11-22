<script lang="ts">
  import SidebarItem from "./SidebarItem.svelte";
  import { LAUNCHER_VERSION, type Game } from "../data";
  import { errorMessage } from "../settings/api";
  import { updateLauncher, updaterAvailable } from "../updater";

  let { games, selectedId, onselect }: { games: Game[]; selectedId: string; onselect: (id: string) => void } = $props();

  let available = $state(false);
  let status = $state<"idle" | "checking" | "current" | "failed">("idle");
  let failure = $state("");
  $effect(() => {
    updaterAvailable().then((yes) => (available = yes));
  });

  async function update() {
    status = "checking";
    try {
      await updateLauncher(); // restarts into the new version when there is one
      status = "current";
    } catch (e) {
      failure = errorMessage(e);
      status = "failed";
    }
  }

  const labels = { idle: "Update launcher", checking: "Checking…", current: "Up to date", failed: "Update failed" };
</script>

<aside class="sidebar">
  <nav class="list" aria-label="Games">
    {#each games as game (game.id)}
      <SidebarItem {game} selected={game.id === selectedId} onselect={() => onselect(game.id)} />
    {/each}
  </nav>
  <footer>
    <span class="version">v{LAUNCHER_VERSION}</span>
    {#if available}
      <button class="update" disabled={status === "checking"} title={status === "failed" ? failure : undefined} onclick={update}>
        {labels[status]}
      </button>
    {/if}
  </footer>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: var(--sidebar-width);
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
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-3) var(--space-3) var(--space-4);
    border-top: 1px solid var(--color-border);
  }
  .version { font-size: 12px; color: var(--color-text-muted); }
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
