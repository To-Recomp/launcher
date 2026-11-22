import { launcherAppearance } from "./api";

type Theme = "dark" | "light";

const DARK_QUERY = "(prefers-color-scheme: dark)";

/** The theme and the game colors, applied to the page as soon as they are read and when they change. */
export class AppearanceStore {
  #choice = $state<"dark" | "light" | "system">("dark");
  #palette = $state(true);
  #systemDark = $state(true);

  /** The theme shown: the system's own when "system" is chosen. */
  get theme(): Theme {
    return this.#choice === "system" ? (this.#systemDark ? "dark" : "light") : this.#choice;
  }

  /** The game's logo colors are made for the dark theme: they never tint a light one. */
  get usesGamePalette(): boolean {
    return this.#palette && this.theme === "dark";
  }

  /** Reads the settings and follows the system's theme from now on. */
  async load() {
    if (typeof window !== "undefined" && window.matchMedia) {
      const query = window.matchMedia(DARK_QUERY);
      this.#systemDark = query.matches;
      query.addEventListener?.("change", (e) => {
        this.#systemDark = e.matches;
        this.#apply();
      });
    }
    await this.reload();
  }

  /** Reads the settings again (after the player changed one). */
  async reload() {
    try {
      const found = await launcherAppearance();
      this.#choice = found.theme;
      this.#palette = found.gamePalette;
    } catch {
      // No Tauri runtime (plain browser): the dark theme with game colors stays.
    }
    this.#apply();
  }

  #apply() {
    if (typeof document !== "undefined") document.documentElement.dataset.theme = this.theme;
  }
}

export const appearance = new AppearanceStore();
