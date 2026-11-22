<script lang="ts">
  import Icon from "./Icon.svelte";
  import type { Game } from "../catalog/games";

  let { game, selected = false, onselect }: { game: Game; selected?: boolean; onselect: () => void } = $props();

  const action = $derived(
    game.state === "ready"
      ? { icon: "play" as const, label: "Play" }
      : game.state === "installed"
        ? { icon: "settings" as const, label: "Settings" }
        : { icon: "download" as const, label: "Install" },
  );
</script>

<button class="item" class:selected aria-current={selected ? "true" : undefined} onclick={onselect}>
  <span class="badge">{game.icon}</span>
  <span class="text">
    <span class="name">{game.name}</span>
    <span class="year">{game.year}</span>
    <span class="platforms">
      {#each game.platforms as p (p.platform + p.region)}
        <span class="chip {p.status}" title="{p.platform} ({p.region}): {p.status}{p.tested ? '' : ', untested'}">
          <span class="dot"></span>{p.platform}
        </span>
      {/each}
    </span>
  </span>
  <span class="status" class:play={game.state === "ready"} role="img" aria-label={action.label}>
    <Icon name={action.icon} size={18} />
  </span>
</button>

<style>
  .item {
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-2) var(--space-3) var(--space-2) var(--space-4);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    text-align: left;
  }
  .item:hover { background: var(--color-surface-alt); }
  .item.selected { background: var(--color-selected); border-color: var(--color-border); }
  .item.selected::before {
    content: "";
    position: absolute;
    left: -1px;
    top: 14px;
    bottom: 14px;
    width: 3px;
    border-radius: 2px;
    background: var(--color-accent);
  }
  .badge {
    display: grid;
    place-items: center;
    flex: none;
    width: 44px;
    height: 44px;
    border-radius: var(--radius-sm);
    background: var(--color-surface-raised);
    font-size: 20px;
    font-weight: 600;
  }
  .text { display: flex; flex-direction: column; flex: 1; min-width: 0; }
  .name { font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .year { font-size: 12px; color: var(--color-text-muted); }
  .platforms { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 3px; }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 6px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-pill);
    font-size: 10px;
    color: var(--color-text-muted);
  }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--status-color); }
  .chip.stable { --status-color: var(--color-stable); }
  .chip.beta { --status-color: var(--color-beta); }
  .chip.alpha { --status-color: var(--color-alpha); }
  .chip.unsupported { --status-color: var(--color-unsupported); }
  .chip.coming-soon { --status-color: var(--color-coming-soon); }
  .status {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-alt);
    color: var(--color-text-muted);
  }
  .status.play { color: var(--color-accent); border-color: color-mix(in srgb, var(--color-accent) 40%, transparent); }
</style>
