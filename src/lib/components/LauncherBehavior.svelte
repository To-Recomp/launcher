<script lang="ts">
  import { onMount } from "svelte";
  import Row from "./Row.svelte";
  import Toggle from "./Toggle.svelte";
  import { autostartEnabled, autostartSupported, setAutostart } from "../launcher/api";
  import { t } from "../i18n/index.svelte";
  import { errorMessage } from "../settings/api";

  let supported = $state(false);
  let enabled = $state(false);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      supported = await autostartSupported();
      if (supported) enabled = await autostartEnabled();
    } catch {
      supported = false; // no Tauri runtime
    }
  });

  async function change(on: boolean) {
    try {
      enabled = await setAutostart(on);
      error = null;
    } catch (e) {
      error = errorMessage(e);
    }
  }
</script>

{#if supported}
  <div class="group">
    <Row label={t("launcher.autostart.label")} help={t("launcher.autostart.help")}>
      <Toggle label={t("launcher.autostart.label")} checked={enabled} onchange={change} />
    </Row>
  </div>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
{/if}

<style>
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }
  .error { margin: var(--space-2) var(--space-4) 0; color: var(--color-primary); }
</style>
