import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import App from "./App.svelte";

describe("App", () => {
  it("opens on the Home tab with Eternia selected", () => {
    render(App);
    expect(screen.getByRole("tab", { name: "Home" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("heading", { level: 1, name: "Eternia" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Tales of Eternia/ })).toHaveAttribute("aria-current", "true");
  });

  it("does not expose the hidden Mods tab", () => {
    render(App);
    expect(screen.queryByRole("tab", { name: "Mods" })).not.toBeInTheDocument();
  });

  it("switches the Home title and platforms when another game is selected", async () => {
    render(App);
    await fireEvent.click(screen.getByRole("button", { name: /Tales of Symphonia/ }));
    expect(screen.getByRole("heading", { level: 1, name: "Symphonia" })).toBeInTheDocument();
    const platform = screen.getByRole("combobox", { name: "Platform" }) as HTMLSelectElement;
    expect(platform.value).toBe("GameCube");
  });

  it("switches between Home and Settings", async () => {
    render(App);
    await fireEvent.click(screen.getByRole("tab", { name: "Settings" }));
    expect(screen.getByRole("heading", { level: 1, name: "Settings" })).toBeInTheDocument();
    expect(screen.queryByText("Action status")).not.toBeInTheDocument();
    await fireEvent.click(screen.getByRole("tab", { name: "Home" }));
    expect(screen.getByText("Action status")).toBeInTheDocument();
  });

  it("always shows the non-affiliation notice and the launcher version", async () => {
    render(App);
    expect(screen.getByText(/Not affiliated with or endorsed by/)).toBeInTheDocument();
    expect(within(screen.getByRole("complementary")).getByText("v0.1.0")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("tab", { name: "Settings" }));
    expect(screen.getByText(/Not affiliated with or endorsed by/)).toBeInTheDocument();
  });
});
