<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    label,
    help = null,
    disabled = false,
    children,
  }: { label: string; help?: string | null; disabled?: boolean; children: Snippet } = $props();
</script>

<div class="row" class:disabled>
  <div class="text">
    <span>{label}</span>
    {#if help}<small>{help}</small>{/if}
  </div>
  <div class="control" inert={disabled} aria-disabled={disabled}>
    {@render children()}
  </div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
    min-height: 55px;
  }
  .row { border-top: 1px solid var(--color-border); }
  .row:first-child { border-top: 0; }
  .text { display: flex; flex-direction: column; gap: 2px; }
  small { color: var(--color-text-muted); font-size: 12px; }
  .row.disabled { opacity: 0.45; }
</style>
