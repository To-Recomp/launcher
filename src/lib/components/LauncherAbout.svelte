<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import Row from "./Row.svelte";
  import { openLink } from "../diagnostics/api";
  import { CREDITS, LAUNCHER_REPOSITORY, LAUNCHER_URL, LAUNCHER_VERSION, NON_AFFILIATION } from "../data";

  let { onreportbug }: { onreportbug: () => Promise<void> | void } = $props();
</script>

<h2>About</h2>
<div class="group">
  <Row label="To Recomp Launcher"><span class="muted">v{LAUNCHER_VERSION}</span></Row>
  <Row label="License"><span class="muted">GNU GPL v3</span></Row>
  <ActionRow label="Repository" value={LAUNCHER_REPOSITORY} action="Open" onrun={() => openLink(LAUNCHER_URL)} />
  <ActionRow
    label="Something went wrong?"
    help="Creates a file you can review and paste into a GitHub issue."
    action="Report a bug"
    onrun={onreportbug}
  />
  <p class="muted notice">{NON_AFFILIATION}</p>
</div>

<h2>Credits</h2>
<div class="group">
  {#each CREDITS as credit (credit.name)}
    <ActionRow label={credit.name} value={credit.license} action="Open" onrun={() => openLink(credit.url)} />
  {/each}
</div>

<style>
  h2 { margin: var(--space-5) 0 var(--space-2) var(--space-4); font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    overflow: hidden;
  }
  .muted { color: var(--color-text-muted); }
  .notice { margin: 0; padding: var(--space-3) var(--space-4); border-top: 1px solid var(--color-border); font-size: 12px; }
</style>
