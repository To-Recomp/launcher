<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import { t } from "../i18n/index.svelte";
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

<!-- A row of the "Updates" card: the game list is refreshed like the launcher is checked. -->
<ActionRow
  label={t("launcher.catalog.label")}
  help={status?.error ?? t("launcher.catalog.help")}
  value={describeCatalog(status, refreshing)}
  action={t("launcher.catalog.refresh")}
  onrun={onrefresh}
/>
