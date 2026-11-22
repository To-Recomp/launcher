<script lang="ts">
  import Card from "./Card.svelte";
  import Dropdown from "./Dropdown.svelte";
  import Icon, { type IconName } from "./Icon.svelte";
  import type { Game, Platform } from "../catalog/games";
  import { logoFor } from "../catalog/logos";
  import { updates } from "../data";
  import { gameJob, installJob, instanceId, prepareJob } from "../library/types";
  import type { LibraryStore } from "../library/store.svelte";

  let {
    game,
    platform,
    library,
    onplatform,
  }: { game: Game; platform: Platform | undefined; library: LibraryStore; onplatform: (label: string) => void } = $props();

  const title = $derived(game.name.replace(/^Tales of (the )?/, ""));
  const logo = $derived(logoFor(game.name));

  const [featured, ...others] = updates;

  const installation = $derived(library.installationOf(game.id));
  const instanceState = $derived(platform ? library.instanceState(game.id, platform) : "notPrepared");
  const instance = $derived(platform ? instanceId(platform.platform) : "");
  const jobs = $derived({
    install: installJob(game.id),
    prepare: prepareJob(game.id, instance),
    game: gameJob(game.id, instance),
  });
  const activeJob = $derived(
    [jobs.install, jobs.prepare, jobs.game].find((job) => library.isRunning(job)),
  );
  const active = $derived(activeJob ? library.progress[activeJob] : undefined);
  const busy = $derived(activeJob !== undefined);

  const required = $derived(installation?.inputs.filter((i) => !i.optional) ?? []);

  function chooseLabel(): string {
    if (required.length === 0) return "Prepare";
    if (required.length > 1) return "Choose game files";
    return required[0].kinds.length === 1 && required[0].kinds[0] === "folder" ? "Choose game folder" : "Choose game file";
  }

  const primary = $derived.by((): { label: string; icon: IconName; run: () => void } => {
    if (!installation) return { label: "Install", icon: "download", run: () => library.install(game) };
    if (!platform) return { label: "Install", icon: "download", run: () => {} };
    if (instanceState === "ready") return { label: "Play", icon: "play", run: () => library.play(game, platform) };
    if (instanceState === "needsRebuild") {
      return { label: installation.keepsCopy ? "Rebuild" : "Prepare again", icon: "folder", run: () => library.prepare(game, platform) };
    }
    return { label: chooseLabel(), icon: "folder", run: () => library.prepare(game, platform) };
  });

  /** Why the selected console can't be installed or prepared, if it can't. */
  const unavailable = $derived(
    platform?.state === "planned"
      ? "This console is planned: nothing to install yet."
      : platform?.state === "in-progress"
        ? "This console is in progress: there is nothing to install yet."
        : platform?.state === "unsupported"
          ? "This console isn't supported."
          : null,
  );

  const hint = $derived(
    unavailable ??
      (!installation
        ? "Install the game to get started."
        : instanceState === "ready"
          ? "Ready to play."
          : instanceState === "needsRebuild"
            ? "The game needs a new build after the update."
            : "Choose your game files to get started."),
  );
</script>

<section class="home">
  <h1 class:has-logo={logo}>
    {#if logo}<img src={logo} alt={title} />{:else}{title}{/if}
  </h1>
  <p class="hint">{hint}</p>

  <div class="actions">
    <Dropdown
      caption="Platform"
      label="Platform"
      options={game.platforms.map((p) => p.platform)}
      value={platform?.platform ?? ""}
      onchange={onplatform}
    />
    <button class="primary" disabled={busy || !platform || unavailable !== null} onclick={primary.run}>
      <Icon name={primary.icon} size={20} /> {primary.label}
    </button>
    {#if installation && instanceState === "ready" && required.length > 0 && platform}
      <button class="secondary" disabled={busy} onclick={() => library.prepare(game, platform)}>Change game files</button>
    {/if}
  </div>

  <div class="status" role="status" class:error={!active && library.status?.tone === "error"}>
    <span class="status-icon"><Icon name="info" size={20} /></span>
    <div class="status-body">
      <strong>Action status</strong>
      {#if active}
        <p>{active.message}{active.fraction !== null ? ` · ${Math.round(active.fraction * 100)}%` : ""}</p>
        <div
          class="bar"
          role="progressbar"
          aria-label="Progress"
          aria-valuemin="0"
          aria-valuemax="100"
          aria-valuenow={active.fraction === null ? undefined : Math.round(active.fraction * 100)}
        >
          <span class:indeterminate={active.fraction === null} style:width="{(active.fraction ?? 0.3) * 100}%"></span>
        </div>
      {:else if library.status}
        <p>{library.status.text}</p>
      {:else}
        <p>No recent actions. Results will appear here.</p>
      {/if}
    </div>
    <div class="status-actions">
      {#if activeJob && activeJob !== jobs.game}
        <button onclick={() => library.cancel(activeJob)}>Cancel</button>
      {/if}
      {#if library.failure?.gameId === game.id && !busy}
        <button onclick={() => library.reportFailure(game, platform)}>Create diagnostic file</button>
      {/if}
      {#if installation && platform && !busy}
        <button onclick={() => library.showLog(game, platform, "prepare")}>Preparation log</button>
        {#if instanceState === "ready"}
          <button onclick={() => library.showLog(game, platform, "game")}>Game log</button>
        {/if}
      {/if}
    </div>
  </div>
  {#if library.log !== null}
    <pre class="log" aria-label="Log">{library.log}</pre>
  {/if}

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
  .home {
    display: flex;
    flex-direction: column;
    padding: var(--space-6) var(--space-6) 0;
    min-height: 100%;
  }
  h1 { margin: var(--space-6) 0 0; font-size: 48px; font-weight: 700; }
  h1.has-logo { line-height: 0; }
  h1 img { display: block; max-width: 100%; max-height: 120px; object-fit: contain; object-position: left; }
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
  .primary:hover:not(:disabled) { background: var(--color-primary-hover); }
  .primary:disabled { opacity: 0.5; cursor: default; }
  .secondary {
    padding: 0 var(--space-4);
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-alt);
  }
  .secondary:disabled { opacity: 0.5; cursor: default; }
  .status {
    display: flex;
    gap: var(--space-4);
    align-items: center;
    margin-top: var(--space-5);
    padding: var(--space-4) var(--space-5);
    border: 1px solid color-mix(in srgb, var(--color-info) 50%, transparent);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--game-bg-mid, #1f2b42) 60%, transparent);
  }
  .status.error { border-color: rgb(200 48 44 / 0.7); background: rgb(200 48 44 / 0.12); }
  .status-icon { color: var(--color-info); }
  .status-body { flex: 1; min-width: 0; }
  .status p { margin: 2px 0 0; font-size: 12px; color: var(--color-text-muted); user-select: text; }
  .status-actions { display: flex; gap: var(--space-2); }
  .status-actions button {
    padding: 5px 12px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-sm);
    background: none;
    font-size: 12px;
  }
  .bar { height: 6px; margin-top: var(--space-2); border-radius: 3px; background: var(--color-bg-sunken); overflow: hidden; }
  .bar span { display: block; height: 100%; background: var(--color-accent); transition: width 0.2s; }
  .bar span.indeterminate { animation: slide 1.2s ease-in-out infinite alternate; }
  @keyframes slide { from { margin-left: 0; } to { margin-left: 70%; } }
  .log {
    max-height: 200px;
    margin: var(--space-3) 0 0;
    padding: var(--space-3) var(--space-4);
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-sunken);
    font-size: 11px;
    user-select: text;
    white-space: pre-wrap;
  }
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
