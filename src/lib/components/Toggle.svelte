<script lang="ts">
  let { checked, label, onchange }: { checked: boolean; label: string; onchange: (checked: boolean) => void } = $props();
</script>

<button class="toggle" role="switch" aria-checked={checked} aria-label={label} onclick={() => onchange(!checked)}>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    position: relative;
    width: 44px;
    height: 26px;
    padding: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-pill);
    background: var(--color-control);
    transition: background 0.15s;
  }
  .toggle[aria-checked="true"] { background: var(--color-info); }
  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--color-text);
    transition: transform 0.15s;
  }
  .toggle[aria-checked="true"] .knob { transform: translateX(18px); }
</style>
