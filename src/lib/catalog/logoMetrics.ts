/** What the game page needs to line up a logo with the others. */
export interface LogoMetrics {
  /** Width over height of the image. */
  aspect: number;
  /** Where the lettering's baseline falls, from the top of the image (0 to 1): below it only descenders hang. */
  baseline: number;
}

/** A pixel counts as part of the lettering from this opacity. */
const OPAQUE = 128;

/**
 * The baseline of a logo made of letters (RGBA bytes, `width` x `height`). Most letters end on the
 * baseline and a few (y, p, g, q, j) hang under it, so the baseline is where most columns of the
 * lettering end; the lowest of those columns is it, and what goes lower is a descender.
 * Returns 1 for an image with nothing in it (no descender to account for).
 */
export function baselineOf(data: ArrayLike<number>, width: number, height: number): number {
  // The last opaque row of each column that has any.
  const bottoms: number[] = [];
  for (let x = 0; x < width; x++) {
    for (let y = height - 1; y >= 0; y--) {
      if (data[(y * width + x) * 4 + 3] >= OPAQUE) {
        bottoms.push(y);
        break;
      }
    }
  }
  if (bottoms.length === 0) return 1;

  // Group the columns by where they end (bins of about 3% of the height); the fullest bin, and the
  // nearer one when two tie, is the baseline's.
  const bin = Math.max(1, Math.round(height * 0.03));
  const counts = new Map<number, number>();
  for (const bottom of bottoms) counts.set(Math.floor(bottom / bin), (counts.get(Math.floor(bottom / bin)) ?? 0) + 1);
  let best = 0;
  for (const [key, count] of counts) {
    if (count > (counts.get(best) ?? 0) || (count === counts.get(best) && key > best)) best = key;
  }
  // Letters of the baseline's bin and of the bin next to it (an overshoot like an "S" or an "o").
  const onBaseline = bottoms.filter((b) => Math.abs(Math.floor(b / bin) - best) <= 0);
  return (Math.max(...onBaseline) + 1) / height;
}

const cache = new Map<string, Promise<LogoMetrics | null>>();

/** Reads a logo through a canvas. Null when the image or the canvas is unavailable (the logo then shows as it is). */
export function loadLogoMetrics(url: string): Promise<LogoMetrics | null> {
  let found = cache.get(url);
  if (!found) {
    const { promise, resolve } = Promise.withResolvers<LogoMetrics | null>();
    const img = new Image();
    img.onload = () => {
      try {
        const width = 256;
        const height = Math.max(1, Math.round((width * img.naturalHeight) / Math.max(1, img.naturalWidth)));
        const canvas = document.createElement("canvas");
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext("2d");
        if (!ctx) return resolve(null);
        ctx.drawImage(img, 0, 0, width, height);
        const { data } = ctx.getImageData(0, 0, width, height);
        resolve({ aspect: width / height, baseline: baselineOf(data, width, height) });
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
