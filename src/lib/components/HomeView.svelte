<script lang="ts">
  import Card from "./Card.svelte";
  import Dropdown from "./Dropdown.svelte";
  import Icon, { type IconName } from "./Icon.svelte";
  import { hasRepository, type Game, type Platform } from "../catalog/games";
  import { logoFor } from "../catalog/logos";
  import { loadLogoMetrics, type LogoMetrics } from "../catalog/logoMetrics";
  import { cachedReleases, gameReleases, releaseDate, releaseEntries, type ReleaseNote } from "../catalog/releases";
  import { openLink, repositoryUrl } from "../diagnostics/api";
  import { gameJob, installJob, instanceId, prepareJob } from "../library/types";
  import type { LibraryStore } from "../library/store.svelte";
  import { t } from "../i18n/index.svelte";

  let {
    game,
    platform,
    library,
    platformSizing,
    onplatform,
    onsettings,
  }: {
    game: Game;
    platform: Platform | undefined;
    library: LibraryStore;
    /** Every console of every game: the platform selector keeps the width of the widest. */
    platformSizing: { option: string; prefix: string }[];
    onplatform: (label: string) => void;
    /** Opens the game's settings. */
    onsettings: () => void;
  } = $props();

  const title = $derived(game.name.replace(/^Tales of (the )?/, ""));
  const logo = $derived(logoFor(game.name));

  // The letters of every logo stand on the same line: LOGO_BODY px from the top of the logo to its baseline,
  // and what hangs under it (the tail of a y or a p) takes the room below, inside LOGO_BOX.
  const LOGO_BODY = 76;
  const LOGO_BOX = 104;
  /** `undefined` while the logo is read, `null` when it can't be (it then shows at a plain height). */
  let logoMetrics = $state<LogoMetrics | null | undefined>(undefined);
  $effect(() => {
    logoMetrics = undefined;
    if (!logo) return;
    let stale = false;
    loadLogoMetrics(logo).then((found) => {
      if (!stale) logoMetrics = found;
    });
    return () => (stale = true);
  });
  const logoHeight = $derived(logoMetrics ? LOGO_BODY / Math.max(logoMetrics.baseline, 0.3) : LOGO_BOX);

  /** The game's latest releases; `null` while they load. */
  let releases = $state<ReleaseNote[] | null>(null);
  let releasesError = $state(false);
  $effect(() => {
    const repository = game.repository;
    if (!hasRepository(game)) {
      releases = []; // nothing is published for a planned game: no request
      releasesError = false;
      return;
    }
    releases = cachedReleases(repository) ?? null; // coming back to a game shows what was read, with no reload
    releasesError = false;
    let stale = false;
    gameReleases(repository)
      .then((found) => !stale && (releases = found))
      .catch(() => !stale && ((releases = []), (releasesError = true)));
    return () => (stale = true);
  });
  const featured = $derived(releases?.[0]);
  /** "tos-recomp v1.2", or "v1.2 · Name" when the name doesn't say the tag already. */
  const cardTitle = (r: ReleaseNote) =>
    r.name && r.name !== r.tag ? (r.name.includes(r.tag) ? r.name : `${r.tag} · ${r.name}`) : r.tag;

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
  const tasks = $derived(activeJob ? (library.tasks[activeJob] ?? []) : []);
  const taskMarks = { running: "", ok: "✓", skipped: "", failed: "✕", cancelled: "■" } as const;
  const taskState = (state: keyof typeof taskMarks) =>
    ({
      running: t("game.home.task.running"),
      ok: t("game.home.task.ok"),
      skipped: t("game.home.task.skipped"),
      failed: t("game.home.task.failed"),
      cancelled: t("game.home.task.cancelled"),
    })[state];
  /** "0.9 s", "36 s", "2 min 5 s". */
  const duration = (seconds: number) =>
    seconds < 10 ? `${seconds.toFixed(1)} s` : seconds < 60 ? `${Math.round(seconds)} s` : `${Math.floor(seconds / 60)} min ${Math.round(seconds % 60)} s`;
  const busy = $derived(activeJob !== undefined);

  // The live log: the preparation's log re-read every second while it runs, kept scrolled to the end
  // unless the player scrolled up to read.
  let follow = $state(false);
  let logElement = $state<HTMLPreElement>();
  let pinned = true;
  function closeLog() {
    follow = false; // a followed log stops being re-read
    library.log = null;
  }
  function toggleLiveLog() {
    if (follow) closeLog();
    else follow = true;
  }
  $effect(() => {
    // The job ended while the log was followed: read the last lines once more and keep them.
    if (!busy && follow) {
      follow = false;
      if (platform) void library.showLog(game, platform, "prepare");
    }
  });
  $effect(() => {
    if (follow && platform && activeJob === jobs.prepare) return library.followLog(game, platform, "prepare");
  });
  $effect.pre(() => {
    library.log;
    if (logElement) pinned = logElement.scrollHeight - logElement.scrollTop - logElement.clientHeight < 24;
  });
  $effect(() => {
    library.log;
    if (follow && pinned && logElement) logElement.scrollTop = logElement.scrollHeight;
  });

  const required = $derived(installation?.inputs.filter((i) => !i.optional) ?? []);

  function chooseLabel(): string {
    if (required.length === 0 || (platform && library.filesChosen(game.id, platform))) return t("game.home.prepare");
    if (required.length > 1) return t("game.home.chooseFiles");
    return required[0].kinds.length === 1 && required[0].kinds[0] === "folder"
      ? t("game.home.chooseFolder")
      : t("game.home.chooseFile");
  }

  const primary = $derived.by((): { label: string; icon: IconName; run: () => void } => {
    if (!installation) return { label: t("game.home.install"), icon: "download", run: () => library.install(game) };
    if (!platform) return { label: t("game.home.install"), icon: "download", run: () => {} };
    if (instanceState === "ready") return { label: t("game.home.play"), icon: "play", run: () => library.play(game, platform) };
    if (instanceState === "needsRebuild") {
      return {
        label: installation.keepsCopy ? t("game.home.rebuild") : t("game.home.prepareAgain"),
        icon: "folder",
        run: () => library.prepare(game, platform),
      };
    }
    return { label: chooseLabel(), icon: "folder", run: () => library.prepare(game, platform) };
  });

  /** Why the selected console can't be installed or prepared, if it can't. */
  const unavailable = $derived(
    platform?.state === "planned"
      ? t("game.home.planned")
      : platform?.state === "in-progress"
        ? t("game.home.inProgress")
        : platform?.state === "unsupported"
          ? t("game.home.unsupported")
          : null,
  );

  /** The newest release, when the installed one isn't it. */
  const newer = $derived(library.updateAvailable(game.id));

  const hint = $derived(
    unavailable ??
      (newer && installation
        ? t("game.home.hintUpdate", { tag: newer, current: installation.tag })
        : !installation
        ? t("game.home.hintInstall")
        : instanceState === "ready"
          ? t("game.home.hintReady")
          : instanceState === "needsRebuild"
            ? t("game.home.hintRebuild")
            : t("game.home.hintChoose")),
  );

  /** Every label the main button may carry: it is as wide as the widest, whatever its state or language. */
  const labels = $derived([
    t("game.home.install"),
    t("game.home.play"),
    t("game.home.rebuild"),
    t("game.home.prepareAgain"),
    t("game.home.prepare"),
    t("game.home.chooseFiles"),
    t("game.home.chooseFolder"),
    t("game.home.chooseFile"),
  ]);
  const unusable = $derived(busy || !platform || unavailable !== null);

  /** The build can be deleted once it exists. */
  const hasBuild = $derived(
    installation !== undefined && platform !== undefined && (instanceState === "ready" || instanceState === "needsRebuild"),
  );
  /** The main button's menu: the game's settings once it is installed (also before it is prepared, to choose its files), and deleting the build. */
  const hasMenu = $derived(installation !== undefined && platform !== undefined);
  let menuOpen = $state(false);
  let split = $state<HTMLDivElement>();
  $effect(() => {
    if (!hasMenu) menuOpen = false;
  });

  const releaseUrl = (note: ReleaseNote) => `${repositoryUrl(game.repository)}/releases/tag/${encodeURIComponent(note.tag)}`;

  /** The tint of the action status: idle with no message, else the message's tone (a running action is info). */
  const tone = $derived<"idle" | "info" | "success" | "error">(active ? "info" : (library.status?.tone ?? "idle"));

  /** What the selector says next to a console the game can't be played on, by its state; those consoles are listed but not selectable. */
  const stateNames = $derived({
    planned: t("game.home.state.planned"),
    "in-progress": t("game.home.state.inProgress"),
    unsupported: t("game.home.state.unsupported"),
  });
  /** A playable console says how far it is, unless it is stable; it stays selectable. */
  const maturityNames = $derived({
    alpha: t("game.home.maturity.alpha"),
    beta: t("game.home.maturity.beta"),
  });
  const unplayable = $derived(game.platforms.filter((p) => p.state !== "playable").map((p) => p.platform));
  const platformNotes = $derived(
    Object.fromEntries(
      game.platforms.flatMap((p) => {
        const note = p.state === "playable" ? (p.maturity && p.maturity !== "stable" ? maturityNames[p.maturity] : undefined) : stateNames[p.state];
        return note ? [[p.platform, note]] : [];
      }),
    ),
  );
