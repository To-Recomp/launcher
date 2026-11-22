import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ProxyView } from "../launcher/api";

const api = vi.hoisted(() => ({ proxySettings: vi.fn(), setProxy: vi.fn(), testProxy: vi.fn() }));
vi.mock("../launcher/api", () => api);
vi.mock("../settings/api", () => ({ errorMessage: (e: unknown) => String(e) }));

import LauncherProxy from "./LauncherProxy.svelte";

const view = (over: Partial<ProxyView["settings"]> = {}, extra: Partial<ProxyView> = {}): ProxyView => ({
  settings: { mode: "system", protocol: "http", host: "", port: 8080, username: "", noProxy: "", ...over },
  hasPassword: false,
  keychainError: null,
  ...extra,
});

beforeEach(() => {
  vi.clearAllMocks();
  api.proxySettings.mockResolvedValue(view());
});

async function chooseManual() {
  await fireEvent.click(await screen.findByRole("radio", { name: "Manual" }));
  await screen.findByRole("textbox", { name: "Proxy host" });
}

describe("LauncherProxy", () => {
  it("asks for nothing more than the mode while the system's proxy is used", async () => {
    render(LauncherProxy);
    await screen.findByRole("radio", { name: "System" });
    expect(screen.queryByRole("textbox", { name: "Proxy host" })).not.toBeInTheDocument();
  });

  it("saves a manual proxy with its login, and shows that the password is kept in the keychain", async () => {
    api.setProxy.mockResolvedValue(view({ mode: "manual", host: "proxy.corp", port: 3128, username: "yuri" }, { hasPassword: true }));
    render(LauncherProxy);
    await chooseManual();
    await fireEvent.input(screen.getByRole("textbox", { name: "Proxy host" }), { target: { value: "proxy.corp" } });
    await fireEvent.input(screen.getByRole("spinbutton", { name: "Proxy port" }), { target: { value: "3128" } });
    await fireEvent.input(screen.getByRole("textbox", { name: "User name" }), { target: { value: "yuri" } });
    await fireEvent.input(screen.getByLabelText("Password"), { target: { value: "hunter2" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() =>
      expect(api.setProxy).toHaveBeenCalledWith(
        { mode: "manual", protocol: "http", host: "proxy.corp", port: 3128, username: "yuri", noProxy: "" },
        "hunter2",
      ),
    );
    expect(await screen.findByRole("status")).toHaveTextContent("Saved.");
    expect(screen.getByLabelText("Password")).toHaveValue("");
    expect(screen.getByPlaceholderText("Saved in the keychain")).toBeInTheDocument();
  });

  it("keeps the saved password unless told to forget it", async () => {
    api.proxySettings.mockResolvedValue(view({ mode: "manual", host: "proxy.corp", username: "yuri" }, { hasPassword: true }));
    api.setProxy.mockResolvedValue(view({ mode: "manual", host: "proxy.corp", username: "yuri" }, { hasPassword: true }));
    render(LauncherProxy);
    await screen.findByRole("textbox", { name: "Proxy host" });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(api.setProxy).toHaveBeenLastCalledWith(expect.anything(), null));
    await fireEvent.click(await screen.findByRole("button", { name: "Forget" }));
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => expect(api.setProxy).toHaveBeenLastCalledWith(expect.anything(), ""));
  });

  it("says why nothing was saved, such as a keychain that isn't there", async () => {
    api.setProxy.mockRejectedValue("The system keychain isn't available, so the password can't be kept safely");
    render(LauncherProxy);
    await chooseManual();
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("keychain isn't available");
  });

  it("tests the values on the form, saved or not, and reports the precise failure", async () => {
    api.testProxy.mockRejectedValue("The proxy refused the connection: nothing is listening at proxy.corp:3128.");
    render(LauncherProxy);
    await chooseManual();
    await fireEvent.input(screen.getByRole("textbox", { name: "Proxy host" }), { target: { value: "proxy.corp" } });
    await fireEvent.click(screen.getByRole("button", { name: "Test" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("nothing is listening at proxy.corp:3128");
    expect(api.testProxy).toHaveBeenCalledWith(expect.objectContaining({ mode: "manual", host: "proxy.corp" }), null);
    expect(api.setProxy).not.toHaveBeenCalled();
  });

  it("tests without the saved password once Forget is pending", async () => {
    api.proxySettings.mockResolvedValue(view({ mode: "manual", host: "proxy.corp", username: "yuri" }, { hasPassword: true }));
    api.testProxy.mockResolvedValue("Connected: GitHub answered 200.");
    render(LauncherProxy);
    await fireEvent.click(await screen.findByRole("button", { name: "Test" }));
    await waitFor(() => expect(api.testProxy).toHaveBeenLastCalledWith(expect.anything(), null));
    await fireEvent.click(screen.getByRole("button", { name: "Forget" }));
    await fireEvent.click(screen.getByRole("button", { name: "Test" }));
    await waitFor(() => expect(api.testProxy).toHaveBeenLastCalledWith(expect.anything(), ""));
  });

  it("shows the keychain's problem next to the password", async () => {
    api.proxySettings.mockResolvedValue(view({ mode: "manual" }, { keychainError: "The system keychain isn't available" }));
    render(LauncherProxy);
    expect(await screen.findByText("The system keychain isn't available")).toBeInTheDocument();
  });
});
