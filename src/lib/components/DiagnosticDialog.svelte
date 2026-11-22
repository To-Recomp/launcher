<script lang="ts">
  import { copyText, openIssues, revealFile } from "../diagnostics/api";
  import type { Diagnostic } from "../library/store.svelte";
  import { errorMessage } from "../settings/api";

  let { diagnostic, onclose }: { diagnostic: Diagnostic; onclose: () => void } = $props();

  let copied = $state(false);
  let problem = $state<string | null>(null);

  async function run(action: () => Promise<void>) {
    problem = null;
    try {
      await action();
    } catch (e) {
      problem = errorMessage(e);
    }
  }

  async function copy() {
    if (!diagnostic.file) return;
    const { text } = diagnostic.file;
    await run(async () => {
      await copyText(text);
      copied = true;
    });
  }
</script>

<div class="backdrop">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="diagnostic-title">
    <h2 id="diagnostic-title">Diagnostic file</h2>
    {#if diagnostic.file}
      {@const file = diagnostic.file}
      <p class="warning" role="note">Review this file before you publish it: it will be public.</p>
      <p class="muted">
        Saved as <code>{file.name}</code>. Nothing was sent: paste it into an issue of
        <strong>{diagnostic.repository}</strong>.
      </p>
      <pre class="preview" aria-label="File content">{file.text}</pre>
    {:else}
      <p role="alert">{diagnostic.error}</p>
    {/if}
    {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
    {#if copied}<p class="muted" role="status">Copied to the clipboard.</p>{/if}
    <div class="buttons">
      {#if diagnostic.file}
        {@const file = diagnostic.file}
        <button class="primary" onclick={copy}>Copy</button>
        <button onclick={() => run(() => revealFile(file.path))}>Open folder</button>
        <button onclick={() => run(() => openIssues(diagnostic.repository))}>Open issues</button>
      {/if}
      <button onclick={onclose}>Close</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.6);
  }
  .dialog {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    width: min(720px, calc(100vw - 48px));
    max-height: calc(100vh - 48px);
    padding: var(--space-5);
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-lg);
    background: var(--color-surface-raised);
  }
  h2 { margin: 0; font-size: 22px; }
  p { margin: 0; }
  .muted { font-size: 12px; color: var(--color-text-muted); }
  .warning { font-weight: 600; }
  .problem { color: var(--color-text); padding: var(--space-2) var(--space-3); border-radius: var(--radius-sm); background: rgb(200 48 44 / 0.15); }
  .preview {
    flex: 1;
    min-height: 120px;
    margin: 0;
    padding: var(--space-3) var(--space-4);
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-sunken);
    font-size: 11px;
    user-select: text;
    white-space: pre-wrap;
  }
  .buttons { display: flex; gap: var(--space-2); justify-content: flex-end; }
  .buttons button {
    padding: 8px 16px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-alt);
  }
  .buttons .primary { border: 0; background: var(--color-primary); font-weight: 600; }
</style>
