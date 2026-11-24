import { launcherAppearance } from "./api";

type Theme = "dark" | "light";

const DARK_QUERY = "(prefers-color-scheme: dark)";

/** The theme, applied to the page as soon as it is read and when it changes. */
export class AppearanceStore {
  #choice = $state<"dark" | "light" | "system">("dark");
  #systemDark = $state(true);

  /** The theme shown: the system's own when "system" is chosen. */
  get theme(): Theme {
    return this.#choice === "system" ? (this.#systemDark ? "dark" : "light") : this.#choice;
  }

  /** The game's logo colors are made for the dark theme: they never tint a light one. */
  get usesGamePalette(): boolean {
    return this.theme === "dark";
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
    } catch {
      // No Tauri runtime (plain browser): the dark theme stays.
    }
    this.#apply();
  }

  #apply() {
    if (typeof document !== "undefined") document.documentElement.dataset.theme = this.theme;
  }
}

export const appearance = new AppearanceStore();
