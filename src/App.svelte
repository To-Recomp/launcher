<script lang="ts">
  import { onMount } from "svelte";
  import HomeView from "./lib/components/HomeView.svelte";
  import SettingsView from "./lib/components/SettingsView.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import { cachedCatalog, refreshCatalog } from "./lib/catalog/api";
  import { toGames, type Game } from "./lib/catalog/games";
  import { NON_AFFILIATION, type TabId } from "./lib/data";
  import { locationFor } from "./lib/instances";
  import { errorMessage } from "./lib/settings/api";

  let games = $state<Game[]>([]);
  let loading = $state(true);
  let catalogError = $state<string | null>(null);
  let selectedId = $state<string | null>(null);
  let tab = $state<TabId>("home");

  // The first game until the player picks one (or the picked one left the catalog).
  const game = $derived(games.find((g) => g.id === selectedId) ?? games[0]);
  const location = $derived(game ? locationFor(game) : null);

  /** The saved list shows at once; the download then replaces it if something newer exists. */
  async function loadCatalog() {
    loading = true;
    try {
      const saved = await cachedCatalog();
      if (saved) games = toGames(saved);
    } catch {
      // No saved list yet: the download below decides.
    }
    try {
      const result = await refreshCatalog();
      if (result.catalog) games = toGames(result.catalog);
      catalogError = result.error;
    } catch (e) {
      catalogError = errorMessage(e);
    }
    loading = false;
  }

  onMount(loadCatalog);
</script>

<div class="app">
  <Sidebar
    {games}
    selectedId={game?.id ?? ""}
    onselect={(id) => (selectedId = id)}
    {loading}
    {catalogError}
    onretry={loadCatalog}
  />
  <div class="main">
    <TitleBar {tab} onchange={(t) => (tab = t)} />
    <main>
      {#if !game}
        <p class="empty">Pick a game once the list has loaded.</p>
      {:else if tab === "home"}
        <HomeView {game} />
      {:else}
        <SettingsView gameId={game.id} gameName={game.name} {location} />
      {/if}
    </main>
    <footer>{NON_AFFILIATION}</footer>
  </div>
</div>

<style>
  .app { display: flex; height: 100%; }
  .main {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    background: linear-gradient(180deg, #1d2a40 0%, #171a2b 60%, var(--color-bg) 100%);
  }
  main { flex: 1; overflow-y: auto; }
  .empty { padding: var(--space-6); color: var(--color-text-muted); }
  footer { padding: var(--space-4) var(--space-6); font-size: 11px; color: var(--color-text-muted); }
</style>
