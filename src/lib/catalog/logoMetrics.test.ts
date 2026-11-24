import { describe, expect, it } from "vitest";
import { baselineOf } from "./logoMetrics";

/** An image of `height` rows where each column is opaque from `top` down to its own last row. */
function logo(width: number, height: number, bottoms: number[], top = 2): number[] {
  const data = new Array<number>(width * height * 4).fill(0);
  for (let x = 0; x < width; x++) {
    for (let y = top; y <= bottoms[x]; y++) data[(y * width + x) * 4 + 3] = 255;
  }
  return data;
}

describe("baselineOf", () => {
  it("is the bottom of a lettering that has no descender", () => {
    const bottoms = Array.from({ length: 40 }, () => 79);
    expect(baselineOf(logo(40, 100, bottoms), 40, 100)).toBeCloseTo(0.8, 2);
  });

  it("ignores the letters that hang under the baseline (a y and a p)", () => {
    const bottoms = Array.from({ length: 40 }, (_, x) => (x >= 10 && x < 14) || (x >= 28 && x < 33) ? 97 : 79);
    expect(baselineOf(logo(40, 100, bottoms), 40, 100)).toBeCloseTo(0.8, 2);
  });

  it("keeps the slight overshoot of round letters on the baseline", () => {
    const bottoms = Array.from({ length: 40 }, (_, x) => (x % 5 === 0 ? 80 : 79));
    const baseline = baselineOf(logo(40, 100, bottoms), 40, 100);
    expect(baseline).toBeGreaterThanOrEqual(0.8);
    expect(baseline).toBeLessThan(0.83);
  });

  it("is the whole image when it has nothing in it", () => {
    expect(baselineOf(new Array(40 * 100 * 4).fill(0), 40, 100)).toBe(1);
  });
});
