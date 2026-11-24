<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../i18n/index.svelte";
  import { startLanguage } from "../i18n/start";
  import LauncherAbout from "./LauncherAbout.svelte";
  import LauncherAutostart from "./LauncherAutostart.svelte";
  import LauncherCatalog from "./LauncherCatalog.svelte";
  import LauncherDiagnostics from "./LauncherDiagnostics.svelte";
  import LauncherProxy from "./LauncherProxy.svelte";
  import LauncherFolders from "./LauncherFolders.svelte";
  import LauncherUpdates from "./LauncherUpdates.svelte";
  import RestartBanner from "./RestartBanner.svelte";
  import SettingRow from "./SettingRow.svelte";
  import type { CatalogStatus } from "../catalog/status";
  import { nonAffiliation } from "../data";
  import { applyDownloadLimit, loadLauncherSettings, setLauncherSetting } from "../launcher/api";
  import { errorMessage } from "../settings/api";
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
      if (setting.id === "downloadLimit") await applyDownloadLimit();
      if (setting.id === "minimizeToTray") await syncTray();
      if (setting.id === "language") {
        await startLanguage(); // the menus of the notification area follow it as well
        await syncTray();
      }
      actionError = null;
    } catch (e) {
      actionError = errorMessage(e);
      // Show what is really in the file, not the value the player tried.
      await reload();
    }
  }

  /** The manifest's settings by its own group ids; what is shown of each is localized. */
  const sections = $derived(sectionsOf(view?.settings ?? []));
  const itemsOf = (group: string) => sections.find((s) => s.group === group)?.items ?? [];
  /** The cards the launcher always shows, in this order: the manifest fills them with its settings and each
   *  adds rows of its own (Application the games folder, Updates the launcher and the game list, Network the
   *  proxy). A group the manifest adds later gets a card of its own before About. */
  const CARDS = ["Application", "Updates", "Network"];
  const otherSections = $derived(sections.filter((s) => !CARDS.includes(s.group ?? "")));
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
  {/if}

  <h2>{t("launcher.group.Application")}</h2>
  <div class="group">
    {#each itemsOf("Application") as setting (setting.id)}
      {@render row(setting)}
      <!-- Starting with the system belongs right under the language. -->
      {#if setting.id === "language"}<LauncherAutostart />{/if}
    {/each}
    <LauncherFolders />
  </div>

  <h2>{t("launcher.group.Updates")}</h2>
  <div class="group">
    {#each itemsOf("Updates") as setting (setting.id)}{@render row(setting)}{/each}
    <LauncherUpdates />
    <LauncherCatalog status={catalogStatus} refreshing={refreshingCatalog} onrefresh={onrefreshcatalog} />
  </div>

  <LauncherProxy>
    {#each itemsOf("Network") as setting (setting.id)}{@render row(setting)}{/each}
  </LauncherProxy>

  {#each otherSections as section, i (i)}
    {#if section.group}<h2>{localizeSetting(section.items[0]).group}</h2>{/if}
    <div class="group">
      {#each section.items as setting (setting.id)}{@render row(setting)}{/each}
    </div>
  {/each}

  <h2>{t("launcher.about.heading")}</h2>
  <div class="group">
    <LauncherAbout {onreportbug} />
    <LauncherDiagnostics {oncleaned} />
    <p class="notice">{nonAffiliation()}</p>
  </div>
</section>

{#snippet row(setting: SettingView)}
  <SettingRow setting={localizeSetting(setting)} onchange={(input) => apply(setting, input)} />
{/snippet}

<style>
  .settings { padding: var(--space-6); }
  h1 { margin: var(--space-6) 0 var(--space-5); font-size: 40px; font-weight: 700; }
  h2 { margin: var(--space-5) 0 var(--space-2) var(--space-4); font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    /* No overflow clipping: a dropdown's list opens over the edge of its group. */
  }
  .notice { margin: 0; padding: var(--space-3) var(--space-4); border-top: 1px solid var(--color-border); font-size: 12px; color: var(--color-text-muted); }
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
