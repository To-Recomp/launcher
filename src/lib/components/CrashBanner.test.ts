import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({ copyText: vi.fn() }));
vi.mock("../diagnostics/api", () => api);

import CrashBanner from "./CrashBanner.svelte";

const file = { name: "launcher-diagnostic-x.md", path: "/logs/x.md", folder: "/logs", text: "Diagnostic format: 1\n" };
const onview = vi.fn();
const ondismiss = vi.fn();

beforeEach(() => vi.clearAllMocks());

describe("CrashBanner", () => {
  it("says the launcher stopped, and views, copies or dismisses the file", async () => {
    api.copyText.mockResolvedValue(undefined);
    render(CrashBanner, { file, onview, ondismiss });
    expect(screen.getByRole("alert")).toHaveTextContent("The launcher stopped unexpectedly");

    await fireEvent.click(screen.getByRole("button", { name: "View file" }));
    expect(onview).toHaveBeenCalledOnce();

    await fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    expect(api.copyText).toHaveBeenCalledWith(file.text);
    expect(await screen.findByText(/Copied\./)).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(ondismiss).toHaveBeenCalledOnce();
  });

  it("says when the copy fails", async () => {
    api.copyText.mockRejectedValue("denied");
    render(CrashBanner, { file, onview, ondismiss });
    await fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    expect(await screen.findByText(/denied/)).toBeInTheDocument();
  });
});
