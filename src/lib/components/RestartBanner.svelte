<script lang="ts">
  import { t } from "../i18n/index.svelte";
  import { restartLauncher } from "../launcher/api";
  import { errorMessage } from "../settings/api";

  let failure = $state<string | null>(null);

  async function restart() {
    try {
      await restartLauncher();
    } catch (e) {
      failure = errorMessage(e);
    }
  }
</script>

<div class="banner" role="status">
  <span>{failure ?? t("launcher.restart.message")}</span>
  <button onclick={restart}>{t("launcher.restart.now")}</button>
</div>

<style>
  .banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: var(--space-4);
    padding: var(--space-3) var(--space-4);
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-alt);
  }
  button {
    padding: 6px 14px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    background: var(--color-surface-raised);
    font-weight: 500;
  }
</style>
