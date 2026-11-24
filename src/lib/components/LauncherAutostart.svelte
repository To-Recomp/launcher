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

<!-- A row for the card of the "Application" settings; it shows nothing where the system can't start the launcher. -->
{#if supported}
  <Row label={t("launcher.autostart.label")} help={t("launcher.autostart.help")}>
    <Toggle label={t("launcher.autostart.label")} checked={enabled} onchange={change} />
  </Row>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
{/if}

<style>
  .error { margin: 0; padding: var(--space-2) var(--space-4); color: var(--color-primary); }
</style>
