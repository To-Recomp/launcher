<script lang="ts">
  import { onMount } from "svelte";
  import CrashBanner from "./lib/components/CrashBanner.svelte";
  import DiagnosticDialog from "./lib/components/DiagnosticDialog.svelte";
  import HomeView from "./lib/components/HomeView.svelte";
  import LauncherSettingsView from "./lib/components/LauncherSettingsView.svelte";
  import SettingsView from "./lib/components/SettingsView.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import { cachedCatalog, refreshCatalog } from "./lib/catalog/api";
  import { toGames, type Game } from "./lib/catalog/games";
  import { logoFor } from "./lib/catalog/logos";
  import type { CatalogStatus } from "./lib/catalog/status";
  import { loadPalette, paletteStyle, type Palette } from "./lib/catalog/palette";
  import { nonAffiliation, type TabId } from "./lib/data";
  import { t } from "./lib/i18n/index.svelte";
  import { instanceLocation } from "./lib/library/api";
  import { appearance } from "./lib/launcher/appearance.svelte";
  import { syncTray } from "./lib/launcher/tray";
  import { launcherUpdates } from "./lib/launcher/updates.svelte";
  import { LibraryStore } from "./lib/library/store.svelte";
  import type { Location } from "./lib/settings/types";
  import { errorMessage } from "./lib/settings/api";

  const library = new LibraryStore();

  let catalog = $state<Game[]>([]);
  let loading = $state(true);
  let catalogLoads = 0;
  let catalogError = $state<string | null>(null);
  let catalogStatus = $state<CatalogStatus | null>(null);
  let selectedId = $state<string | null>(null);
  let tab = $state<TabId>("home");

  // The sidebar's icon says whether the game is installed or ready to play.
  const games = $derived(catalog.map((g) => ({ ...g, state: library.gameState(g.id) })));

  // The first game until the player picks one (or the picked one left the catalog).
  const game = $derived(games.find((g) => g.id === selectedId) ?? games[0]);

  // The page takes its colors from the game's logo; until they are read (or without a logo) the theme stays.
  let palette = $state<Palette | null>(null);
  $effect(() => {
    palette = null;
    const logo = game ? logoFor(game.name) : undefined;
    if (!logo) return;
    let stale = false;
    loadPalette(logo).then((found) => {
      if (!stale) palette = found;
    });
    return () => (stale = true);
  });

  let platformByGame = $state<Record<string, string>>({});
  const platform = $derived(game?.platforms.find((p) => p.platform === platformByGame[game.id]) ?? game?.platforms[0]);

  /** Where the selected instance keeps its settings, once its game is installed. */
  let location = $state<Location | null>(null);
  $effect(() => {
    const current = game && platform ? { id: game.id, platform: platform.platform } : null;
    library.installationOf(current?.id ?? "")?.tag; // reload when the game gets (re)installed
    location = null;
    if (!current) return;
    let stale = false;
    instanceLocation(current.id, current.platform)
      .then((found) => !stale && (location = found))
      .catch(() => {});
    return () => (stale = true);
  });

  /** The saved list shows at once; the download then replaces it if something newer exists. */
  async function loadCatalog() {
    if (catalogLoads > 0) return; // a refresh is already under way
    catalogLoads++;
    loading = true;
    try {
      const saved = await cachedCatalog();
      if (saved) catalog = toGames(saved);
    } catch {
      // No saved list yet: the download below decides.
    }
    try {
      const result = await refreshCatalog();
      if (result.catalog) catalog = toGames(result.catalog);
      catalogError = result.error;
      catalogStatus = { at: new Date(), updated: result.updated, error: result.error };
    } catch (e) {
      catalogError = errorMessage(e);
      catalogStatus = { at: new Date(), updated: false, error: catalogError };
    }
    loading = false;
    catalogLoads--;
  }

  onMount(() => {
    library.start();
    launcherUpdates.start();
    syncTray().catch(() => {}); // no notification area where there is no Tauri runtime
    loadCatalog();
    return () => library.stop();
  });
</script>

<div class="app" style={paletteStyle(tab === "launcher" || !appearance.usesGamePalette ? null : palette)}>
  <Sidebar
    {games}
    selectedId={tab === "launcher" ? "" : (game?.id ?? "")}
    onselect={(id) => {
      selectedId = id;
      if (tab === "launcher") tab = "home"; // picking a game leaves the launcher's own page
    }}
    {loading}
    {catalogError}
    onretry={loadCatalog}
  />
  <div class="main">
    <TitleBar {tab} onchange={(next) => (tab = next)} />
    {#if library.crash}
      <CrashBanner
        file={library.crash}
        onview={() => library.viewCrash()}
        ondismiss={() => library.dismissCrash()}
      />
    {/if}
    <main>
      {#if tab === "launcher"}
        <LauncherSettingsView
          catalogStatus={catalogStatus}
          refreshingCatalog={loading}
          onrefreshcatalog={loadCatalog}
          onreportbug={() => library.reportBug()}
        />
      {:else if !game}
        <p class="empty">{t("shell.app.pickGame")}</p>
      {:else if tab === "home"}
        <HomeView {game} {platform} {library} onplatform={(label) => (platformByGame[game.id] = label)} />
      {:else}
        <SettingsView
          gameId={game.id}
          gameName={game.name}
          repository={game.repository}
          platform={platform?.platform ?? null}
          tag={library.installationOf(game.id)?.tag ?? null}
          {location}
          onreportbug={() => library.reportGameBug(game, platform)}
        />
      {/if}
    </main>
    <footer>{nonAffiliation()}</footer>
  </div>
  {#if library.diagnostic}
    <DiagnosticDialog diagnostic={library.diagnostic} onclose={() => library.closeDiagnostic()} />
  {/if}
</div>

<style>
  .app { display: flex; height: 100%; }
  .main {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    background:
      radial-gradient(720px 420px at 100% 0, var(--game-glow, transparent), transparent),
      linear-gradient(180deg, var(--game-bg-top, var(--page-top)) 0%, var(--game-bg-mid, var(--page-mid)) 60%, var(--color-bg) 100%);
  }
  main { flex: 1; overflow-y: auto; }
  .empty { padding: var(--space-6); color: var(--color-text-muted); }
  footer { padding: var(--space-4) var(--space-6); font-size: 11px; color: var(--color-text-muted); }
</style>
