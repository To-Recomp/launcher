<script module lang="ts">
  let counter = 0;
</script>

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

  const id = `dropdown-${counter++}`;

  let root = $state<HTMLDivElement>();
  let menu = $state<HTMLUListElement>();
  let open = $state(false);
  /** The highlighted option while the list is open (keyboard or pointer). */
  let active = $state(0);

  function show() {
    active = Math.max(0, options.indexOf(value));
    open = true;
  }

  function choose(index: number) {
    open = false;
    if (options[index] !== undefined && options[index] !== value) onchange(options[index]);
  }

  function onkeydown(e: KeyboardEvent) {
    const last = options.length - 1;
    if (!open) {
      if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
        e.preventDefault();
        show();
      }
      return;
    }
    switch (e.key) {
      case "ArrowDown": active = Math.min(last, active + 1); break;
      case "ArrowUp": active = Math.max(0, active - 1); break;
      case "Home": active = 0; break;
      case "End": active = last; break;
      case "Enter":
      case " ": choose(active); break;
      case "Escape": open = false; break;
      case "Tab": open = false; return;
      default: return;
    }
    e.preventDefault();
  }

  // Keep the highlighted option in view in a long list.
  $effect(() => {
    if (open) menu?.children[active]?.scrollIntoView?.({ block: "nearest" });
  });
</script>

<svelte:window
  onpointerdown={(e) => {
    if (open && root && !root.contains(e.target as Node)) open = false;
  }}
/>

<div class="dropdown" class:captioned={caption} bind:this={root}>
  <button
    type="button"
    class="trigger"
    role="combobox"
    aria-label={label}
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-controls="{id}-list"
    aria-activedescendant={open ? `${id}-${active}` : undefined}
    onclick={() => (open ? (open = false) : show())}
    {onkeydown}
  >
    {#if caption}<span class="caption">{caption}</span>{/if}
    <span class="value">{value}</span>
  </button>
  <span class="chevron" class:open><Icon name="chevron-down" size={16} /></span>
  {#if open}
    <ul class="menu" id="{id}-list" role="listbox" aria-label={label} bind:this={menu}>
      {#each options as option, i (option)}
        <!-- Keyboard use goes through the combobox above (aria-activedescendant); options only take the pointer. -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <li
          id="{id}-{i}"
          role="option"
          aria-selected={option === value}
          class:active={i === active}
          class:selected={option === value}
          onpointerenter={() => (active = i)}
          onclick={() => choose(i)}
        >
          {option}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .dropdown {
    position: relative;
    display: inline-flex;
    flex-direction: column;
    min-width: 216px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--game-bg-mid, var(--color-control)) 70%, transparent);
  }
  .dropdown.captioned { min-width: 132px; border-radius: var(--radius-md); }
  /* Keyboard focus: an accent border, doubled to 2px with an inset shadow (no outer ring). */
  .dropdown:has(.trigger:focus-visible) {
    border-color: var(--color-accent);
    box-shadow: inset 0 0 0 1px var(--color-accent);
  }
  .trigger {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 8px 36px 8px 14px;
    border: 0;
    border-radius: inherit;
    background: transparent;
    font-size: 13px;
    text-align: left;
  }
  .trigger:focus-visible { outline: none; }
  .captioned .trigger { padding-top: 8px; padding-bottom: 8px; font-weight: 600; }
  .caption { margin-bottom: 2px; font-size: 11px; font-weight: 400; color: var(--color-text-muted); }
  .value { overflow: hidden; max-width: 100%; text-overflow: ellipsis; white-space: nowrap; }
  .chevron {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    pointer-events: none;
    color: var(--color-text-muted);
    transition: transform 0.15s;
  }
  .chevron.open { transform: translateY(-50%) rotate(180deg); }
  @media (prefers-reduced-motion: reduce) {
    .chevron { transition: none; }
  }
  .menu {
    position: absolute;
    z-index: 20;
    top: calc(100% + 6px);
    left: 0;
    min-width: 100%;
    max-height: 260px;
    margin: 0;
    padding: var(--space-1);
    overflow-y: auto;
    list-style: none;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-md);
    /* Opaque, so the page behind never shows through the list. */
    background: var(--game-bg-mid, var(--color-surface));
    box-shadow: 0 12px 32px rgb(0 0 0 / 0.45);
  }
  li {
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    white-space: nowrap;
    cursor: pointer;
  }
  li.active { background: color-mix(in srgb, var(--color-accent) 16%, transparent); }
  li.selected { color: var(--color-accent); font-weight: 600; }
</style>
