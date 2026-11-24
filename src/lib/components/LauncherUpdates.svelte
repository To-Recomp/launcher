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

<!-- Rows of the "Updates" card; nothing where this installation can't update itself. -->
{#if launcherUpdates.available}
  <ActionRow label={t("launcher.updates.version")} value={`v${LAUNCHER_VERSION}`} action={t("launcher.updates.checkNow")} onrun={check} />
  {#if launcherUpdates.found}
    <ActionRow
      label={t("launcher.updates.available", { version: launcherUpdates.found.version })}
      help={t("launcher.updates.restartHelp")}
      action={t("launcher.updates.install")}
      onrun={() => launcherUpdates.install()}
    />
  {/if}
{/if}
