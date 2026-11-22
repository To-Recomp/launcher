<script lang="ts">
  import Dropdown from "./Dropdown.svelte";
  import Icon, { type IconName } from "./Icon.svelte";
  import Segmented from "./Segmented.svelte";
  import Toggle from "./Toggle.svelte";
  import { settingsTabs, type SettingsTab } from "../data";

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

  // Static values; real settings come from the game's manifest (#4).
  let windowMode = $state("Windowed");
  let windowSize = $state("1280 × 720");
  let renderResolution = $state("Match window");
  let aspectRatio = $state("Automatic");
  let letterbox = $state(true);
  let frameLimit = $state("Match display");
  let vsync = $state(true);
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

  {#if tab === "Display"}
    <h2>Picture</h2>
    <div class="group">
      <div class="row">
        <span>Window</span>
        <Segmented label="Window" options={["Windowed", "Borderless", "Fullscreen"]} value={windowMode} onchange={(v) => (windowMode = v)} />
      </div>
      <div class="row">
        <span>Window size</span>
        <Dropdown label="Window size" options={["1280 × 720", "1600 × 900", "1920 × 1080"]} value={windowSize} onchange={(v) => (windowSize = v)} />
      </div>
      <div class="row">
        <span>Render resolution</span>
        <Dropdown label="Render resolution" options={["Match window", "Native", "2x"]} value={renderResolution} onchange={(v) => (renderResolution = v)} />
      </div>
      <div class="row">
        <span>Aspect ratio</span>
        <Dropdown label="Aspect ratio" options={["Automatic", "4:3", "16:9"]} value={aspectRatio} onchange={(v) => (aspectRatio = v)} />
      </div>
      <div class="row">
        <span>Letterbox instead of stretching</span>
        <Toggle label="Letterbox instead of stretching" checked={letterbox} onchange={(v) => (letterbox = v)} />
      </div>
    </div>

    <h2>Frame rate</h2>
    <div class="group">
      <div class="row">
        <span>Frame rate limit</span>
        <Dropdown label="Frame rate limit" options={["Match display", "30", "60", "120"]} value={frameLimit} onchange={(v) => (frameLimit = v)} />
      </div>
      <div class="row">
        <span>VSync</span>
        <Toggle label="VSync" checked={vsync} onchange={(v) => (vsync = v)} />
      </div>
    </div>
  {:else}
    <p class="empty">{tab} settings will be generated from the game's manifest.</p>
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
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    min-height: 55px;
  }
  .row + .row { border-top: 1px solid var(--color-border); }
  .empty { margin-top: var(--space-5); color: var(--color-text-muted); }
</style>
