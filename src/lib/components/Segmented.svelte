<script lang="ts">
  let {
    options,
    value,
    label,
    onchange,
  }: { options: string[]; value: string; label: string; onchange: (value: string) => void } = $props();
</script>

<div class="segmented" role="radiogroup" aria-label={label}>
  {#each options as option (option)}
    <button
      role="radio"
      aria-checked={option === value}
      class:active={option === value}
      onclick={() => onchange(option)}
    >
      {option}
    </button>
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    padding: 4px;
    border-radius: var(--radius-pill);
    background: var(--color-bg-sunken);
  }
  button {
    padding: 6px 20px;
    border: 1px solid transparent;
    border-radius: var(--radius-pill);
    background: none;
    color: var(--color-text-muted);
    font-size: 13px;
    font-weight: 500;
  }
  button.active {
    background: var(--color-surface-raised);
    border-color: var(--color-border-strong);
    color: var(--color-text);
  }
</style>
