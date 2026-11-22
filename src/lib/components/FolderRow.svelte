<script lang="ts">
  import Row from "./Row.svelte";
  import { pickFolder } from "../launcher/api";

  let {
    label,
    help = null,
    path,
    onpick,
    onreset,
  }: {
    label: string;
    help?: string | null;
    /** The chosen folder; `null` while the default is used. */
    path: string | null;
    onpick: (path: string) => void;
    /** Goes back to the default; offered only while a folder is chosen. */
    onreset?: () => void;
  } = $props();

  async function choose() {
    const picked = await pickFolder(label);
    if (picked !== null) onpick(picked);
  }
</script>

<Row {label} {help}>
  <div class="pick">
    <span class="path" title={path ?? undefined}>{path ?? "Default"}</span>
    <button onclick={choose}>Choose…</button>
    {#if path !== null && onreset}<button onclick={onreset}>Use default</button>{/if}
  </div>
</Row>

<style>
  .path {
    max-width: 320px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl; /* a long path keeps its end visible */
    color: var(--color-text-muted);
    font-size: 13px;
    user-select: text;
  }
  .pick { display: flex; align-items: center; gap: var(--space-3); }
  button {
    padding: 6px 14px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    background: var(--color-surface-alt);
    font-weight: 500;
  }
</style>
