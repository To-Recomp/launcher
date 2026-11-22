<script lang="ts">
  import Dropdown from "./Dropdown.svelte";
  import KeyBindings from "./KeyBindings.svelte";
  import Segmented from "./Segmented.svelte";
  import Slider from "./Slider.svelte";
  import TextField from "./TextField.svelte";
  import Toggle from "./Toggle.svelte";
  import type { SettingInput, SettingView } from "../settings/types";

  let { setting, onchange }: { setting: SettingView; onchange: (input: SettingInput) => void } = $props();

  const control = $derived(setting.control);
  const choose = (options: string[], label: string) => onchange({ type: "choice", index: options.indexOf(label) });
</script>

<div class="row" class:disabled={!setting.enabled}>
  <div class="text">
    <span>{setting.label}</span>
    {#if setting.help}<small>{setting.help}</small>{/if}
  </div>
  <div class="control" inert={!setting.enabled} aria-disabled={!setting.enabled}>
    {#if control.type === "choices"}
      {@const custom = control.value === null ? (control.custom ?? "Custom") : null}
      {#if control.style === "toggle"}
        <Toggle label={setting.label} checked={control.value === 1} onchange={(on) => onchange({ type: "choice", index: on ? 1 : 0 })} />
      {:else if control.style === "segmented"}
        <Segmented
          label={setting.label}
          options={control.options}
          value={control.value === null ? "" : control.options[control.value]}
          onchange={(label) => choose(control.options, label)}
        />
      {:else}
        <Dropdown
          label={setting.label}
          options={custom === null ? control.options : [...control.options, custom]}
          value={custom ?? control.options[control.value ?? 0]}
          onchange={(label) => label !== custom && choose(control.options, label)}
        />
      {/if}
    {:else if control.type === "slider"}
      <Slider
        label={setting.label}
        min={control.min}
        max={control.max}
        step={control.step}
        value={control.value}
        format={control.format}
        onchange={(value) => onchange({ type: "number", value })}
      />
    {:else if control.type === "text"}
      <TextField label={setting.label} value={control.value} onchange={(value) => onchange({ type: "text", value })} />
    {:else}
      <KeyBindings
        actions={control.actions}
        max={control.max}
        onchange={(action, keys) => onchange({ type: "bindings", action, keys })}
      />
    {/if}
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
