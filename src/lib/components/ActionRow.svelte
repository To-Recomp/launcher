<script lang="ts">
  import Row from "./Row.svelte";
  import { errorMessage } from "../settings/api";

  let {
    label,
    help = null,
    action,
    onrun,
  }: {
    label: string;
    help?: string | null;
    /** The button's text. */
    action: string;
    /** Resolves with a message to show next to the button, or nothing. Rejects with the failure's message. */
    onrun: () => Promise<string | void>;
  } = $props();

  let busy = $state(false);
  let result = $state<{ text: string; failed: boolean } | null>(null);

  async function run() {
    busy = true;
    result = null;
    try {
      const text = await onrun();
      if (text) result = { text, failed: false };
    } catch (e) {
      result = { text: errorMessage(e), failed: true };
    } finally {
      busy = false;
    }
  }
</script>

<Row {label} {help}>
  <div class="action">
    {#if result}
      <span class="result" class:failed={result.failed} role={result.failed ? "alert" : "status"}>{result.text}</span>
    {/if}
    <button disabled={busy} onclick={run}>{action}</button>
  </div>
</Row>

<style>
  .action { display: flex; align-items: center; gap: var(--space-3); }
  .result { color: var(--color-text-muted); font-size: 13px; }
  .result.failed { color: var(--color-primary); }
  button {
    padding: 6px 14px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    background: var(--color-surface-alt);
    font-weight: 500;
  }
  button:disabled { opacity: 0.5; }
</style>
