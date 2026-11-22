<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import { t } from "../i18n/index.svelte";
  import Row from "./Row.svelte";
  import { openLink } from "../diagnostics/api";
  import { CREDITS, LAUNCHER_REPOSITORY, LAUNCHER_URL, LAUNCHER_VERSION, nonAffiliation } from "../data";

  let { onreportbug }: { onreportbug: () => Promise<void> | void } = $props();
</script>

<h2>{t("launcher.about.heading")}</h2>
<div class="group">
  <Row label={t("launcher.about.name")}><span class="muted">v{LAUNCHER_VERSION}</span></Row>
  <Row label={t("launcher.about.license")}><span class="muted">GNU GPL v3</span></Row>
  <ActionRow label={t("launcher.about.repository")} value={LAUNCHER_REPOSITORY} action={t("launcher.about.open")} onrun={() => openLink(LAUNCHER_URL)} />
  <ActionRow
    label={t("launcher.about.problemLabel")}
    help={t("launcher.about.problemHelp")}
    action={t("launcher.about.reportBug")}
    onrun={onreportbug}
  />
  <p class="muted notice">{nonAffiliation()}</p>
</div>

<h2>{t("launcher.about.credits")}</h2>
<div class="group">
  {#each CREDITS as credit (credit.name)}
    <ActionRow label={credit.name} value={credit.license ?? (credit.licenseKey ? t(credit.licenseKey) : "")} action={t("launcher.about.open")} onrun={() => openLink(credit.url)} />
  {/each}
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
  .muted { color: var(--color-text-muted); }
  .notice { margin: 0; padding: var(--space-3) var(--space-4); border-top: 1px solid var(--color-border); font-size: 12px; }
</style>
