<script lang="ts">
  import ActionRow from "./ActionRow.svelte";
  import Row from "./Row.svelte";
  import { gameLicense, type RepositoryLicense } from "../catalog/releases";
  import { openLink, repositoryUrl, revealFile } from "../diagnostics/api";
  import type { Location } from "../settings/types";
  import { t } from "../i18n/index.svelte";

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
  <Row label={t("game.about.game")}><span class="muted">{name}</span></Row>
  {#if platform}<Row label={t("game.about.console")}><span class="muted">{platform}</span></Row>{/if}
  <Row label={t("game.about.installedRelease")}><span class="muted">{tag ?? t("game.about.notInstalled")}</span></Row>
  <ActionRow label={t("game.about.repository")} value={repository} action={t("game.about.open")} onrun={() => openLink(repositoryUrl(repository))} />
  {#if license}
    {@const link = license}
    <ActionRow label={t("game.about.license")} value={link.name} action={t("game.about.open")} onrun={() => openLink(link.url)} />
  {:else}
    <Row label={t("game.about.license")}><span class="muted">{license === undefined ? t("game.about.loading") : t("game.about.notPublished")}</span></Row>
  {/if}
  {#if location}
    {@const folders = location}
    <ActionRow label={t("game.about.installFolder")} value={folders.installDir} action={t("game.about.open")} onrun={() => revealFile(folders.installDir)} />
    {#if folders.dataDir}
      {@const data = folders.dataDir}
      <ActionRow label={t("game.about.dataFolder")} value={data} action={t("game.about.open")} onrun={() => revealFile(data)} />
    {/if}
  {/if}
  <ActionRow
    label={t("game.about.wentWrong")}
    help={t("game.about.reportHelp")}
    action={t("game.about.reportBug")}
    onrun={onreportbug}
  />
</div>

<style>
  .group {
    margin-top: var(--space-5);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--game-card-top, var(--color-surface)) 55%, transparent);
    overflow: hidden;
  }
  .muted { color: var(--color-text-muted); }
</style>
