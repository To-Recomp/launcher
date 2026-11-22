<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import Row from "./Row.svelte";
  import { gameLicense, type RepositoryLicense } from "../catalog/releases";
  import { openLink, repositoryUrl, revealFile } from "../diagnostics/api";
  import type { Location } from "../settings/types";

  let {
    name,
    repository,
    platform,
    tag,
    location,
    onreportbug,
  }: {
    name: string;
    repository: string;
    platform: string | null;
    /** The installed release; `null` when the game isn't installed. */
    tag: string | null;
    location: Location | null;
    onreportbug: () => Promise<void> | void;
  } = $props();

  /** `undefined` while it loads, `null` when GitHub knows none (or can't be reached). */
  let license = $state<RepositoryLicense | null | undefined>(undefined);
  $effect(() => {
    license = undefined;
    let stale = false;
    gameLicense(repository)
      .then((found) => !stale && (license = found))
      .catch(() => !stale && (license = null));
    return () => (stale = true);
  });
</script>

<div class="group">
  <Row label="Game"><span class="muted">{name}</span></Row>
  {#if platform}<Row label="Console"><span class="muted">{platform}</span></Row>{/if}
  <Row label="Installed release"><span class="muted">{tag ?? "Not installed"}</span></Row>
  <ActionRow label="Repository" value={repository} action="Open" onrun={() => openLink(repositoryUrl(repository))} />
  {#if license}
    {@const link = license}
    <ActionRow label="License" value={link.name} action="Open" onrun={() => openLink(link.url)} />
  {:else}
    <Row label="License"><span class="muted">{license === undefined ? "Loading…" : "Not published"}</span></Row>
  {/if}
  {#if location}
    {@const folders = location}
    <ActionRow label="Install folder" value={folders.installDir} action="Open" onrun={() => revealFile(folders.installDir)} />
    {#if folders.dataDir}
      {@const data = folders.dataDir}
      <ActionRow label="Data folder" value={data} action="Open" onrun={() => revealFile(data)} />
    {/if}
  {/if}
  <ActionRow
    label="Something went wrong?"
    help="Creates a file you can review and paste into an issue of this game's repository."
    action="Report a bug"
    onrun={onreportbug}
  />
</div>

<style>
  .group {
    margin-top: var(--space-5);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }
  .muted { color: var(--color-text-muted); }
</style>
