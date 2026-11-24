<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import { t } from "../i18n/index.svelte";
  import Row from "./Row.svelte";
  import { openLink } from "../diagnostics/api";
  import { CREDITS, LAUNCHER_REPOSITORY, LAUNCHER_URL, LAUNCHER_VERSION } from "../data";

  let { onreportbug }: { onreportbug: () => Promise<void> | void } = $props();
</script>

<!-- Rows of the "About" card: the launcher, what it is built with, and how to report a problem. -->
<Row label={t("launcher.about.name")}><span class="muted">v{LAUNCHER_VERSION}</span></Row>
<Row label={t("launcher.about.license")}><span class="muted">GNU GPL v3</span></Row>
<ActionRow label={t("launcher.about.repository")} value={LAUNCHER_REPOSITORY} action={t("launcher.about.open")} onrun={() => openLink(LAUNCHER_URL)} />
{#each CREDITS as credit (credit.name)}
  <ActionRow label={t("launcher.about.builtWith", { name: credit.name })} value={credit.license} action={t("launcher.about.open")} onrun={() => openLink(credit.url)} />
{/each}
<ActionRow
  label={t("launcher.about.problemLabel")}
  help={t("launcher.about.problemHelp")}
  action={t("launcher.about.reportBug")}
  onrun={onreportbug}
/>

<style>
  .muted { color: var(--color-text-muted); }
</style>
