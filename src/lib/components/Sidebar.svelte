<script lang="ts">
  import SidebarItem from "./SidebarItem.svelte";
  import { LAUNCHER_VERSION, type Game } from "../data";

  let { games, selectedId, onselect }: { games: Game[]; selectedId: string; onselect: (id: string) => void } = $props();
</script>

<aside class="sidebar">
  <nav class="list" aria-label="Games">
    {#each games as game (game.id)}
      <SidebarItem {game} selected={game.id === selectedId} onselect={() => onselect(game.id)} />
    {/each}
  </nav>
  <footer>
    <span class="version">v{LAUNCHER_VERSION}</span>
    <button class="update">Update launcher</button>
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
</style>
