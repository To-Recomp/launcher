<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import { launcherUpdates } from "../launcher/updates.svelte";
  import { LAUNCHER_VERSION } from "../data";

  async function check() {
    const found = await launcherUpdates.check();
    return found ? `Version ${found.version} is available.` : "You have the latest version.";
  }
</script>

{#if launcherUpdates.available}
  <div class="group">
    <ActionRow label="Launcher version" value={`v${LAUNCHER_VERSION}`} action="Check now" onrun={check} />
    {#if launcherUpdates.found}
      <ActionRow
        label={`Version ${launcherUpdates.found.version} is available`}
        help="The launcher restarts when it is installed."
        action="Install and restart"
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
