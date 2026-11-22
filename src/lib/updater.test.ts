import { beforeEach, describe, expect, it, vi } from "vitest";

const core = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => core);

import { updaterAvailable } from "./updater";

beforeEach(() => vi.clearAllMocks());

describe("updaterAvailable", () => {
  it("is what the backend says", async () => {
    core.invoke.mockResolvedValue(true);
    expect(await updaterAvailable()).toBe(true);
  });

  it("is false without a Tauri runtime, instead of failing", async () => {
    core.invoke.mockRejectedValue(new TypeError("invoke is not defined"));
    expect(await updaterAvailable()).toBe(false);
  });
});
