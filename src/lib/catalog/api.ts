import { invoke } from "@tauri-apps/api/core";
import type { Catalog, Refresh } from "./types";

/** The saved catalog, without touching the network. */
export const cachedCatalog = () => invoke<Catalog | null>("cached_catalog");

/** Conditional download; never rejects for network or validation problems (see `error`). */
export const refreshCatalog = () => invoke<Refresh>("refresh_catalog");
