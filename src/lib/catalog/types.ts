// Mirrors src-tauri/src/catalog/mod.rs (serde camelCase).

/** What the launcher can do with a console: only a `playable` one can be installed. */
export type State = "planned" | "in-progress" | "playable" | "unsupported";

export type Maturity = "alpha" | "beta" | "stable";

export interface CatalogPlatform {
  platform: string;
  /** What the game covers on this console: information, never a choice. */
  regions: string[];
  state: State;
  /** Only for a playable console. */
  maturity: Maturity | null;
  /** The year the game came out on this console, when the catalog says. */
  releaseYear: number | null;
}

export interface CatalogGame {
  id: string;
  name: string;
  japanReleaseYear: number;
  repository: string;
  platforms: CatalogPlatform[];
}

export interface Catalog {
  /** Ordered by first Japanese release year. */
  games: CatalogGame[];
}

export interface Refresh {
  catalog: Catalog | null;
  updated: boolean;
  error: string | null;
}
