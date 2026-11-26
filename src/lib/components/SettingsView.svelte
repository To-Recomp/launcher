<script lang="ts">
  import { onMount, untrack } from "svelte";
  import GameAbout from "./GameAbout.svelte";
  import Icon, { type IconName } from "./Icon.svelte";
  import Row from "./Row.svelte";
  import SettingRow from "./SettingRow.svelte";
  import { errorMessage, loadSettings, setSetting } from "../settings/api";
  import { sectionsOf } from "../settings/sections";
  import type { Location, SettingInput, SettingView, SettingsView } from "../settings/types";
  import { settingsTabs, type SettingsTab } from "../data";
  import { t, type MessageKey } from "../i18n/index.svelte";

  let {
    gameId,
    gameName,
    repository,
    platform,
    tag,
    location,
    files,
    onback,
    onreportbug,
  }: {
    gameId: string;
    gameName: string;
    repository: string;
    platform: string | null;
    /** The installed release; `null` when the game isn't installed. */
    tag: string | null;
    location: Location | null;
    /** The files the game is built from; `onchange` chooses another file for one of them (nothing is built). */
    files: {
      hasInputs: boolean;
      current: { id: string; label: string; path: string | null }[];
      busy: boolean;
      onchange: (inputId: string) => void;
      /** What the last choice said, until the settings are left (`clearNotice`). */
      notice: { tone: "info" | "success" | "error"; text: string } | null;
      clearNotice: () => void;
    };
    /** Back to the game's page. */
    onback: () => void;
    /** Settings → About → Report a bug, for this game. */
    onreportbug: () => Promise<void> | void;
  } = $props();

  /** The manifest's tabs; the game's files are a section of "Game". */
  type Tab = SettingsTab;
  const tabs: Tab[] = [...settingsTabs];

  const icons: Record<Tab, IconName> = {
    Display: "display",
    Graphics: "graphics",
    Audio: "audio",
    Controls: "controls",
    Saves: "saves",
    Game: "game",
    About: "about",
  };

  /** The tab names are ids matched with game manifests; only the label shown is translated. */
  const tabLabels: Record<Tab, MessageKey> = {
    Display: "game.settings.tab.display",
    Graphics: "game.settings.tab.graphics",
    Audio: "game.settings.tab.audio",
    Controls: "game.settings.tab.controls",
    Saves: "game.settings.tab.saves",
    Game: "game.settings.tab.game",
    About: "game.settings.tab.about",
  };

  let tab = $state<Tab>("Display");
  onMount(() => () => files.clearNotice()); // what the last choice said isn't kept for the next visit
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

  /** How much of the end of a path is always shown: enough for the end of the file name and its extension. */
  const PATH_END = 22;

  /**
   * A path as its start and its end: the end (the last characters, the extension with them) never shrinks, the
   * start takes what room is left and is cut with an ellipsis, so the ellipsis falls in the middle.
   */
  function splitPath(path: string): [string, string] {
    const cut = Math.max(0, path.length - PATH_END);
    return [path.slice(0, cut), path.slice(cut)];
  }

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

  const sections = $derived(sectionsOf(visible));

  /** A tab without any setting is disabled; "Game" also counts the files it shows, and About is always there. */
  function enabled(name: Tab): boolean {
    if (name === "About") return true;
    if (name === "Game" && files.hasInputs) return true;
    return (view?.settings ?? []).some((s) => s.tab === name);
  }
  // Once the settings are known, leave a disabled tab for the first one that has something. Without settings
  // (not installed, unreadable), the tab stays: the message under it says why.
  $effect(() => {
    if (view !== null && !enabled(tab)) tab = tabs.find(enabled) ?? "About";
  });
</script>

