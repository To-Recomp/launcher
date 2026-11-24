<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import ActionRow from "./ActionRow.svelte";
  import Row from "./Row.svelte";
  import Segmented from "./Segmented.svelte";
  import { t } from "../i18n/index.svelte";
  import { proxySettings, setProxy, testProxy, type ProxySettings, type ProxyView } from "../launcher/api";
  import { errorMessage } from "../settings/api";

  /** Rows placed at the top of the card (the game download speed). */
  let { children }: { children?: Snippet } = $props();

  const NONE: ProxySettings = { mode: "system", protocol: "http", host: "", port: 8080, username: "", noProxy: "" };

  let saved = $state<ProxyView | null>(null);
  let draft = $state<ProxySettings>({ ...NONE });
  /** What the player typed in the password box; `forget` clears the saved one. */
  let password = $state("");
  let forget = $state(false);
  let notice = $state<{ text: string; failed: boolean } | null>(null);

  const modes = $derived([
    { value: "system" as const, label: t("launcher.proxy.mode.system") },
    { value: "none" as const, label: t("launcher.proxy.mode.none") },
    { value: "manual" as const, label: t("launcher.proxy.mode.manual") },
  ]);
  const protocols = [
    { value: "http" as const, label: "HTTP" },
    { value: "https" as const, label: "HTTPS" },
    { value: "socks5" as const, label: "SOCKS5" },
  ];

  const passwordChange = $derived(forget ? "" : password === "" ? null : password);

  function show(view: ProxyView) {
    saved = view;
    draft = { ...view.settings };
    password = "";
    forget = false;
  }

  onMount(async () => {
    try {
      show(await proxySettings());
    } catch (e) {
      notice = { text: errorMessage(e), failed: true };
    }
  });

  async function save() {
    try {
      show(await setProxy(draft, passwordChange));
      notice = { text: t("launcher.proxy.saved"), failed: false };
    } catch (e) {
      notice = { text: errorMessage(e), failed: true };
    }
  }

  async function test() {
    return testProxy(draft, passwordChange);
  }
</script>

<h2>{t("launcher.proxy.heading")}</h2>
{#if children || saved}
<div class="group">
  {@render children?.()}
  {#if saved}
  <Row label={t("launcher.proxy.label")} help={t("launcher.proxy.help")}>
    <Segmented
      label={t("launcher.proxy.label")}
      options={modes.map((m) => m.label)}
      value={modes.find((m) => m.value === draft.mode)?.label ?? ""}
      onchange={(label) => (draft.mode = modes.find((m) => m.label === label)!.value)}
    />
  </Row>
  {#if draft.mode === "manual"}
    <Row label={t("launcher.proxy.protocol")}>
      <Segmented
        label={t("launcher.proxy.protocol")}
        options={protocols.map((p) => p.label)}
        value={protocols.find((p) => p.value === draft.protocol)?.label ?? ""}
        onchange={(label) => (draft.protocol = protocols.find((p) => p.label === label)!.value)}
      />
    </Row>
    <Row label={t("launcher.proxy.address")}>
      <div class="pair">
        <input type="text" spellcheck="false" aria-label={t("launcher.proxy.host")} placeholder="proxy.example.com" bind:value={draft.host} />
        <input class="port" type="number" min="1" max="65535" aria-label={t("launcher.proxy.port")} bind:value={draft.port} />
      </div>
    </Row>
    <Row label={t("launcher.proxy.username")}>
      <input type="text" spellcheck="false" autocomplete="off" aria-label={t("launcher.proxy.username")} bind:value={draft.username} />
    </Row>
    <Row
      label={t("launcher.proxy.password")}
      help={saved?.keychainError ?? t("launcher.proxy.passwordHelp")}
    >
      <div class="pair">
        <input
          type="password"
          autocomplete="off"
          aria-label={t("launcher.proxy.password")}
          placeholder={saved?.hasPassword && !forget ? t("launcher.proxy.passwordSaved") : ""}
          bind:value={password}
          disabled={forget}
        />
        {#if saved?.hasPassword}
          <button class="small" onclick={() => (forget = !forget)}>
            {forget ? t("launcher.proxy.keepPassword") : t("launcher.proxy.forgetPassword")}
          </button>
        {/if}
      </div>
    </Row>
    <Row label={t("launcher.proxy.noProxy")} help={t("launcher.proxy.noProxyHelp")}>
      <input type="text" spellcheck="false" aria-label={t("launcher.proxy.noProxy")} placeholder="localhost, .intranet.example" bind:value={draft.noProxy} />
    </Row>
  {/if}
  <ActionRow label={t("launcher.proxy.test")} help={t("launcher.proxy.testHelp")} action={t("launcher.proxy.testAction")} onrun={test} />
  <Row label={t("launcher.proxy.saveLabel")} help={t("launcher.proxy.saveHelp")}>
    <div class="pair">
      {#if notice}
        <span class="notice" class:failed={notice.failed} role={notice.failed ? "alert" : "status"}>{notice.text}</span>
      {/if}
      <button onclick={save}>{t("launcher.proxy.save")}</button>
    </div>
  </Row>
  {/if}
</div>
{/if}
{#if !saved && notice}<p class="notice failed" role="alert">{notice.text}</p>{/if}

<style>
  h2 { margin: var(--space-5) 0 var(--space-2) var(--space-4); font-size: 12px; font-weight: 500; color: var(--color-text-muted); }
  .group {
    margin-top: var(--space-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    /* No overflow clipping: the download speed's dropdown opens over the edge of the card. */
  }
  .pair { display: flex; align-items: center; gap: var(--space-2); }
  input {
    min-width: 216px;
    padding: 8px 14px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-control);
    font: inherit;
    font-size: 13px;
    color: inherit;
    user-select: text;
  }
  input.port { min-width: 0; width: 84px; }
  input:disabled { opacity: 0.5; }
  button {
    padding: 6px 14px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-pill);
    background: var(--color-surface-alt);
    font-weight: 500;
  }
  .notice { max-width: 420px; font-size: 13px; color: var(--color-text-muted); }
  .notice.failed { color: var(--color-primary); }
</style>
