<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "./Icon.svelte";
  import TabBar from "./TabBar.svelte";
  import { tabs, type TabId } from "../data";

  let { tab, onchange }: { tab: TabId; onchange: (tab: TabId) => void } = $props();

  // Outside Tauri (plain `vite dev`, tests) there is no window to control.
  const win = "__TAURI_INTERNALS__" in window ? getCurrentWindow() : null;
</script>

<header class="titlebar" data-tauri-drag-region>
  <TabBar {tabs} value={tab} {onchange} />
  <div class="controls">
    <button aria-label="Minimize" onclick={() => win?.minimize()}><Icon name="minimize" size={16} /></button>
    <button aria-label="Maximize" onclick={() => win?.toggleMaximize()}><Icon name="maximize" size={14} /></button>
    <button aria-label="Close" class="close" onclick={() => win?.close()}><Icon name="close" size={16} /></button>
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    height: var(--titlebar-height);
    padding-left: var(--space-6);
    flex: none;
  }
  .controls { display: flex; align-items: center; gap: var(--space-1); padding-right: var(--space-3); }
  .controls button {
    display: grid;
    place-items: center;
    width: 36px;
    height: 32px;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--color-text-muted);
  }
  .controls button:hover { background: var(--color-surface-alt); color: var(--color-text); }
  .controls .close:hover { background: var(--color-primary); color: #fff; }
</style>
