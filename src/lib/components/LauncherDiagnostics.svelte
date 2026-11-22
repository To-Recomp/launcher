<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import { cleanDiagnostics, diagnosticsFolder, revealFile } from "../diagnostics/api";
  import { formatSize } from "../format";
  import { t } from "../i18n/index.svelte";

  let { oncleaned }: { oncleaned: () => void } = $props();

  async function clean() {
    const { files, bytes } = await cleanDiagnostics();
    oncleaned(); // a crash file offered by the banner may be gone
    if (files === 0) return t("launcher.diagnostics.nothing");
    const size = formatSize(bytes);
    return files === 1 ? t("launcher.diagnostics.removedOne", { size }) : t("launcher.diagnostics.removed", { count: files, size });
  }
</script>

<h2>{t("launcher.diagnostics.heading")}</h2>
<div class="group">
  <ActionRow
    label={t("launcher.diagnostics.folderLabel")}
    help={t("launcher.diagnostics.folderHelp")}
    action={t("launcher.diagnostics.open")}
    onrun={async () => revealFile(await diagnosticsFolder())}
  />
  <ActionRow
    label={t("launcher.diagnostics.cleanLabel")}
    help={t("launcher.diagnostics.cleanHelp")}
    action={t("launcher.diagnostics.clean")}
    confirm={t("launcher.diagnostics.confirm")}
    onrun={clean}
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
