<script lang="ts">
  import { onMount } from "svelte";
  import FolderRow from "./FolderRow.svelte";
  import { t } from "../i18n/index.svelte";
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

<!-- A row of the "Application" card, with the folder in use under it. -->
<FolderRow
  label={t("launcher.folders.gamesLabel")}
  help={t("launcher.folders.gamesHelp")}
  path={folders?.configured ?? null}
  onpick={(folder) => show(() => setGamesFolder(folder))}
  onreset={() => show(() => setGamesFolder(null))}
/>
{#if folders}<p class="muted current">{t("launcher.folders.inUse", { folder: folders.inUse ?? t("launcher.folders.inUseNone") })}</p>{/if}
{#if error}<p class="error" role="alert">{error}</p>{/if}

<style>
  .current { margin: 0; padding: 0 var(--space-4) var(--space-3); font-size: 12px; }
  .muted { color: var(--color-text-muted); }
  .error { margin: 0; padding: 0 var(--space-4) var(--space-3); color: var(--color-primary); }
</style>
