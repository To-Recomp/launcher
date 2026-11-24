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
  import { lastPlayedPlatform } from "./lib/library/lastPlayed";
  import { defaultPlatform, quickPlayPlatform, toGames, type Game } from "./lib/catalog/games";
  import { logoFor } from "./lib/catalog/logos";
  import type { CatalogStatus } from "./lib/catalog/status";
  import { loadPalette, paletteStyle, type Palette } from "./lib/catalog/palette";
  import { nonAffiliation, type View } from "./lib/data";
  import { t } from "./lib/i18n/index.svelte";
  import { instanceLocation } from "./lib/library/api";
  import { gameJob, installJob, instanceId, prepareJob } from "./lib/library/types";
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
  let view = $state<View>("game");

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
  const platform = $derived(
    game &&
      (game.platforms.find((p) => p.platform === platformByGame[game.id]) ??
        defaultPlatform(game, lastPlayedPlatform(game.id))),
  );

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

  /** Every console of every game: the platform selector keeps the width of the widest label. */
  const platformSizing = $derived(
    games.flatMap((g) => g.platforms.map((p) => ({ option: p.platform, prefix: String(p.year) }))),
  );

  /** The files the selected game is built from, for its settings' "Files" tab. */
  const files = $derived.by(() => {
    const installation = game ? library.installationOf(game.id) : undefined;
    const instance = platform ? instanceId(platform.platform) : "";
    const used = installation?.instances.find((i) => i.id === instance)?.inputs ?? {};
    return {
      hasInputs: (installation?.inputs.filter((i) => !i.optional).length ?? 0) > 0,
      /** Each file the game asks for and the path it was prepared from, if any. */
      current: (installation?.inputs ?? []).map((input) => ({ label: input.label, path: used[input.id] ?? null })),
      busy:
        !game ||
        [installJob(game.id), prepareJob(game.id, instance), gameJob(game.id, instance)].some((job) =>
          library.isRunning(job),
        ),
      onchange: () => {
        if (!game || !platform) return;
        view = "game"; // the preparation reports its progress on the game's page
        void library.prepare(game, platform);
      },
    };
  });

  // Each installed game is checked once for a newer release; the game's page then offers the update.
  $effect(() => {
    for (const installation of library.installations) {
      const installed = catalog.find((g) => g.id === installation.gameId);
      if (installed) void library.checkUpdate(installed);
    }
  });

  /** Quick play from the sidebar: starts the console the game was last played on (else its most recent ready one). */
  function quickPlay(id: string) {
    const target = games.find((g) => g.id === id);
    if (!target) return;
    const chosen = quickPlayPlatform(target, (p) => library.instanceState(id, p) === "ready", lastPlayedPlatform(id));
    if (!chosen || library.isRunning(gameJob(id, instanceId(chosen.platform)))) return;
    selectedId = id; // the game's page shows how it went
    view = "game";
    platformByGame[id] = chosen.platform;
    void library.play(target, chosen);
  }
</script>

<div class="app" style={paletteStyle(view === "launcher" || !appearance.usesGamePalette ? null : palette)}>
  <Sidebar
    {games}
    selectedId={view === "launcher" ? "" : (game?.id ?? "")}
    onselect={(id) => {
      selectedId = id;
      view = "game"; // picking a game leaves the launcher's settings and the previous game's
    }}
    onplay={quickPlay}
    launcherActive={view === "launcher"}
    onlauncher={() => (view = view === "launcher" ? "game" : "launcher")}
    {loading}
    {catalogError}
    onretry={loadCatalog}
  />
  <div class="main">
    <TitleBar />
    {#if library.crash}
      <CrashBanner
        file={library.crash}
        onview={() => library.viewCrash()}
        ondismiss={() => library.dismissCrash()}
      />
    {/if}
    <main>
      {#if view === "launcher"}
        <LauncherSettingsView
          catalogStatus={catalogStatus}
          refreshingCatalog={loading}
          onrefreshcatalog={loadCatalog}
          onreportbug={() => library.reportBug()}
          oncleaned={() => library.dismissCrash()}
        />
      {:else if !game}
        <p class="empty">{t("shell.app.pickGame")}</p>
      {:else if view === "game"}
        <HomeView
          {game}
          {platform}
          {library}
          {platformSizing}
          onplatform={(label) => (platformByGame[game.id] = label)}
          onsettings={() => (view = "settings")}
        />
      {:else}
        <SettingsView
          gameId={game.id}
          gameName={game.name}
          repository={game.repository}
          platform={platform?.platform ?? null}
          tag={library.installationOf(game.id)?.tag ?? null}
          {location}
          {files}
          onback={() => (view = "game")}
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
