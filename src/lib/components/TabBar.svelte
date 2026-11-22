<script lang="ts" generics="T extends string">
  let {
    tabs,
    value,
    onchange,
  }: { tabs: { id: T; label: string }[]; value: T; onchange: (id: T) => void } = $props();
</script>

<div class="tabs" role="tablist">
  {#each tabs as tab (tab.id)}
    <button
      role="tab"
      aria-selected={tab.id === value}
      class:active={tab.id === value}
      onclick={() => onchange(tab.id)}
    >
      {tab.label}
    </button>
  {/each}
</div>

<style>
  .tabs { display: flex; gap: var(--space-5); height: 100%; }
  button {
    position: relative;
    padding: 0;
    background: none;
    border: 0;
    color: var(--color-text-muted);
    font-weight: 500;
  }
  button:hover { color: var(--color-text); }
  button.active { color: var(--color-text); font-weight: 600; }
  button.active::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 3px;
    border-radius: 2px 2px 0 0;
    background: var(--color-accent);
  }
</style>
