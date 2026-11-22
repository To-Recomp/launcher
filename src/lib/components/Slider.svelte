<script lang="ts">
  import { formatNumber } from "../settings/format";

  let {
    min,
    max,
    step,
    value,
    format = null,
    label,
    onchange,
  }: {
    min: number;
    max: number;
    step: number;
    value: number;
    format?: string | null;
    label: string;
    onchange: (value: number) => void;
  } = $props();

  // Follows the thumb while dragging; the change is committed on release.
  let shown = $state(0);
  $effect(() => {
    shown = value;
  });
</script>

<div class="slider">
  <input
    type="range"
    aria-label={label}
    {min}
    {max}
    {step}
    value={shown}
    oninput={(e) => (shown = e.currentTarget.valueAsNumber)}
    onchange={(e) => onchange(e.currentTarget.valueAsNumber)}
  />
  <output>{formatNumber(format, shown)}</output>
</div>

<style>
  .slider { display: flex; align-items: center; gap: var(--space-3); min-width: 260px; }
  input { flex: 1; accent-color: var(--color-info); }
  output { min-width: 4ch; text-align: right; font-variant-numeric: tabular-nums; color: var(--color-text-muted); }
</style>
