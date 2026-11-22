<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import { describeCatalog, type CatalogStatus } from "../catalog/status";

  let {
    status,
    refreshing,
    onrefresh,
  }: {
    status: CatalogStatus | null;
    refreshing: boolean;
    onrefresh: () => Promise<void>;
  } = $props();
</script>

<h2>Game list</h2>
<div class="group">
  <ActionRow
    label="Game catalog"
    help={status?.error ?? "Downloaded from the To-Recomp catalog when the launcher starts."}
    value={describeCatalog(status, refreshing)}
    action="Refresh"
    onrun={onrefresh}
  />
</div>

<style>
  h2 { margin: var(--space-5) 0 var(--space-2) var(--space-4); font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }
</style>
