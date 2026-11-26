import { describe, expect, it } from "vitest";
// @ts-expect-error plain JS module, no type declarations
import { titleProblem } from "./check-pr-title.mjs";

describe("titleProblem", () => {
  it("accepts conventional titles, with a scope or a breaking mark", () => {
    for (const title of ["feat: add a page", "fix(ui): crash on start", "feat(api)!: drop v1", "chore(deps): Bump vite from 8 to 9"]) {
      expect(titleProblem(title), title).toBeNull();
    }
  });

  it("rejects what release-please would silently ignore", () => {
    for (const title of ["Add a page", "feat add a page", "feat:no space", "feat: ", "release: 0.2.0", "feature: x"]) {
      expect(titleProblem(title), title).not.toBeNull();
    }
  });
});
