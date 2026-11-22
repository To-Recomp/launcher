<script lang="ts">
  import { keyName, mouseName, wheelName } from "../settings/keys";
  import type { ActionView } from "../settings/types";

  let {
    actions,
    max,
    onchange,
  }: { actions: ActionView[]; max: number; onchange: (action: string, keys: string[]) => void } = $props();

  // `index` is the chip being replaced, or the next free slot while adding.
  let listening = $state<{ action: string; index: number } | null>(null);

  function capture(action: ActionView, name: string | null) {
    const target = listening;
    listening = null;
    if (!target || !name) return;
    const keys = [...action.keys];
    keys[target.index] = name;
    const unique = keys.filter((k, i) => keys.indexOf(k) === i);
    if (unique.join("|") !== action.keys.join("|")) onchange(action.key, unique);
  }

  function isListening(action: ActionView, index: number) {
    return listening?.action === action.key && listening.index === index;
  }

  const same = (a: string[], b: string[]) => a.join("|") === b.join("|");
</script>

<div class="bindings">
  {#each actions as action (action.key)}
    <div class="action">
      <span class="label">{action.label}</span>
      <div class="chips">
        {#each action.keys as key, index (index)}
          <span class="chip-group">
            <button
              class="chip"
              class:listening={isListening(action, index)}
              aria-label={isListening(action, index) ? `Press a key for ${action.label}` : `${action.label}: ${key}`}
              onclick={() => (listening = { action: action.key, index })}
              onkeydown={(e) => {
                if (!isListening(action, index)) return;
                e.preventDefault();
                capture(action, keyName(e));
              }}
              onmousedown={(e) => {
                if (!isListening(action, index)) return;
                e.preventDefault();
                capture(action, mouseName(e));
              }}
              onwheel={(e) => {
                if (!isListening(action, index)) return;
                e.preventDefault();
                capture(action, wheelName(e));
              }}
              onblur={() => isListening(action, index) && (listening = null)}
              oncontextmenu={(e) => isListening(action, index) && e.preventDefault()}
            >
              {isListening(action, index) ? "Press a key…" : key}
            </button>
            <button
              class="remove"
              aria-label="Remove {key} from {action.label}"
              onclick={() => onchange(action.key, action.keys.filter((_, i) => i !== index))}>×</button
            >
          </span>
        {/each}
        {#if action.keys.length < max}
          {#if isListening(action, action.keys.length)}
            <button
              class="chip listening"
              aria-label="Press a key for {action.label}"
              onkeydown={(e) => {
                e.preventDefault();
                capture(action, keyName(e));
              }}
              onmousedown={(e) => {
                e.preventDefault();
                capture(action, mouseName(e));
              }}
              onwheel={(e) => {
                e.preventDefault();
                capture(action, wheelName(e));
              }}
              onblur={() => (listening = null)}
              oncontextmenu={(e) => e.preventDefault()}
            >
              Press a key…
            </button>
          {:else}
            <button
              class="add"
              aria-label="Add a key to {action.label}"
              onclick={() => (listening = { action: action.key, index: action.keys.length })}>+</button
            >
          {/if}
        {/if}
        {#if !same(action.keys, action.default)}
          <button class="reset" onclick={() => onchange(action.key, action.default)}>Reset</button>
        {/if}
      </div>
    </div>
  {/each}
</div>

<style>
  .bindings { display: flex; flex-direction: column; gap: var(--space-2); min-width: 320px; }
  .action { display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); }
  .label { color: var(--color-text-muted); font-size: 13px; }
  .chips { display: flex; align-items: center; gap: var(--space-2); }
  .chip-group { display: inline-flex; }
  .chip, .add, .remove, .reset {
    border: 1px solid var(--color-border);
    background: var(--color-control);
    font-size: 12px;
  }
  .chip { padding: 5px 12px; border-radius: var(--radius-pill) 0 0 var(--radius-pill); font-weight: 600; }
  .chip.listening { border-color: var(--color-accent); color: var(--color-accent); border-radius: var(--radius-pill); }
  .remove { padding: 5px 8px; border-left: 0; border-radius: 0 var(--radius-pill) var(--radius-pill) 0; color: var(--color-text-muted); }
  .add { width: 28px; padding: 5px 0; border-radius: 50%; }
  .reset { padding: 5px 10px; border-radius: var(--radius-pill); color: var(--color-accent); }
</style>
