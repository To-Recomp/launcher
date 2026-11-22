<script lang="ts">
  import HomeView from "./lib/components/HomeView.svelte";
  import SettingsView from "./lib/components/SettingsView.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import { games, NON_AFFILIATION, type TabId } from "./lib/data";
  import { locationFor } from "./lib/instances";

  let selectedId = $state("eternia");
  let tab = $state<TabId>("home");

  const game = $derived(games.find((g) => g.id === selectedId)!);
  const location = $derived(locationFor(game));
</script>

<div class="app">
  <Sidebar {games} {selectedId} onselect={(id) => (selectedId = id)} />
  <div class="main">
    <TitleBar {tab} onchange={(t) => (tab = t)} />
    <main>
      {#if tab === "home"}
        <HomeView {game} />
      {:else}
        <SettingsView gameId={game.id} gameName={game.name} location={location} />
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
  footer { padding: var(--space-4) var(--space-6); font-size: 11px; color: var(--color-text-muted); }
</style>
