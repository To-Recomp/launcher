import { beforeEach, describe, expect, it, vi } from "vitest";

const plugin = vi.hoisted(() => ({ writeText: vi.fn() }));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => plugin);
const opener = vi.hoisted(() => ({ openUrl: vi.fn(), revealItemInDir: vi.fn() }));
vi.mock("@tauri-apps/plugin-opener", () => opener);
const core = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => core);

import { buildReport, copyText, dismissPanic, pendingPanic, issuesUrl, openIssues, revealFile } from "./api";

const web = vi.fn();
beforeEach(() => {
  vi.clearAllMocks();
  Object.defineProperty(navigator, "clipboard", { value: { writeText: web }, configurable: true });
});

describe("copyText", () => {
  it("uses the web clipboard", async () => {
    web.mockResolvedValue(undefined);
    await copyText("hello");
    expect(web).toHaveBeenCalledWith("hello");
    expect(plugin.writeText).not.toHaveBeenCalled();
  });

  it("falls back to the clipboard plugin when the web API refuses", async () => {
    web.mockRejectedValue(new Error("denied"));
    await copyText("héllo\n日本");
    expect(plugin.writeText).toHaveBeenCalledWith("héllo\n日本");
  });
});

describe("issuesUrl", () => {
  it("opens a new issue of the given repository", () => {
    expect(issuesUrl("To-Recomp/launcher")).toBe("https://github.com/To-Recomp/launcher/issues/new");
  });
});

describe("backend and opener calls", () => {
  it("asks the backend for the file, reveals it and opens the issue page", async () => {
    core.invoke.mockResolvedValue({ name: "n", path: "/p", folder: "/", text: "t" });
    await buildReport({ context: "manual" });
    expect(core.invoke).toHaveBeenCalledWith("build_report", { request: { context: "manual" } });
    await revealFile("/p");
    expect(opener.revealItemInDir).toHaveBeenCalledWith("/p");
    await openIssues("To-Recomp/toe");
    expect(opener.openUrl).toHaveBeenCalledWith("https://github.com/To-Recomp/toe/issues/new");
    await pendingPanic();
    expect(core.invoke).toHaveBeenLastCalledWith("pending_panic");
    await dismissPanic();
    expect(core.invoke).toHaveBeenLastCalledWith("dismiss_panic");
  });
});
