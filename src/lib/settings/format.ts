/** printf-style number formats of a slider (`%g` by default, `%d`, `%.Nf`); mirrors the backend. */
export function formatNumber(format: string | null, value: number): string {
  if (format === "%d") return String(Math.round(value));
  const fixed = format?.match(/^%\.(\d+)f$/);
  if (fixed) return value.toFixed(Number(fixed[1]));
  return String(value);
}
