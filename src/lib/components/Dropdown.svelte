<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    options,
    value,
    label,
    caption,
    onchange,
  }: {
    options: string[];
    value: string;
    label: string;
    /** Small label shown above the value (Home platform selector). */
    caption?: string;
    onchange: (value: string) => void;
  } = $props();
</script>

<label class="dropdown" class:captioned={caption}>
  {#if caption}<span class="caption">{caption}</span>{/if}
  <select aria-label={label} {value} onchange={(e) => onchange(e.currentTarget.value)}>
    {#each options as option (option)}
      <option value={option}>{option}</option>
    {/each}
  </select>
  <span class="chevron"><Icon name="chevron-down" size={16} /></span>
</label>

<style>
  .dropdown {
    position: relative;
    display: inline-flex;
    flex-direction: column;
    min-width: 216px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-control);
  }
  .dropdown.captioned { min-width: 132px; border-radius: var(--radius-md); }
  .caption { padding: 8px 14px 0; font-size: 11px; color: var(--color-text-muted); }
  select {
    appearance: none;
    width: 100%;
    padding: 8px 36px 8px 14px;
    border: 0;
    background: transparent;
    font-size: 13px;
  }
  .captioned select { padding-top: 2px; font-weight: 600; }
  select option { background: var(--color-control); }
  .chevron {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    pointer-events: none;
    color: var(--color-text-muted);
  }
</style>
