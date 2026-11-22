import { en } from "./en";

/** Every message of the interface; English is the source, other languages translate what they can. */
export type MessageKey = keyof typeof en;
export type Catalog = Partial<Record<MessageKey, string>>;

/** The languages the launcher ships, by code. Adding one is adding its catalog here. */
export const catalogs: Record<string, Catalog> = { en };

export const FALLBACK_LOCALE = "en";

class Language {
  code = $state(FALLBACK_LOCALE);
}
const language = new Language();

/** The language of the interface. */
export const locale = () => language.code;

export const availableLocales = () => Object.keys(catalogs);

/** The shipped language that best matches the system's preferences (`en-GB` -> `en`), else English. */
export function detectLocale(preferred: readonly string[]): string {
  for (const wanted of preferred) {
    const code = wanted.toLowerCase();
    const exact = availableLocales().find((l) => l.toLowerCase() === code);
    if (exact) return exact;
    const base = code.split("-")[0];
    const sameLanguage = availableLocales().find((l) => l.toLowerCase().split("-")[0] === base);
    if (sameLanguage) return sameLanguage;
  }
  return FALLBACK_LOCALE;
}

/** Switches the interface to `code`; a language that isn't shipped is refused. */
export function setLocale(code: string) {
  if (!(code in catalogs)) throw new Error(`The interface isn't translated to ${code}.`);
  language.code = code;
  if (typeof document !== "undefined") document.documentElement.lang = code;
}

/**
 * The message for `key` in the current language, else in English. `{name}` placeholders take
 * their value from `params`. Read inside a template or a `$derived`, it follows language changes.
 */
export function t(key: MessageKey, params?: Record<string, string | number>): string {
  const text: string = catalogs[language.code]?.[key] ?? en[key] ?? key;
  if (!params) return text;
  return text.replace(/\{(\w+)\}/g, (whole, name: string) => (name in params ? String(params[name]) : whole));
}

/**
 * A message whose placeholders stand for markup the template builds (`<code>`, `<strong>`): its
 * text pieces, and the name of each placeholder in `names` where it falls, in the language's order.
 */
export function tParts<N extends string>(key: MessageKey, names: readonly N[]): ({ text: string } | N)[] {
  const text: string = catalogs[language.code]?.[key] ?? en[key] ?? key;
  return text
    .split(/\{(\w+)\}/)
    .map((piece, i) => (i % 2 === 1 && (names as readonly string[]).includes(piece) ? (piece as N) : { text: i % 2 === 1 ? `{${piece}}` : piece }))
    .filter((part) => typeof part !== "object" || part.text !== "");
}
