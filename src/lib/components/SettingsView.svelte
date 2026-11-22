<script lang="ts">
  import { untrack } from "svelte";
  import Icon, { type IconName } from "./Icon.svelte";
  import SettingRow from "./SettingRow.svelte";
  import { errorMessage, loadSettings, setSetting } from "../settings/api";
  import type { Location, SettingInput, SettingView, SettingsView } from "../settings/types";
  import { LAUNCHER_VERSION, NON_AFFILIATION, settingsTabs, type SettingsTab } from "../data";

  let {
    gameId,
    gameName,
    location,
    onreportbug,
  }: { gameId: string; gameName: string; location: Location | null; onreportbug: () => void } = $props();

  const icons: Record<SettingsTab, IconName> = {
    Display: "display",
    Graphics: "graphics",
    Audio: "audio",
    Controls: "controls",
    Saves: "saves",
    Game: "game",
    About: "about",
  };

  let tab = $state<SettingsTab>("Display");
  let view = $state<SettingsView | null>(null);
  let loadError = $state<string | null>(null);
  let actionError = $state<string | null>(null);

  const advancedKey = $derived(`settings.showAdvanced.${gameId}`);
  let showAdvanced = $state(false);
  $effect(() => {
    showAdvanced = localStorage.getItem(advancedKey) === "true";
  });
  function toggleAdvanced() {
    showAdvanced = !showAdvanced;
    localStorage.setItem(advancedKey, String(showAdvanced));
  }

  async function reload(current: Location) {
    try {
      const loaded = await loadSettings(current);
      if (current === location) view = loaded;
    } catch (e) {
      if (current === location) loadError = errorMessage(e);
    }
  }

  $effect(() => {
    const current = location;
    untrack(() => {
      view = null;
      loadError = null;
      actionError = null;
    });
    if (current) reload(current);
  });

  async function apply(setting: SettingView, input: SettingInput) {
    if (!location) return;
    try {
      view = await setSetting(location, setting.id, input);
      actionError = null;
    } catch (e) {
      actionError = errorMessage(e);
      // Show what is really in the file, not the value the player tried.
      await reload(location);
    }
  }

  const inTab = $derived((view?.settings ?? []).filter((s) => s.tab === tab));
  const hasAdvanced = $derived(inTab.some((s) => s.advanced));
  const visible = $derived(inTab.filter((s) => showAdvanced || !s.advanced));

  /** Consecutive settings of the same group share a card under one heading. */
  const sections = $derived(
    visible.reduce<{ group: string | null; items: SettingView[] }[]>((acc, setting) => {
      const last = acc.at(-1);
      if (last && last.group === setting.group) last.items.push(setting);
      else acc.push({ group: setting.group, items: [setting] });
      return acc;
    }, []),
  );
</script>

<section class="settings">
  <h1>Settings</h1>

  <div class="subtabs" role="tablist">
    {#each settingsTabs as name (name)}
      <button role="tab" aria-selected={name === tab} class:active={name === tab} onclick={() => (tab = name)}>
        <Icon name={icons[name]} size={18} />{name}
      </button>
    {/each}
  </div>

  {#if actionError}
    <div class="error" role="alert">
      <span>{actionError}</span>
      <button aria-label="Dismiss" onclick={() => (actionError = null)}>×</button>
    </div>
  {/if}

  {#if tab === "About"}
    <div class="group about">
      <div class="row"><span>To Recomp Launcher</span><span class="muted">v{LAUNCHER_VERSION}</span></div>
      <div class="row"><span>License</span><span class="muted">GNU GPL v3</span></div>
      <div class="row">
        <span>Something went wrong?<span class="muted help"> Creates a file you can review and paste into a GitHub issue.</span></span>
        <button class="report" onclick={onreportbug}>Report a bug</button>
      </div>
      <p class="muted notice">{NON_AFFILIATION}</p>
    </div>
  {:else if !location}
    <p class="empty">Install and prepare {gameName} to configure it.</p>
  {:else if loadError}
    <p class="empty" role="alert">{loadError}</p>
  {:else if !view}
    <p class="empty">Loading…</p>
  {:else if sections.length === 0}
    <p class="empty">No settings in this section.</p>
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

  {#if hasAdvanced && view}
    <button class="advanced" onclick={toggleAdvanced}>
      {showAdvanced ? "Hide advanced options" : "Show advanced options"}
    </button>
  {/if}
</section>

<style>
  .settings { padding: var(--space-6); }
  h1 { margin: var(--space-6) 0 var(--space-5); font-size: 40px; font-weight: 700; }
  h2 { margin: var(--space-5) 0 var(--space-2) var(--space-4); font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .subtabs {
    display: inline-flex;
    gap: var(--space-1);
    padding: 4px;
    border-radius: var(--radius-pill);
    background: var(--color-bg-sunken);
    border: 1px solid var(--color-border);
  }
  .subtabs button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 8px 14px;
    border: 1px solid transparent;
    border-radius: var(--radius-pill);
    background: none;
    color: var(--color-text-muted);
    font-weight: 500;
  }
  .subtabs button.active {
    background: var(--color-surface-raised);
    border-color: var(--color-border-strong);
    color: var(--color-text);
  }
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }
  .about { margin-top: var(--space-5); }
  .row { display: flex; align-items: center; justify-content: space-between; padding: var(--space-3) var(--space-4); }
  .row + .row { border-top: 1px solid var(--color-border); }
  .muted { color: var(--color-text-muted); }
  .help { display: block; font-size: 12px; }
  .report {
    padding: 6px 14px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    background: var(--color-surface-alt);
    font-weight: 500;
  }
  .notice { margin: 0; padding: var(--space-3) var(--space-4); border-top: 1px solid var(--color-border); font-size: 12px; }
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
  .advanced {
    margin-top: var(--space-5);
    padding: 8px 16px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    background: var(--color-surface-alt);
    font-weight: 500;
  }
</style>
