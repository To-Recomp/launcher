<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import { revealFile } from "../diagnostics/api";
  import { errorMessage } from "../settings/api";
  import { deleteSave, exportSaves, importSaves, listSaves, pickBackup, pickBackupTarget, type SavesView } from "../settings/saves";
  import type { Location } from "../settings/types";
  import { locale, t } from "../i18n/index.svelte";

  let { location, gameId }: { location: Location; gameId: string } = $props();

  let view = $state<SavesView | null>(null);
  let loadError = $state<string | null>(null);

  async function reload() {
    const current = location;
    try {
      const loaded = await listSaves(current);
      if (current === location) view = loaded;
    } catch (e) {
      if (current === location) loadError = errorMessage(e);
    }
  }

  $effect(() => {
    location; // a new location, a new list
    view = null;
    loadError = null;
    reload();
  });

  const empty = $derived(view !== null && view.folders.every((f) => f.files.length === 0));

  function size(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  const details = (file: { size: number; modified: number | null }) =>
    [size(file.size), file.modified === null ? null : new Date(file.modified * 1000).toLocaleString(locale())]
      .filter(Boolean)
      .join(" · ");

  async function backup(): Promise<string | void> {
    const target = await pickBackupTarget(t("game.saves.backupDialog"), `${gameId}-saves`);
    if (!target) return;
    return t("game.saves.backedUp", { count: await exportSaves(location, target) });
  }

  async function restore(): Promise<string | void> {
    const source = await pickBackup(t("game.saves.restoreDialog"));
    if (!source) return;
    const count = await importSaves(location, source);
    await reload();
    return t("game.saves.restored", { count });
  }
</script>

{#if loadError}
  <p class="empty" role="alert">{loadError}</p>
{:else if !view}
  <p class="empty">{t("game.settings.loading")}</p>
{:else}
  <h2>{t("game.saves.backup")}</h2>
  <div class="group">
    <ActionRow label={t("game.saves.backupLabel")} help={t("game.saves.backupHelp")} action={t("game.saves.backupButton")} onrun={backup} />
    <ActionRow label={t("game.saves.restoreLabel")} help={t("game.saves.restoreHelp")} action={t("game.saves.restoreButton")} onrun={restore} />
  </div>

  {#each view.folders as folder, index (index)}
    <h2>{t("game.saves.folder")}</h2>
    <div class="group">
      <ActionRow label={t("game.saves.location")} value={folder.path} action={t("game.about.open")} onrun={() => revealFile(folder.exists ? folder.path : folder.path.replace(/[\\/][^\\/]*$/, ""))} />
      {#each folder.files as file (file.name)}
        <ActionRow
          label={file.name}
          value={details(file)}
          action={t("game.saves.delete")}
          confirm={t("game.saves.deleteConfirm")}
          onrun={async () => {
            await deleteSave(location, index, file.name);
            await reload();
          }}
        />
      {/each}
    </div>
  {/each}
  {#if empty}<p class="empty">{t("game.saves.none")}</p>{/if}
{/if}

<style>
  h2 { margin: var(--space-5) 0 var(--space-2) var(--space-4); font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--game-card-top, var(--color-surface)) 55%, transparent);
  }
  .empty { margin-top: var(--space-5); color: var(--color-text-muted); }
</style>
