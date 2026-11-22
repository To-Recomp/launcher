import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const api = vi.hoisted(() => ({ launcherLanguage: vi.fn() }));
vi.mock("../launcher/api", () => api);

import { FALLBACK_LOCALE, locale, setLocale } from "./index.svelte";
import { startLanguage } from "./start";

beforeEach(() => vi.clearAllMocks());
afterEach(() => setLocale(FALLBACK_LOCALE));

describe("startLanguage", () => {
  it("uses the language chosen in the launcher settings, whatever the system says", async () => {
    api.launcherLanguage.mockResolvedValue("fr");
    await startLanguage(["en-US"]);
    expect(locale()).toBe("fr");
  });

  it("follows the system when none is chosen", async () => {
    api.launcherLanguage.mockResolvedValue(null);
    await startLanguage(["fr-FR", "en-US"]);
    expect(locale()).toBe("fr");
  });

  it("falls back to English for a system language that isn't shipped, or a choice that no longer is", async () => {
    api.launcherLanguage.mockResolvedValue(null);
    await startLanguage(["ja-JP"]);
    expect(locale()).toBe("en");
    api.launcherLanguage.mockResolvedValue("klingon");
    await startLanguage(["fr-FR"]);
    expect(locale()).toBe("en");
  });

  it("still starts when the settings can't be read", async () => {
    api.launcherLanguage.mockRejectedValue("no runtime");
    await startLanguage(["fr-FR"]);
    expect(locale()).toBe("fr");
  });
});
