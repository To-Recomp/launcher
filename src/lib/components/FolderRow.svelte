<script lang="ts">
  import Row from "./Row.svelte";
  import { pickFolder } from "../launcher/api";

  let {
    label,
    help = null,
    path,
    onpick,
  }: { label: string; help?: string | null; path: string | null; onpick: (path: string) => void } = $props();

  async function choose() {
    const picked = await pickFolder(label);
    if (picked !== null) onpick(picked);
  }
</script>

<Row {label} {help}>
  <div class="pick">
    <span class="path" title={path ?? undefined}>{path ?? "Default"}</span>
    <button onclick={choose}>Choose…</button>
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