<section class="settings">
  <button class="back" onclick={onback}>
    <span class="back-icon"><Icon name="chevron-down" size={16} /></span>{t("game.settings.back")}
  </button>
  <h1>{t("game.settings.title")}</h1>

  <div class="subtabs" role="tablist">
    {#each tabs as name (name)}
      <button
        role="tab"
        aria-selected={name === tab}
        class:active={name === tab}
        disabled={!enabled(name)}
        onclick={() => (tab = name)}
      >
        <Icon name={icons[name]} size={18} />{t(tabLabels[name])}
      </button>
    {/each}
  </div>

  {#if actionError}
    <div class="error" role="alert">
      <span>{actionError}</span>
      <button aria-label={t("game.settings.dismiss")} onclick={() => (actionError = null)}>×</button>
    </div>
  {/if}

  {#if tab === "Game" && files.hasInputs}
    <!-- Shown even before the game is prepared: it is where the files are chosen. A game without files has no section. -->
    <h2>{t("game.settings.files.title")}</h2>
    <div class="group">
      {#each files.current as file (file.id)}
        <Row label={file.label} labelMax="22%">
          <div class="file-control">
            {#if file.path}
              {@const [head, tail] = splitPath(file.path)}
              <span class="path" title={file.path}>
                <span class="head">{head}</span><span class="tail">{tail}</span>
              </span>
            {:else}
              <span class="path none">{t("game.settings.files.notChosen")}</span>
            {/if}
            <button
              class="small"
              disabled={files.busy}
              title={t("game.settings.files.help")}
              aria-label={t("game.settings.files.change", { file: file.label })}
              onclick={() => files.onchange(file.id)}
            >
              {t("game.settings.files.changeButton")}
            </button>
          </div>
        </Row>
      {/each}
    </div>
    {#if files.notice}
      <p class="file-notice {files.notice.tone}" role={files.notice.tone === "error" ? "alert" : "status"}>{files.notice.text}</p>
    {/if}
  {/if}

  {#if tab === "About"}
    <GameAbout name={gameName} {repository} {platform} {tag} {location} {onreportbug} />
  {:else if !location}
    <p class="empty">{t("game.settings.installFirst", { game: gameName })}</p>
  {:else if loadError}
    <p class="empty" role="alert">{loadError}</p>
  {:else if !view}
    <p class="empty">{t("game.settings.loading")}</p>
  {:else if sections.length === 0 && !(tab === "Game" && files.hasInputs)}
    <p class="empty">{t("game.settings.noSettings")}</p>
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
      {showAdvanced ? t("game.settings.hideAdvanced") : t("game.settings.showAdvanced")}
    </button>
  {/if}
</section>

<style>
  /* The page follows the game's palette: its buttons (here and in About) are tinted with the accent. */
  .settings {
    --button-bg: color-mix(in srgb, var(--color-accent) 12%, transparent);
    --button-border: color-mix(in srgb, var(--color-accent) 45%, transparent);
    padding: var(--space-6);
  }
  h1 { margin: var(--space-4) 0 var(--space-5); font-size: 40px; font-weight: 700; }
  h2 { margin: var(--space-5) 0 var(--space-2) var(--space-4); font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .back {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 0;
    border: 0;
    background: none;
    color: var(--color-text-muted);
    font-weight: 500;
  }
  .back:hover { color: var(--color-accent); }
  .back-icon { display: inline-grid; transform: rotate(90deg); }
  /* The page follows the game's palette: surfaces are its shades, selections its accent. */
  .subtabs {
    display: flex;
    /* One line whatever the OS's text width: the tabs share the bar, and scroll in a very narrow window. */
    overflow-x: auto;
    width: 100%;
    gap: 2px;
    padding: 4px;
    border-radius: var(--radius-md);
    background: var(--game-bg-mid, var(--color-bg-sunken));
    border: 1px solid var(--color-border);
  }
  .subtabs button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    flex: 1 1 auto;
    justify-content: center;
    padding: 8px 10px;
    white-space: nowrap;
    outline-offset: -2px; /* inside: the bar clips what is outside it */
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    background: none;
    color: var(--color-text-muted);
    font-weight: 500;
  }
  .subtabs button:hover:not(:disabled) { color: var(--color-text); }
  .subtabs button:disabled { opacity: 0.4; cursor: default; }
  .subtabs button.active {
    background: color-mix(in srgb, var(--color-accent) 18%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 45%, transparent);
    color: var(--color-text);
  }
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--game-card-top, var(--color-surface)) 55%, transparent);
    /* No overflow clipping: a dropdown's list opens over the edge of its group. */
  }
  .file-notice { margin: var(--space-2) 0 0 var(--space-4); font-size: 12px; color: var(--color-text-muted); }
  .file-notice.success { color: var(--color-success); }
  .file-notice.error { color: var(--color-error); }
  .file-control { display: flex; align-items: center; gap: var(--space-3); width: 100%; min-width: 0; }
  /* The path takes the room the (short) label leaves: its start (as much as fits) and its end are shown, and
     the ellipsis is between them. */
  .path { display: flex; flex: 1; min-width: 0; font-size: 13px; user-select: text; color: var(--color-text-muted); white-space: nowrap; }
  .head { flex: 0 1 auto; min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .tail { flex: none; }
  .path.none { font-style: italic; }
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
  .small {
    padding: 6px 14px;
    border: 1px solid var(--button-border);
    border-radius: var(--radius-pill);
    background: var(--button-bg);
    font-weight: 500;
  }
  .small:disabled { opacity: 0.5; cursor: default; }
  .advanced {
    margin-top: var(--space-5);
    padding: 8px 16px;
    border: 1px solid var(--button-border);
    border-radius: var(--radius-pill);
    background: var(--button-bg);
    font-weight: 500;
  }
</style>
