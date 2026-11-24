import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const links = vi.hoisted(() => ({ openLink: vi.fn() }));
vi.mock("../diagnostics/api", () => links);

import { LAUNCHER_VERSION } from "../data";
import LauncherAbout from "./LauncherAbout.svelte";

const onreportbug = vi.fn();

beforeEach(() => vi.clearAllMocks());

const open = (label: string) =>
  fireEvent.click(within(screen.getByText(label).closest(".row") as HTMLElement).getByRole("button", { name: "Open" }));

describe("LauncherAbout", () => {
  it("shows the launcher's version and license", () => {
    render(LauncherAbout, { onreportbug });
    expect(screen.getByText(`v${LAUNCHER_VERSION}`)).toBeInTheDocument();
    expect(screen.getByText("GNU GPL v3")).toBeInTheDocument();
  });

  it("opens the launcher's repository and each credit's page", async () => {
    render(LauncherAbout, { onreportbug });
    await open("Repository");
    await open("Built with Tauri");
    expect(links.openLink).toHaveBeenNthCalledWith(1, "https://github.com/To-Recomp/launcher");
    expect(links.openLink).toHaveBeenNthCalledWith(2, "https://github.com/tauri-apps/tauri");
  });

  it("offers Report a bug for the launcher", async () => {
    render(LauncherAbout, { onreportbug });
    await fireEvent.click(screen.getByRole("button", { name: "Report a bug" }));
    expect(onreportbug).toHaveBeenCalledOnce();
  });
});
