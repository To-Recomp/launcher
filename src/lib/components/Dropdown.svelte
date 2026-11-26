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
    prefixes = {},
    sizing = [],
    disabled = [],
    notes = {},
    onchange,
  }: {
    options: string[];
    value: string;
    label: string;
    /** Small label shown above the value (Home platform selector). */
    caption?: string;
    /** A small label shown before an option (its release year), by option. */
    prefixes?: Record<string, string>;
    /**
     * What the control must have room for, besides its own options: it is as wide as the widest of
     * these and of the options, so it never grows or shrinks when another one is chosen.
     */
    sizing?: { option: string; prefix?: string }[];
    /** Options that are listed but can't be chosen. */
    disabled?: string[];
    /** A small text shown at the end of an option (like "Planned"), by option. */
    notes?: Record<string, string>;
    onchange: (value: string) => void;
  } = $props();

  const sizes = $derived([...options.map((option) => ({ option, prefix: prefixes[option] })), ...sizing]);

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

  const isDisabled = (index: number) => disabled.includes(options[index]);

  /** The nearest option that can be chosen from `from` in direction `step`; `from` itself when there is none. */
  function nearestEnabled(from: number, step: 1 | -1): number {
    for (let i = from + step; i >= 0 && i < options.length; i += step) if (!isDisabled(i)) return i;
    return from;
  }

  function choose(index: number) {
    if (options[index] === undefined || isDisabled(index)) return;
    open = false;
    if (options[index] !== value) onchange(options[index]);
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
      case "ArrowDown": active = nearestEnabled(active, 1); break;
      case "ArrowUp": active = nearestEnabled(active, -1); break;
      case "Home": active = isDisabled(0) ? nearestEnabled(0, 1) : 0; break;
      case "End": active = isDisabled(last) ? nearestEnabled(last, -1) : last; break;
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
    <span class="shown">
      <!-- Invisible copies of every text the control may show: together they set its width. -->
      {#each sizes as size}
        <span class="sizer" aria-hidden="true">{@render named(size.option, size.prefix)}</span>
      {/each}
      <span class="value">{@render named(value, prefixes[value])}</span>
    </span>
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
          class:active={i === active && !isDisabled(i)}
          class:selected={option === value}
          class:disabled={isDisabled(i)}
          aria-disabled={isDisabled(i) ? "true" : undefined}
          onpointerenter={() => !isDisabled(i) && (active = i)}
          onclick={() => choose(i)}
        >
          <span>{@render named(option, prefixes[option])}</span>
          {#if notes[option]}<span class="note">{notes[option]}</span>{/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#snippet named(option: string, prefix: string | undefined)}
  {#if prefix}<span class="prefix">{prefix} ·</span>{" "}{/if}{option}
{/snippet}

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
    /* A grid, so the icon fills the box (no line strut under it): the box is centred and rotates about
       the icon's own centre, which keeps the chevron in the same place open or closed. */
    display: grid;
    place-items: center;
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
    display: flex;
    align-items: baseline; /* the year, the name and the state share one baseline */
    justify-content: space-between;
    gap: var(--space-4);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    white-space: nowrap;
    cursor: pointer;
  }
  li.disabled { cursor: default; opacity: 0.45; }
  .note { font-size: 11px; color: var(--color-text-muted); }
  li.active { background: color-mix(in srgb, var(--color-accent) 16%, transparent); }
  li.selected { color: var(--color-accent); font-weight: 600; }
  .shown { display: grid; width: 100%; }
  .shown > * { grid-area: 1 / 1; }
  .sizer { height: 0; overflow: hidden; visibility: hidden; white-space: nowrap; }
  .prefix { font-size: 11px; font-weight: 400; color: var(--color-text-muted); }
  li.selected .prefix { color: inherit; opacity: 0.8; }
</style>
