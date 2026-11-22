<script lang="ts">
  import { onMount } from "svelte";
  import FolderRow from "./FolderRow.svelte";
  import { launcherFolders, setGamesFolder, type Folders } from "../launcher/api";
  import { restartNeeded } from "../launcher/restart.svelte";
  import { errorMessage } from "../settings/api";

  let folders = $state<Folders | null>(null);
  let error = $state<string | null>(null);

  async function show(next: () => Promise<Folders>) {
    try {
      folders = await next();
      error = null;
      // The folder is read at startup: a choice that differs from the one in use waits for a restart.
      if (folders.inUse !== null && (folders.configured ?? folders.default) !== folders.inUse) restartNeeded.mark();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  onMount(() => show(launcherFolders));
</script>

<h2>Storage</h2>
<div class="group">
  <FolderRow
    label="Games folder"
    help="Where games are installed and keep their data. A change applies after a restart; games already installed stay where they are."
    path={folders?.configured ?? null}
    onpick={(folder) => show(() => setGamesFolder(folder))}
    onreset={() => show(() => setGamesFolder(null))}
  />
</div>
{#if folders}<p class="muted current">In use: {folders.inUse ?? "none, see the message below"}</p>{/if}
{#if error}<p class="error" role="alert">{error}</p>{/if}

<style>
  h2 { margin: var(--space-5) 0 var(--space-2) var(--space-4); font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }
  .current { margin: var(--space-2) var(--space-4) 0; font-size: 12px; }
  .muted { color: var(--color-text-muted); }
  .error { margin: var(--space-2) var(--space-4) 0; color: var(--color-primary); }
</style>
