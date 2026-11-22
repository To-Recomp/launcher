import { locale } from "./i18n/index.svelte";

/** "812 B", "3.4 KB", "1.2 MB", with the interface language's number format. */
export function formatSize(bytes: number): string {
  const number = (value: number) => new Intl.NumberFormat(locale(), { maximumFractionDigits: 1 }).format(value);
  if (bytes < 1024) return `${number(bytes)} B`;
  if (bytes < 1024 * 1024) return `${number(bytes / 1024)} KB`;
  return `${number(bytes / (1024 * 1024))} MB`;
}