</script>

<svelte:window
  onpointerdown={(e) => {
    if (menuOpen && split && !split.contains(e.target as Node)) menuOpen = false;
  }}
  onkeydown={(e) => {
    if (menuOpen && e.key === "Escape") menuOpen = false;
  }}
/>

<section class="home">
  <h1 class:has-logo={logo}>
    {#if logo}
      <img
        src={logo}
        alt={title}
        style:height="{logoHeight}px"
        style:width={logoMetrics ? `${logoHeight * logoMetrics.aspect}px` : undefined}
        style:opacity={logoMetrics === undefined ? 0 : 1}
      />
    {:else}{title}{/if}
  </h1>
  <p class="hint">{hint}</p>

  <div class="actions">
    <Dropdown
      caption={t("game.home.platform")}
      label={t("game.home.platform")}
      options={game.platforms.map((p) => p.platform)}
      prefixes={Object.fromEntries(game.platforms.map((p) => [p.platform, String(p.year)]))}
      disabled={unplayable}
      notes={platformNotes}
      sizing={platformSizing}
      value={platform?.platform ?? ""}
      onchange={onplatform}
    />
    <div class="split" class:has-menu={hasMenu} bind:this={split}>
      <button class="main" disabled={unusable} onclick={primary.run}>
        <Icon name={primary.icon} size={20} />
        <span class="label">
          {#each labels as text}<span class="sizer" aria-hidden="true">{text}</span>{/each}
          <span>{primary.label}</span>
        </span>
      </button>
      {#if hasMenu && platform}
        <span class="separator" aria-hidden="true"></span>
        <button
          class="chevron"
          aria-label={t("game.home.moreActions")}
          aria-haspopup="menu"
          aria-expanded={menuOpen}
          onclick={() => (menuOpen = !menuOpen)}
        >
          <Icon name="chevron-down" size={18} />
        </button>
        {#if menuOpen}
          <ul class="menu" role="menu">
            <li role="none">
              <button
                role="menuitem"
                onclick={() => {
                  menuOpen = false;
                  onsettings();
                }}
              >
                <Icon name="gear" size={16} />{t("game.home.settings")}
              </button>
            </li>
            {#if hasBuild}
              <li role="none">
                <button
                  class="danger"
                  role="menuitem"
                  disabled={busy}
                  onclick={() => {
                    menuOpen = false;
                    library.deleteBuild(game, platform);
                  }}
                >
                  <Icon name="trash" size={16} />{t("game.home.deleteBuild")}
                </button>
              </li>
            {/if}
          </ul>
        {/if}
      {/if}
    </div>
    {#if newer}
      <button class="update" disabled={busy} title={t("game.home.updateTo", { tag: newer })} onclick={() => library.install(game)}>
        <Icon name="download" size={18} />{t("game.home.update")}
      </button>
    {/if}
  </div>

  <div class="status {tone}" role="status">
    <span class="status-icon"><Icon name={tone === "error" ? "alert" : tone === "success" ? "check" : "info"} size={20} /></span>
    <div class="status-body">
      <strong>{t("game.home.actionStatus")}</strong>
      {#if active}
        <p>{active.message}{active.fraction !== null ? ` · ${Math.round(active.fraction * 100)}%` : ""}</p>
        <div
          class="bar"
          role="progressbar"
          aria-label={t("game.home.progress")}
          aria-valuemin="0"
          aria-valuemax="100"
          aria-valuenow={active.fraction === null ? undefined : Math.round(active.fraction * 100)}
        >
          <span class:indeterminate={active.fraction === null} style:width="{(active.fraction ?? 0.3) * 100}%"></span>
        </div>
        {#if tasks.length > 0}
          <ol class="tasks" aria-label={t("game.home.tasks")}>
            {#each tasks as task (task.id)}
              <li class={task.state}>
                <span class="task-mark" role="img" aria-label={taskState(task.state)}>{taskMarks[task.state]}</span>
                <span class="task-title"><span class="task-index">{task.index}/{task.count}</span> {task.title}</span>
                <span class="task-detail">
                  {#if task.state === "running"}
                    {task.fraction !== null ? `${Math.round(task.fraction * 100)}%` : ""}
                  {:else if task.state === "skipped"}
                    {taskState("skipped")}
                  {:else if task.seconds !== null}
                    {duration(task.seconds)}
                  {/if}
                </span>
                {#if task.state === "running"}
                  {#if task.text}<span class="task-text">{task.text}</span>{/if}
                  {#if task.fraction !== null}
                    <div class="bar" role="progressbar" aria-label={task.title} aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(task.fraction * 100)}>
                      <span style:width="{task.fraction * 100}%"></span>
                    </div>
                  {/if}
                {/if}
              </li>
            {/each}
          </ol>
        {/if}
      {:else if library.status}
        <p>{library.status.text}</p>
      {:else}
        <p>{t("game.home.noActions")}</p>
      {/if}
    </div>
    <div class="status-actions">
      {#if activeJob && activeJob !== jobs.game}
        <button onclick={() => library.cancel(activeJob)}>{t("game.home.cancel")}</button>
      {/if}
      {#if library.failure?.gameId === game.id && !busy}
        <button onclick={() => library.reportFailure(game, platform)}>{t("game.home.createDiagnostic")}</button>
      {/if}
      {#if platform && activeJob === jobs.prepare}
        <button aria-pressed={follow} onclick={toggleLiveLog}>{follow ? t("game.home.hideLiveLog") : t("game.home.liveLog")}</button>
      {/if}
      {#if installation && platform && !busy}
        <button onclick={() => library.showLog(game, platform, "prepare")}>{t("game.home.prepareLog")}</button>
        {#if instanceState === "ready"}
          <button onclick={() => library.showLog(game, platform, "game")}>{t("game.home.gameLog")}</button>
        {/if}
      {/if}
    </div>
  </div>
  {#if library.log !== null}
    <div class="log-box">
      <pre class="log" aria-label={t("game.home.log")} bind:this={logElement}>{library.log}</pre>
      <button class="log-close" aria-label={t("game.home.closeLog")} title={t("game.home.closeLog")} onclick={closeLog}>
        <Icon name="close" size={14} />
      </button>
    </div>
  {/if}

  <h2>{t("game.home.latestUpdates")}</h2>
  {#if releases === null}
    <p class="releases-empty">{t("game.home.releasesLoading")}</p>
  {:else if !featured}
    <p class="releases-empty">{releasesError ? t("game.home.releasesError") : t("game.home.releasesNone")}</p>
  {:else}
    <div class="cards">
      {#each releases ?? [] as note, i (note.tag)}
        {@const { entries, more } = releaseEntries(note.body, 2)}
        <Card featured={i === 0}>
          <span class="meta" class:accent={i === 0}>
            {(i === 0 ? [note.tag, releaseDate(note.publishedAt)] : [releaseDate(note.publishedAt)]).filter(Boolean).join(" · ")}
          </span>
          <h3>{i === 0 ? (note.name ?? note.tag) : cardTitle(note)}</h3>
          {#if entries.length > 0}
            <ul class="entries">
              {#each entries as entry}
                <li><span class="clamp">{entry}</span></li>
              {/each}
              {#if more > 0}
                <li class="entries-more">
                  <button onclick={() => openLink(releaseUrl(note))}>{t("game.home.releaseMore", { count: more })}</button>
                </li>
              {/if}
            </ul>
          {/if}
          {#if i === 0}
            <button class="more" aria-label={t("game.home.openRelease")} title={t("game.home.openRelease")} onclick={() => openLink(releaseUrl(note))}>
              <Icon name="chevron-right" size={16} />
            </button>
          {/if}
        </Card>
      {/each}
    </div>
  {/if}
</section>

<style>
  .home {
    display: flex;
    flex-direction: column;
    padding: var(--space-6) var(--space-6) 0;
    min-height: 100%;
  }
  h1 { margin: var(--space-6) 0 0; font-size: 48px; font-weight: 700; }
  h1.has-logo { min-height: 104px; line-height: 0; }
  /* Every logo has the same letter height on the same baseline; see LOGO_BODY. */
  h1 img { display: block; max-width: 100%; object-fit: contain; object-position: left top; }
  .hint { margin: var(--space-3) 0 var(--space-5); font-size: 16px; color: var(--color-text-muted); }
  .actions { display: flex; gap: var(--space-3); align-items: stretch; }
  .split {
    position: relative;
    display: inline-flex;
    align-items: stretch;
    border-radius: var(--radius-md);
    background: var(--color-primary);
  }
  .main, .chevron {
    border: 0;
    background: transparent;
    color: inherit;
    font-weight: 600;
  }
  .main {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-4);
    border-radius: var(--radius-md);
  }
  .has-menu .main { border-radius: var(--radius-md) 0 0 var(--radius-md); }
  .label { display: grid; text-align: left; }
  .label > * { grid-area: 1 / 1; }
  .sizer { height: 0; overflow: hidden; visibility: hidden; white-space: nowrap; }
  .separator { width: 1px; margin: var(--space-3) 0; background: rgb(255 255 255 / 0.3); }
  .chevron { display: grid; place-items: center; width: 40px; border-radius: 0 var(--radius-md) var(--radius-md) 0; }
  .main:hover:not(:disabled), .chevron:hover:not(:disabled) { background: var(--color-primary-hover); }
  .main:disabled, .chevron:disabled { opacity: 0.5; cursor: default; }
  .menu {
    position: absolute;
    z-index: 20;
    top: calc(100% + 6px);
    right: 0;
    min-width: 100%;
    margin: 0;
    padding: var(--space-1);
    list-style: none;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-md);
    background: var(--game-bg-mid, var(--color-surface));
    box-shadow: 0 12px 32px rgb(0 0 0 / 0.45);
  }
  .menu button {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: 8px 12px;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    font-size: 13px;
    white-space: nowrap;
    text-align: left;
  }
  .menu button:hover:not(:disabled) { background: color-mix(in srgb, var(--color-accent) 16%, transparent); }
  .menu button.danger:hover:not(:disabled) { color: rgb(230 90 86); }
  .menu button:disabled { opacity: 0.5; cursor: default; }
  .update {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-4);
    border: 1px solid color-mix(in srgb, var(--color-accent) 55%, transparent);
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-accent) 14%, transparent);
    color: var(--color-accent);
    font-weight: 600;
  }
  .update:hover:not(:disabled) { background: color-mix(in srgb, var(--color-accent) 24%, transparent); }
  .update:disabled { opacity: 0.5; cursor: default; }
  /* One tint per state: icon, border and background always share it. Idle is neutral; a running action
     or an info message takes the app's palette (the game's accent); success is green and an error red. */
  .status {
    --tint: var(--color-text-muted);
    display: flex;
    gap: var(--space-4);
    align-items: center;
    margin-top: var(--space-5);
    padding: var(--space-4) var(--space-5);
    border: 1px solid color-mix(in srgb, var(--tint) 45%, transparent);
    border-radius: var(--radius-md);
    background: transparent;
  }
  .status.info { --tint: var(--color-accent); }
  .status.success { --tint: var(--color-success); }
  .status.error { --tint: var(--color-error); }
  .status:not(.idle) { background: color-mix(in srgb, var(--tint) 12%, transparent); }
  .status-icon { color: var(--tint); }
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
  .tasks {
    display: grid;
    gap: var(--space-1);
    margin: var(--space-4) 0 0;
    padding: var(--space-3) 0 0;
    border-top: 1px solid var(--color-border);
    list-style: none;
  }
  .tasks li {
    display: grid;
    grid-template-columns: 20px 1fr auto;
    align-items: center;
    column-gap: var(--space-3);
    padding: 9px var(--space-3);
    border-radius: var(--radius-md);
    font-size: 13px;
  }
  .tasks li.running { background: color-mix(in srgb, var(--color-accent) 10%, transparent); }
  .task-mark {
    display: grid;
    place-items: center;
    box-sizing: border-box;
    width: 18px;
    height: 18px;
    border: 2px solid var(--color-border-strong);
    border-radius: 50%;
    font-size: 10px;
    line-height: 1;
    color: var(--color-text-muted);
  }
  .tasks li.ok .task-mark { border-color: var(--color-accent); background: var(--color-accent); color: var(--color-bg-sunken); }
  .tasks li.skipped .task-mark { border-style: dashed; }
  .tasks li.failed .task-mark { border-color: rgb(200 48 44); background: rgb(200 48 44); color: #fff; }
  .tasks li.running .task-mark { border-color: color-mix(in srgb, var(--color-accent) 30%, transparent); border-top-color: var(--color-accent); animation: spin 0.9s linear infinite; }
  .task-title { min-width: 0; color: var(--color-text-muted); }
  .tasks li.running .task-title { color: var(--color-text); font-weight: 600; }
  .task-index { margin-right: var(--space-2); font-size: 11px; font-variant-numeric: tabular-nums; opacity: 0.7; }
  .task-detail { min-width: 56px; text-align: right; font-size: 12px; color: var(--color-text-muted); font-variant-numeric: tabular-nums; }
  .tasks li.running .task-detail { color: var(--color-text); }
  .task-text { grid-column: 2 / 4; margin-top: 2px; font-size: 12px; color: var(--color-text-muted); }
  .tasks .bar { grid-column: 2 / 4; height: 6px; margin-top: var(--space-2); }
  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes slide { from { margin-left: 0; } to { margin-left: 70%; } }
  .log-box { position: relative; margin-top: var(--space-3); }
  .log-close {
    position: absolute;
    top: var(--space-2);
    right: var(--space-2); /* as far from the right edge as from the top; the 6px scrollbar stays clear of it */
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-sm);
    background: var(--color-surface-alt);
    color: var(--color-text-muted);
  }
  .log-close:hover { color: var(--color-text); border-color: var(--color-accent); }
  .log {
    max-height: 260px;
    margin: 0;
    padding: var(--space-4);
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
  .releases-empty { margin: 0; color: var(--color-text-muted); }
  h3 { margin: var(--space-1) 0 0; font-size: 18px; }
  .entries { display: grid; gap: var(--space-1); margin: var(--space-1) 0 0; padding: 0 0 0 var(--space-4); line-height: 1.45; color: var(--color-text-muted); }
  .entries li { overflow-wrap: anywhere; }
  /* A long entry is cut after one line, a long release after two entries: the rest is on GitHub. */
  .clamp { display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 1; line-clamp: 1; overflow: hidden; }
  .entries li.entries-more { list-style: none; margin-left: calc(-1 * var(--space-4)); font-size: 12px; }
  .entries-more button { padding: 0; border: 0; background: none; color: var(--color-accent); font-size: 12px; opacity: 0.85; }
  .entries-more button:hover { opacity: 1; text-decoration: underline; }
  .meta { font-size: 12px; color: var(--color-text-muted); }
  .meta.accent { color: var(--color-accent); font-weight: 600; }
  .more {
    position: absolute;
    right: var(--space-4);
    top: var(--space-4);
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: var(--color-accent);
    color: var(--color-bg);
  }
  .more:hover { filter: brightness(1.12); }
</style>
