/** Colors a game page takes from its logo. All values are CSS colors. */
export interface Palette {
  /** Buttons: dark enough for light text. */
  primary: string;
  primaryHover: string;
  /** Highlights, progress and links: light enough for the dark background. */
  accent: string;
  /** A faint wash behind the page header. */
  glow: string;
  /** The page background fades from `backgroundTop` through `backgroundMid` to the theme's own. */
  backgroundTop: string;
  backgroundMid: string;
  /** Start of the highlighted card's gradient: a lighter shade that still carries small text. */
  cardTop: string;
}

type Rgb = [number, number, number];

const BINS = 24;
/** Text on the primary button is the theme's near-white. */
const LIGHT_TEXT: Rgb = [243, 244, 248];
const MIN_CONTRAST = 4.5;
/** Secondary text (`--color-text-muted`) must stay readable on every background of a palette. */
const MUTED_TEXT: Rgb = [162, 166, 184];
/** A little above WCAG AA's 4.5:1 so rounding to 8-bit colors can't drop below it. */
const READABLE = 4.6;

/** Moves the lightness from `l` by `step` (1% = 0.01) until `ok` holds, stopping before black or white. */
function adjustUntil(h: number, s: number, l: number, step: number, ok: (color: Rgb) => boolean): Rgb {
  let color = fromHsl(h, s, l);
  while (!ok(color) && l + step > 0.08 && l + step < 0.95) {
    l += step;
    color = fromHsl(h, s, l);
  }
  return color;
}

function toHsl([r, g, b]: Rgb): [number, number, number] {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  const d = max - min;
  if (d === 0) return [0, 0, l];
  const s = d / (1 - Math.abs(2 * l - 1));
  const h = max === r ? ((g - b) / d + (g < b ? 6 : 0)) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [h * 60, s, l];
}

function fromHsl(h: number, s: number, l: number): Rgb {
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  const [r, g, b] = h < 60 ? [c, x, 0] : h < 120 ? [x, c, 0] : h < 180 ? [0, c, x] : h < 240 ? [0, x, c] : h < 300 ? [x, 0, c] : [c, 0, x];
  return [Math.round((r + m) * 255), Math.round((g + m) * 255), Math.round((b + m) * 255)];
}

function luminance([r, g, b]: Rgb): number {
  const lin = (v: number) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

export function contrast(a: Rgb, b: Rgb): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
const hex = (rgb: Rgb) => `#${rgb.map((v) => v.toString(16).padStart(2, "0")).join("")}`;

/**
 * The dominant saturated hue of an image (RGBA bytes), turned into a palette that stays readable
 * on the launcher's dark theme. Transparent, near-white, near-black and grey pixels are ignored.
 * Returns null when nothing is colorful enough (the default theme then stays).
 */
export function paletteFromPixels(data: ArrayLike<number>): Palette | null {
  const weight = new Array<number>(BINS).fill(0);
  const sum = Array.from({ length: BINS }, () => [0, 0, 0]);
  const count = new Array<number>(BINS).fill(0);
  let opaque = 0;

  for (let i = 0; i + 3 < data.length; i += 4) {
    if (data[i + 3] < 128) continue;
    opaque++;
    const rgb: Rgb = [data[i], data[i + 1], data[i + 2]];
    const [h, s, l] = toHsl(rgb);
    if (s < 0.15 || l < 0.15 || l > 0.92) continue;
    const w = s * (1 - Math.abs(2 * l - 1));
    const bin = Math.min(BINS - 1, Math.floor((h / 360) * BINS));
    weight[bin] += w;
    count[bin]++;
    for (let k = 0; k < 3; k++) sum[bin][k] += rgb[k] * w;
  }
  if (opaque === 0) return null;

  // A hue spreads over neighbouring bins (red wraps around), so score with the neighbours.
  const score = weight.map((w, i) => w + 0.5 * (weight[(i + BINS - 1) % BINS] + weight[(i + 1) % BINS]));
  const best = score.indexOf(Math.max(...score));
  // Colorful pixels around the winning hue must be a real share of the logo, not a few specks.
  const around = count[best] + count[(best + BINS - 1) % BINS] + count[(best + 1) % BINS];
  if (weight[best] === 0 || around < opaque * 0.05) return null;

  const base: Rgb = [0, 1, 2].map((k) => Math.round(sum[best][k] / weight[best])) as Rgb;
  const [h, s0] = toHsl(base);
  const s = clamp(s0, 0.45, 0.8);

  const lightText = (color: Rgb) => contrast(color, LIGHT_TEXT) >= MIN_CONTRAST;
  const primary = adjustUntil(h, s, 0.45, -0.01, lightText);
  const hover = adjustUntil(h, s, 0.51, -0.01, lightText);
  const mutedReadable = (color: Rgb) => contrast(MUTED_TEXT, color) >= READABLE;
  const sTop = clamp(s0, 0.4, 0.6);
  // Backgrounds are only ever darkened from these lightest shades, so an accent that reads on
  // the lightest one reads on all of them.
  const lightestCard = fromHsl(h, sTop, 0.26);
  const accent = adjustUntil(h, clamp(s0, 0.5, 0.85), 0.68, 0.01, (c) => contrast(c, lightestCard) >= READABLE);
  return {
    primary: hex(primary),
    primaryHover: hex(hover),
    accent: hex(accent),
    glow: `rgb(${base.join(" ")} / 0.28)`,
    backgroundTop: hex(adjustUntil(h, sTop, 0.2, -0.01, mutedReadable)),
    backgroundMid: hex(adjustUntil(h, clamp(s0, 0.3, 0.45), 0.13, -0.01, mutedReadable)),
    cardTop: hex(adjustUntil(h, sTop, 0.26, -0.01, mutedReadable)),
  };
}

/** CSS custom properties for an element: they override the theme inside it. */
export function paletteStyle(palette: Palette | null): string {
  if (!palette) return "";
  return `--color-primary: ${palette.primary}; --color-primary-hover: ${palette.primaryHover}; --color-accent: ${palette.accent}; --color-info: ${palette.accent}; --game-glow: ${palette.glow}; --game-bg-top: ${palette.backgroundTop}; --game-bg-mid: ${palette.backgroundMid}; --game-card-top: ${palette.cardTop};`;
}

const cache = new Map<string, Promise<Palette | null>>();

/** Reads the logo at `url` through a canvas. Null when the image or the canvas is unavailable. */
export function loadPalette(url: string): Promise<Palette | null> {
  let found = cache.get(url);
  if (!found) {
    const { promise, resolve } = Promise.withResolvers<Palette | null>();
    const img = new Image();
    img.onload = () => {
      try {
        const width = 96;
        const height = Math.max(1, Math.round((width * img.naturalHeight) / Math.max(1, img.naturalWidth)));
        const canvas = document.createElement("canvas");
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext("2d");
        if (!ctx) return resolve(null);
        ctx.drawImage(img, 0, 0, width, height);
        resolve(paletteFromPixels(ctx.getImageData(0, 0, width, height).data));
      } catch {
        resolve(null);
      }
    };
    img.onerror = () => resolve(null);
    img.src = url;
    found = promise;
    cache.set(url, found);
  }
  return found;
}
