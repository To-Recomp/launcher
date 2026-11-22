// Mirrors src-tauri/src/catalog/mod.rs (serde camelCase).

export type Status = "stable" | "beta" | "alpha" | "unsupported";

export interface CatalogPlatform {
  platform: string;
  region: string;
  status: Status;
  tested: boolean;
}

export interface CatalogGame {
  id: string;
  name: string;
  japanReleaseYear: number;
  repository: string;
  manifest: string | null;
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
