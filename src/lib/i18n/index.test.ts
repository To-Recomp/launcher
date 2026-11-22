import { afterEach, describe, expect, it } from "vitest";
import { catalogs, detectLocale, FALLBACK_LOCALE, locale, setLocale, t, type Catalog } from "./index.svelte";

const SAMPLE = "shell.sample" as never;

afterEach(() => {
  setLocale(FALLBACK_LOCALE);
  delete (catalogs.en as Record<string, string>)["shell.sample"];
  delete catalogs.xx;
});

describe("i18n", () => {
  it("answers in the current language and falls back to English for what it doesn't translate", () => {
    (catalogs.en as Record<string, string>)["shell.sample"] = "Hello {name}";
    (catalogs.en as Record<string, string>)["shell.other"] = "Other";
    catalogs.xx = { [SAMPLE]: "Bonjour {name}" } as Catalog;
    setLocale("xx");
    expect(t(SAMPLE, { name: "Yuri" })).toBe("Bonjour Yuri");
    expect(t("shell.other" as never)).toBe("Other");
    delete (catalogs.en as Record<string, string>)["shell.other"];
  });

  it("leaves a placeholder that has no value, and a key nobody knows, as they are", () => {
    (catalogs.en as Record<string, string>)["shell.sample"] = "Hello {name}";
    expect(t(SAMPLE)).toBe("Hello {name}");
    expect(t("shell.nobody" as never)).toBe("shell.nobody");
  });

  it("refuses a language that isn't shipped and keeps the current one", () => {
    expect(() => setLocale("zz")).toThrow("isn't translated to zz");
    expect(locale()).toBe("en");
  });

  it("picks the shipped language closest to the system's preferences", () => {
    catalogs.xx = {};
    expect(detectLocale(["fr-FR", "xx-YY", "en-GB"])).toBe("xx");
    expect(detectLocale(["en-GB"])).toBe("en");
    expect(detectLocale(["ja-JP"])).toBe("en");
    expect(detectLocale([])).toBe("en");
  });

  it("tells the document its language", () => {
    catalogs.xx = {};
    setLocale("xx");
    expect(document.documentElement.lang).toBe("xx");
  });
});
