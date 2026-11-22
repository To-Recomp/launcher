<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import { t } from "../i18n/index.svelte";
  import { launcherUpdates } from "../launcher/updates.svelte";
  import { LAUNCHER_VERSION } from "../data";

  async function check() {
    const found = await launcherUpdates.check();
    return found ? t("launcher.updates.availableResult", { version: found.version }) : t("launcher.updates.latest");
  }
</script>

{#if launcherUpdates.available}
  <div class="group">
    <ActionRow label={t("launcher.updates.version")} value={`v${LAUNCHER_VERSION}`} action={t("launcher.updates.checkNow")} onrun={check} />
    {#if launcherUpdates.found}
      <ActionRow
        label={t("launcher.updates.available", { version: launcherUpdates.found.version })}
        help={t("launcher.updates.restartHelp")}
        action={t("launcher.updates.install")}
        onrun={() => launcherUpdates.install()}
      />
    {/if}
  </div>
{/if}

<style>
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }
</style>
