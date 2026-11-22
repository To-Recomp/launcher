<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../i18n/index.svelte";
  import LauncherAbout from "./LauncherAbout.svelte";
  import LauncherBehavior from "./LauncherBehavior.svelte";
  import LauncherCatalog from "./LauncherCatalog.svelte";
  import LauncherDiagnostics from "./LauncherDiagnostics.svelte";
  import LauncherProxy from "./LauncherProxy.svelte";
  import LauncherFolders from "./LauncherFolders.svelte";
  import LauncherUpdates from "./LauncherUpdates.svelte";
  import RestartBanner from "./RestartBanner.svelte";
  import SettingRow from "./SettingRow.svelte";
  import type { CatalogStatus } from "../catalog/status";
  import { applyDownloadLimit, loadLauncherSettings, setLauncherSetting } from "../launcher/api";
  import { errorMessage } from "../settings/api";
  import { appearance } from "../launcher/appearance.svelte";
  import { localizeSetting } from "../launcher/localize";
  import { restartNeeded } from "../launcher/restart.svelte";
  import { syncTray } from "../launcher/tray";
  import { launcherUpdates } from "../launcher/updates.svelte";
  import { sectionsOf } from "../settings/sections";
  import type { SettingInput, SettingView, SettingsView } from "../settings/types";

  let {
    catalogStatus,
    refreshingCatalog,
    onrefreshcatalog,
    onreportbug,
    oncleaned,
  }: {
    catalogStatus: CatalogStatus | null;
    refreshingCatalog: boolean;
    onrefreshcatalog: () => Promise<void>;
    onreportbug: () => Promise<void> | void;
    /** Saved diagnostic files were deleted. */
    oncleaned: () => void;
  } = $props();

  let view = $state<SettingsView | null>(null);
  let loadError = $state<string | null>(null);
  let actionError = $state<string | null>(null);

  async function reload() {
    try {
      view = await loadLauncherSettings();
    } catch (e) {
      loadError = errorMessage(e);
    }
  }

  onMount(reload);

  async function apply(setting: SettingView, input: SettingInput) {
    try {
      view = await setLauncherSetting(setting.id, input);
      if (setting.id === "updateChannel") launcherUpdates.forget();
      if (setting.id === "theme" || setting.id === "gamePalette") await appearance.reload();
      if (setting.id === "downloadLimit") await applyDownloadLimit();
      if (setting.id === "minimizeToTray") await syncTray();
      if (setting.id === "language") restartNeeded.mark(); // the language is chosen when the launcher starts
      actionError = null;
    } catch (e) {
      actionError = errorMessage(e);
      // Show what is really in the file, not the value the player tried.
      await reload();
    }
  }

  const sections = $derived(sectionsOf((view?.settings ?? []).map(localizeSetting)));
</script>

<section class="settings">
  <h1>{t("launcher.settings.title")}</h1>

  {#if restartNeeded.needed}<RestartBanner />{/if}

  {#if actionError}
    <div class="error" role="alert">
      <span>{actionError}</span>
      <button aria-label={t("launcher.settings.dismiss")} onclick={() => (actionError = null)}>×</button>
    </div>
  {/if}

  {#if loadError}
    <p class="empty" role="alert">{loadError}</p>
  {:else if !view}
    <p class="empty">{t("launcher.settings.loading")}</p>
  {:else}
    {#each sections as section, i (i)}
      {#if section.group}<h2>{section.group}</h2>{/if}
      <div class="group">
        {#each section.items as setting (setting.id)}
          <SettingRow {setting} onchange={(input) => apply(setting, input)} />
        {/each}
      </div>
    {/each}
  {/if}

  <LauncherBehavior />

  <LauncherUpdates />

  <LauncherCatalog status={catalogStatus} refreshing={refreshingCatalog} onrefresh={onrefreshcatalog} />

  <LauncherFolders />

  <LauncherProxy />

  <LauncherDiagnostics {oncleaned} />

  <LauncherAbout {onreportbug} />
</section>

<style>
  .settings { padding: var(--space-6); }
  h1 { margin: var(--space-6) 0 var(--space-5); font-size: 40px; font-weight: 700; }
  h2 { margin: var(--space-5) 0 var(--space-2) var(--space-4); font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }
  .empty { margin-top: var(--space-5); color: var(--color-text-muted); }
  .error {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: var(--space-4);
    padding: var(--space-3) var(--space-4);
    border: 1px solid rgb(200 48 44 / 0.6);
    border-radius: var(--radius-md);
    background: rgb(200 48 44 / 0.15);
  }
  .error button { border: 0; background: none; font-size: 18px; color: var(--color-text-muted); }
</style>
