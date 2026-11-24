<script lang="ts">
  import Icon from "./Icon.svelte";
  import type { Game } from "../catalog/games";
  import { logoFor } from "../catalog/logos";
  import { loadPalette } from "../catalog/palette";
  import { t } from "../i18n/index.svelte";
  import { appearance } from "../launcher/appearance.svelte";

  let {
    game,
    selected = false,
    onselect,
    onplay,
  }: {
    game: Game;
    selected?: boolean;
    onselect: () => void;
    /** Quick play: starts the console the game was last played on. Offered for a game that can be played at once. */
    onplay?: () => void;
  } = $props();

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

<div class="row" class:selected style:--item-accent={accent}>
  <button class="item" title={game.name} aria-current={selected ? "true" : undefined} onclick={onselect}>
    <span class="badge">{game.icon}</span>
    <span class="text">
      <span class="name">{game.name}</span>
      <span class="year">{game.year}</span>
    </span>
  </button>
  <!-- A separate button, so a click plays without opening the game's page first. -->
  {#if game.state === "ready" && onplay}
    <button
      class="play"
      aria-label={t("shell.sidebar.quickPlay", { game: game.name })}
      title={t("shell.sidebar.quickPlay", { game: game.name })}
      onclick={onplay}
    >
      <Icon name="play" size={18} />
    </button>
  {/if}
</div>

<style>
  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding-right: var(--space-3);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
  }
  /* Hover and selection take the item's own game colors (its logo palette). */
  .row:hover { background: color-mix(in srgb, var(--item-accent, var(--color-accent)) 10%, transparent); }
  .row.selected {
    background: color-mix(in srgb, var(--item-accent, var(--color-accent)) 16%, transparent);
    border-color: color-mix(in srgb, var(--item-accent, var(--color-accent)) 30%, transparent);
  }
  .row::before {
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
  .row:hover::before, .row.selected::before { opacity: 1; }
  .item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex: 1;
    min-width: 0;
    padding: var(--space-2) 0 var(--space-2) var(--space-4);
    background: none;
    border: 0;
    text-align: left;
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
  .play {
    display: grid;
    place-items: center;
    flex: none;
    width: 34px;
    height: 34px;
    border: 1px solid color-mix(in srgb, var(--color-accent) 40%, transparent);
    border-radius: var(--radius-sm);
    background: var(--color-surface-alt);
    color: var(--color-accent);
  }
  .play:hover { background: color-mix(in srgb, var(--color-accent) 18%, var(--color-surface-alt)); }
</style>
