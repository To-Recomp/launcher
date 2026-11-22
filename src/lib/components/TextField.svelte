<script lang="ts">
  let {
    value,
    label,
    maxlength = 256,
    onchange,
  }: { value: string; label: string; maxlength?: number; onchange: (value: string) => void } = $props();
</script>

<input
  class="text"
  type="text"
  aria-label={label}
  {value}
  {maxlength}
  spellcheck="false"
  onchange={(e) => onchange(e.currentTarget.value)}
/>

<style>
  .text {
    min-width: 216px;
    padding: 8px 14px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-control);
    font: inherit;
    font-size: 13px;
    color: inherit;
    user-select: text;
  }
</style>
