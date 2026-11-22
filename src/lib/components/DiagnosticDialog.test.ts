import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({ copyText: vi.fn(), revealFile: vi.fn(), openIssues: vi.fn() }));
vi.mock("../diagnostics/api", () => api);

import DiagnosticDialog from "./DiagnosticDialog.svelte";

const file = {
  name: "toe-diagnostic-2026-11-22-154045.md",
  path: "/logs/toe-diagnostic-2026-11-22-154045.md",
  folder: "/logs",
  text: "Diagnostic format: 1\n\n| Field | Value |\n|---|---|\n| Game | toe |\n",
};
const onclose = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  api.copyText.mockResolvedValue(undefined);
  api.revealFile.mockResolvedValue(undefined);
  api.openIssues.mockResolvedValue(undefined);
});

describe("DiagnosticDialog", () => {
  it("shows the file as it will be published, with the warning that it will be public", () => {
    render(DiagnosticDialog, { diagnostic: { repository: "To-Recomp/toe", file, error: null }, onclose });
    expect(screen.getByRole("note")).toHaveTextContent("Review this file before you publish it: it will be public.");
    expect(screen.getByLabelText("File content").textContent).toBe(file.text);
    expect(screen.getByText("To-Recomp/toe")).toBeInTheDocument();
  });

  it("copies exactly the file content", async () => {
    render(DiagnosticDialog, { diagnostic: { repository: "To-Recomp/toe", file, error: null }, onclose });
    await fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    expect(api.copyText).toHaveBeenCalledWith(file.text);
    expect(await screen.findByText("Copied to the clipboard.")).toBeInTheDocument();
  });

  it("reveals the file and opens the issues of the repository it was made for", async () => {
    render(DiagnosticDialog, { diagnostic: { repository: "To-Recomp/launcher", file, error: null }, onclose });
    await fireEvent.click(screen.getByRole("button", { name: "Open folder" }));
    expect(api.revealFile).toHaveBeenCalledWith(file.path);
    await fireEvent.click(screen.getByRole("button", { name: "Open issues" }));
    expect(api.openIssues).toHaveBeenCalledWith("To-Recomp/launcher");
    await fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(onclose).toHaveBeenCalledOnce();
  });

  it("says when copying or opening fails, and doesn't claim it copied", async () => {
    api.copyText.mockRejectedValue("clipboard unavailable");
    render(DiagnosticDialog, { diagnostic: { repository: "To-Recomp/toe", file, error: null }, onclose });
    await fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("clipboard unavailable");
    expect(screen.queryByText("Copied to the clipboard.")).not.toBeInTheDocument();
  });

  it("shows why no file was made, with only Close", () => {
    render(DiagnosticDialog, { diagnostic: { repository: "To-Recomp/toe", file: null, error: "Can't write the diagnostic file" }, onclose });
    expect(screen.getByRole("alert")).toHaveTextContent("Can't write the diagnostic file");
    expect(screen.queryByRole("button", { name: "Copy" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Close" })).toBeInTheDocument();
  });
});
