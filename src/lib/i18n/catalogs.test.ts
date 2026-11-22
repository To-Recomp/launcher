import { act, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../launcher/api", () => ({ restartLauncher: vi.fn() }));
vi.mock("../settings/api", () => ({ errorMessage: String }));

import RestartBanner from "../components/RestartBanner.svelte";
import { catalogs, FALLBACK_LOCALE, setLocale, type Catalog } from "./index.svelte";

const placeholders = (text: string) => [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

afterEach(() => {
  setLocale(FALLBACK_LOCALE);
  delete catalogs.xx;
});

describe("message catalogs", () => {
  it("only translate messages that exist in English, with the same placeholders", () => {
    const english = catalogs[FALLBACK_LOCALE] as Record<string, string>;
    for (const [code, catalog] of Object.entries(catalogs)) {
      for (const [key, text] of Object.entries(catalog as Record<string, string>)) {
        expect(english[key], `${code}: ${key} isn't an English message`).toBeDefined();
        expect(placeholders(text), `${code}: ${key}`).toEqual(placeholders(english[key]));
      }
    }
  });

  it("French translates every message", () => {
    const english = Object.keys(catalogs[FALLBACK_LOCALE]);
    const french = Object.keys(catalogs.fr);
    expect(english.filter((key) => !french.includes(key))).toEqual([]);
  });

  it("change the text on screen when the language changes, falling back to English for the rest", async () => {
    catalogs.xx = { "launcher.restart.message": "Redémarrez pour appliquer." } as Catalog;
    render(RestartBanner);
    expect(screen.getByText("Restart the launcher to apply these changes.")).toBeInTheDocument();
    await act(() => setLocale("xx"));
    expect(screen.getByText("Redémarrez pour appliquer.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Restart now" })).toBeInTheDocument(); // untranslated: English
  });
});
