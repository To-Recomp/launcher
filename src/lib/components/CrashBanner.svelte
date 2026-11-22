<script lang="ts">
  import { copyText, type DiagnosticFile } from "../diagnostics/api";
  import { errorMessage } from "../settings/api";
  import { t } from "../i18n/index.svelte";

  let {
    file,
    onview,
    ondismiss,
  }: { file: DiagnosticFile; onview: () => void; ondismiss: () => void } = $props();

  let note = $state<string | null>(null);

  async function copy() {
    try {
      await copyText(file.text);
      note = t("game.crash.copied");
    } catch (e) {
      note = errorMessage(e);
    }
  }
</script>

<div class="banner" role="alert">
  <div class="body">
    <strong>{t("game.crash.title")}</strong>
    <span class="muted">{t("game.crash.saved")}{note ? ` ${note}` : ""}</span>
  </div>
  <button onclick={onview}>{t("game.crash.view")}</button>
  <button onclick={copy}>{t("game.crash.copy")}</button>
  <button class="dismiss" aria-label={t("game.crash.dismiss")} onclick={ondismiss}>×</button>
</div>

<style>
  .banner {
    display: flex;
    gap: var(--space-2);
    align-items: center;
    margin: var(--space-3) var(--space-6) 0;
    padding: var(--space-3) var(--space-4);
    border: 1px solid rgb(200 48 44 / 0.6);
    border-radius: var(--radius-md);
    background: rgb(200 48 44 / 0.15);
  }
  .body { display: flex; flex: 1; flex-direction: column; min-width: 0; }
  .muted { font-size: 12px; color: var(--color-text-muted); }
  button {
    padding: 5px 12px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-sm);
    background: none;
    font-size: 12px;
  }
  .dismiss { border: 0; font-size: 18px; color: var(--color-text-muted); }
</style>
