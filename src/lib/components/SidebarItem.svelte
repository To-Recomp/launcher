<script lang="ts">
  import Icon from "./Icon.svelte";
  import type { Game } from "../catalog/games";
  import { logoFor } from "../catalog/logos";
  import { loadPalette } from "../catalog/palette";
  import { t } from "../i18n/index.svelte";
  import { appearance } from "../launcher/appearance.svelte";

  let { game, selected = false, onselect }: { game: Game; selected?: boolean; onselect: () => void } = $props();

  // Each item tints itself with its own game's colors, so hovering shows that game's, not the selected one's.
  let accent = $state<string | null>(null);
  $effect(() => {
    const logo = logoFor(game.name);
    accent = null;
    if (!logo || !appearance.usesGamePalette) return;
    let stale = false;
    loadPalette(logo).then((found) => {
      if (!stale) accent = found?.accent ?? null;
    });
    return () => (stale = true);
  });
</script>

<button class="item" class:selected style:--item-accent={accent} title={game.name} aria-current={selected ? "true" : undefined} onclick={onselect}>
  <span class="badge">{game.icon}</span>
  <span class="text">
    <span class="name">{game.name}</span>
    <span class="year">{game.year}</span>
  </span>
  <!-- Only a game that can be played at once gets an icon: the others have nothing to offer from the list. -->
  {#if game.state === "ready"}
    <span class="status play" role="img" aria-label={t("shell.sidebar.play")}>
      <Icon name="play" size={18} />
    </span>
  {/if}
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
  /* Hover and selection take the item's own game colors (its logo palette). */
  .item:hover { background: color-mix(in srgb, var(--item-accent, var(--color-accent)) 10%, transparent); }
  .item.selected {
    background: color-mix(in srgb, var(--item-accent, var(--color-accent)) 16%, transparent);
    border-color: color-mix(in srgb, var(--item-accent, var(--color-accent)) 30%, transparent);
  }
  .item::before {
    content: "";
    position: absolute;
    left: -1px;
    top: 14px;
    bottom: 14px;
    width: 3px;
    border-radius: 2px;
    background: var(--item-accent, var(--color-accent));
    opacity: 0;
  }
  .item:hover::before, .item.selected::before { opacity: 1; }
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
