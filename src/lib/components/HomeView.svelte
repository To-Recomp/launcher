<script lang="ts">
  import Card from "./Card.svelte";
  import Dropdown from "./Dropdown.svelte";
  import Icon from "./Icon.svelte";
  import { updates, type Game } from "../data";

  let { game }: { game: Game } = $props();

  let platform = $state("");
  $effect(() => {
    platform = game.platforms[0];
  });

  const title = $derived(game.name.replace(/^Tales of (the )?/, ""));
  const [featured, ...others] = updates;
</script>

<section class="home">
  <h1>{title}</h1>
  <p class="hint">Choose your game folder to get started.</p>

  <div class="actions">
    <Dropdown caption="Platform" label="Platform" options={game.platforms} value={platform} onchange={(v) => (platform = v)} />
    <button class="primary"><Icon name="folder" size={20} /> Choose game folder</button>
  </div>

  <div class="status" role="status">
    <span class="status-icon"><Icon name="info" size={20} /></span>
    <div>
      <strong>Action status</strong>
      <p>No recent actions. Results will appear here.</p>
    </div>
  </div>

  <h2>Latest updates</h2>
  <div class="cards">
    <Card featured>
      <span class="meta accent">{featured.version} · {featured.date}</span>
      <h3>{featured.title}</h3>
      <p>{featured.body}</p>
      <span class="more"><Icon name="chevron-right" size={18} /></span>
    </Card>
    {#each others as note (note.version)}
      <Card>
        <span class="meta">{note.date}</span>
        <h3>{note.version} · {note.title}</h3>
        <p>{note.body}</p>
      </Card>
    {/each}
  </div>
</section>

<style>
  .home { display: flex; flex-direction: column; padding: var(--space-6) var(--space-6) 0; min-height: 100%; }
  h1 { margin: var(--space-6) 0 0; font-size: 48px; font-weight: 700; }
  .hint { margin: var(--space-3) 0 var(--space-5); font-size: 16px; color: var(--color-text-muted); }
  .actions { display: flex; gap: var(--space-3); align-items: stretch; }
  .primary {
    display: inline-flex;
    align-items: center;
    gap: var(--space-3);
    width: 298px;
    padding: 0 var(--space-5);
    border: 0;
    border-radius: var(--radius-md);
    background: var(--color-primary);
    font-weight: 600;
  }
  .primary:hover { background: var(--color-primary-hover); }
  .status {
    display: flex;
    gap: var(--space-4);
    align-items: center;
    margin-top: var(--space-5);
    padding: var(--space-4) var(--space-5);
    border: 1px solid rgb(89 132 173 / 0.5);
    border-radius: var(--radius-md);
    background: rgb(31 43 66 / 0.6);
  }
  .status-icon { color: var(--color-info); }
  .status p { margin: 2px 0 0; font-size: 12px; color: var(--color-text-muted); }
  h2 { margin: auto 0 var(--space-4); padding-top: var(--space-6); font-size: 22px; }
  .cards { display: grid; grid-template-columns: 1.4fr 1fr 1fr; gap: var(--space-4); }
  h3 { margin: var(--space-2) 0 0; font-size: 20px; }
  .cards p { margin: 0; line-height: 1.5; color: var(--color-text-muted); }
  .meta { font-size: 12px; color: var(--color-text-muted); }
  .meta.accent { color: var(--color-accent); font-weight: 600; }
  .more {
    position: absolute;
    right: var(--space-4);
    bottom: var(--space-4);
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: 50%;
    background: var(--color-accent);
    color: var(--color-bg);
  }
</style>
