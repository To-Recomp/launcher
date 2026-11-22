import { describe, expect, it } from "vitest";
import { contrast, paletteFromPixels, paletteStyle } from "./palette";


function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    return Math.round(255 * (l - s * Math.min(l, 1 - l) * Math.max(-1, Math.min(k - 3, 9 - k, 1))));
  };
  return [f(0), f(8), f(4)];
}
/** `count` identical RGBA pixels. */
function pixels(count: number, rgba: readonly number[]): number[] {
  return Array.from({ length: count }, () => rgba).flat();
}

const rgb = (hex: string): [number, number, number] => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16)) as [number, number, number];
const hue = (hex: string) => {
  const [r, g, b] = rgb(hex);
  const max = Math.max(r, g, b);
  const d = max - Math.min(r, g, b);
  const h = max === r ? ((g - b) / d + (g < b ? 6 : 0)) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return h * 60;
};

describe("paletteFromPixels", () => {
  it.each([
    ["red", [200, 40, 40], 0],
    ["gold", [220, 180, 60], 45],
    ["green", [40, 170, 80], 140],
    ["blue", [50, 90, 210], 225],
    ["purple", [140, 60, 190], 280],
  ] as const)("keeps the hue of a %s logo", (_name, color, expected) => {
    const palette = paletteFromPixels(pixels(200, [...color, 255]))!;
    expect(Math.abs(hue(palette.primary) - expected)).toBeLessThan(12);
    expect(Math.abs(hue(palette.accent) - expected)).toBeLessThan(12);
  });

  it("makes a primary that carries light text, whatever the hue", () => {
    for (const color of [[255, 230, 0], [0, 255, 255], [255, 255, 0], [250, 200, 10], [0, 200, 90]]) {
      const palette = paletteFromPixels(pixels(100, [...color, 255]))!;
      expect(contrast(rgb(palette.primary), [243, 244, 248]), palette.primary).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("follows the dominant color, not the most extreme one", () => {
    const data = [...pixels(300, [200, 40, 40, 255]), ...pixels(40, [40, 90, 210, 255])];
    const palette = paletteFromPixels(data)!;
    expect(hue(palette.primary)).toBeLessThan(15);
  });

  it("gives a dark background that keeps the hue of the logo", () => {
    const palette = paletteFromPixels(pixels(100, [40, 90, 210, 255]))!;
    for (const color of [palette.backgroundTop, palette.backgroundMid]) {
      expect(Math.abs(hue(color) - 225)).toBeLessThan(12);
      expect(contrast(rgb(color), [0, 0, 0]), color).toBeLessThan(2.2);
    }
  });

  it("joins a red that wraps around the hue circle", () => {
    const data = [...pixels(100, [220, 30, 50, 255]), ...pixels(100, [220, 50, 30, 255])];
    const palette = paletteFromPixels(data)!;
    const h = hue(palette.primary);
    expect(h < 15 || h > 345).toBe(true);
  });

  it("ignores transparent pixels", () => {
    const data = [...pixels(500, [40, 90, 210, 0]), ...pixels(50, [200, 40, 40, 255])];
    expect(hue(paletteFromPixels(data)!.primary)).toBeLessThan(15);
  });

  it.each([
    ["nothing", []],
    ["fully transparent", pixels(50, [200, 40, 40, 0])],
    ["grey", pixels(50, [128, 128, 128, 255])],
    ["white", pixels(50, [255, 255, 255, 255])],
    ["black", pixels(50, [0, 0, 0, 255])],
  ])("leaves the theme alone for %s", (_name, data) => {
    expect(paletteFromPixels(data)).toBeNull();
  });

  it("ignores a speck of color in a mostly grey logo", () => {
    expect(paletteFromPixels([...pixels(500, [128, 128, 128, 255]), ...pixels(2, [200, 40, 40, 255])])).toBeNull();
  });

  it("keeps every text readable (WCAG AA) on every background, whatever the hue", () => {
    const MUTED = [162, 166, 184] as const;
    const TEXT = [243, 244, 248] as const;
    const colors = [60, 90, 120, 160, 180, 200, 255, 330].flatMap((h) =>
      [[1, 0.5], [0.8, 0.7], [0.6, 0.45]].map(([s, l]) => hslToRgb(h, s, l)),
    );
    for (let hueDeg = 0; hueDeg < 360; hueDeg += 15) colors.push(hslToRgb(hueDeg, 0.9, 0.55));
    for (const color of colors) {
      const palette = paletteFromPixels(pixels(100, [...color, 255]))!;
      const c = (fg: readonly number[], bg: string) => contrast(fg as [number, number, number], rgb(bg));
      const who = `${palette.accent} on ${color}`;
      for (const bg of [palette.backgroundTop, palette.backgroundMid, palette.cardTop]) {
        expect(c(MUTED, bg), `muted on ${bg} (${who})`).toBeGreaterThanOrEqual(4.5);
        expect(c(TEXT, bg), `text on ${bg} (${who})`).toBeGreaterThanOrEqual(4.5);
        expect(c(rgb(palette.accent), bg), `accent on ${bg} (${who})`).toBeGreaterThanOrEqual(4.5);
      }
      for (const bg of [palette.primary, palette.primaryHover]) {
        expect(c(TEXT, bg), `button text on ${bg} (${who})`).toBeGreaterThanOrEqual(4.5);
      }
    }
  });
});

describe("paletteStyle", () => {
  it("is empty without a palette and overrides the theme with one", () => {
    expect(paletteStyle(null)).toBe("");
    const style = paletteStyle(paletteFromPixels(pixels(100, [200, 40, 40, 255])));
    expect(style).toMatch(/--color-primary: #[0-9a-f]{6};.*--color-primary-hover: .*--color-accent: .*--game-glow: rgb\(/);
  });
});
