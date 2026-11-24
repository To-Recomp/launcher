# Game catalog

The sidebar is built from `catalog.json` of [`To-Recomp/catalog`](https://github.com/To-Recomp/catalog)
(format 1), so games appear without a launcher release.

- **Startup:** the saved copy is shown at once; a conditional request (`If-None-Match` with the
  saved `ETag`) then downloads a newer one if there is one. Games are ordered by first Japanese
  release year, whatever the order of the file.
- **Validation:** the download is checked against `src-tauri/src/catalog/catalog-1.schema.json`
  (a copy of the catalog repository's `schema/catalog-1.json`; update it when that file changes)
  plus unique game ids and unique platforms per game. Unknown fields are ignored.
- **Failure:** offline, a server error, an oversized (over 1 MiB) or invalid catalog never
  replaces the saved copy. The sidebar keeps showing it and says why the update failed, with a
  Retry button. With no saved copy yet, it asks to retry.
- **Where:** `catalog/catalog.json` and `catalog/etag` in the app data folder.
- **Development:** `TO_RECOMP_CATALOG_URL` points the launcher at another catalog (a local
  server, a fork).

Each game lists **consoles** (`platforms[]`): `regions` (what the game covers there, information
only), a `state` (`planned`, `in-progress`, `playable`, `unsupported`) and, only when playable, a
`maturity` (`alpha`, `beta`, `stable`) and, optionally, `release_year` (the year the game came out
on that console; the game's `japan_release_year` is used when absent). The sidebar shows each
game's name and year, plus a play icon when the game can be played at once; the platform selector
lists consoles, not regions, each prefixed with its year and ordered from the most recent to the
oldest, and opens on the console the game was last played on (remembered per game in `localStorage`), else the first playable one, else the most recent. It is as wide as the widest console of the whole catalog, so it never resizes. A
console that isn't `playable` can't be installed or prepared: the game page says why.
