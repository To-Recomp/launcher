# Game catalog

The sidebar is built from `catalog.json` of [`To-Recomp/catalog`](https://github.com/To-Recomp/catalog)
(format 1), so games appear without a launcher release.

- **Startup:** the saved copy is shown at once; a conditional request (`If-None-Match` with the
  saved `ETag`) then downloads a newer one if there is one. Games are ordered by first Japanese
  release year, whatever the order of the file.
- **Validation:** the download is checked against `src-tauri/src/catalog/catalog-1.schema.json`
  (a copy of the catalog repository's `schema/catalog-1.json`; update it when that file changes)
  plus unique game ids and unique platform/region pairs per game. Unknown fields are ignored.
- **Failure:** offline, a server error, an oversized (over 1 MiB) or invalid catalog never
  replaces the saved copy. The sidebar keeps showing it and says why the update failed, with a
  Retry button. With no saved copy yet, it asks to retry.
- **Where:** `catalog/catalog.json` and `catalog/etag` in the app data folder.
- **Development:** `TO_RECOMP_CATALOG_URL` points the launcher at another catalog (a local
  server, a fork).

Each platform shows as a chip with a status dot: stable, beta, alpha, unsupported, coming-soon.
Installation state (download / settings / play icon) comes with #3; until then every game shows
the download icon.
