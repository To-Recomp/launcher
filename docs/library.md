# Installations and instances

- **Installation**: one per game, a release of the game's repository (`owner/name` of the
  catalog entry). The launcher asks GitHub for the newest release (a repository with only
  pre-releases gets its newest one), takes the `.zip` or `.tar.gz` made for the OS (x86-64
  preferred), **checks it against the SHA-256 the API publishes** (a release without one is
  refused), unpacks it (paths leaving the folder and links are refused) and keeps it only if its
  `recomplauncher.json` is valid and has a `start` command. Installing the same tag again
  downloads nothing.
- **Instance**: one per platform of an installed game, with its own data folder
  (`{data}`: settings, saves, the prepared game), the player's files and a log.
- **Prepare**: the release's `prepare` command, run once with the player's files (`inputs` of the
  manifest, asked with a file or folder dialog). The launcher passes `{input:<id>}`, `{data}`,
  `RECOMPLAUNCHER*` variables and a scratch `TMPDIR`, shows the step's `progress 0.4 text` lines (and, when it fails, the text of its
  `result failed <code> <text>` line in the error),
  logs everything to `prepare.log`, needs 1 GB free, and can cancel (the program and what it
  started are killed). Afterwards **the game starts without the files**, unless its `start`
  arguments name an input.
  A step that works in several tasks can report them (a launcher extension, ignored by
  RecompLauncher): `stage-start <id> <i>/<n> <title>`, `stage-progress <id> <done>/<total> <text>`
  and `stage-end <id> ok|skipped|failed|cancelled <seconds>`. The Home page then lists each task
  with its position, its own percentage and bar while it runs, and its duration once done, under
  the overall bar (which still follows `progress`). `tos-prepare` prints these lines.
- **Update detection:** each installed game asks for the release an install would fetch now
  (`latest_release_tag`, read from the same cached listing as the home page's notes) when the
  library changes or the game is opened, but at most every 15 minutes (a minute after a failure).
  When it isn't the installed tag the game's page shows an **Update** button next to the main one
  and says which release is available; the main button keeps playing the installed one meanwhile.
- **Update**: installing a newer release replaces the release folder only. Instances stay, and
  show `needs rebuild`. With `prepare.keeps_copy` the rebuild starts from what the last run kept
  in the instance, not from the dump (the launcher leaves out arguments naming the copy);
  otherwise the earlier files are reused if they still exist, and asked for if not.
- **Play**: `start` with the manifest's arguments, then the player's `args` settings; the output
  goes to `game.log`.
- **Delete the build**: the trash button next to Play (a prepared instance, nothing running)
  asks for confirmation, then the `delete_build` command removes the instance's record, logs and
  data, so the game can be prepared again from the player's files. The paths the manifest lists in
  `saves` (`"saves": ["{data}/saves"]`) are kept and found again by the next preparation. The
  installed release is untouched.
- **Releases on the home page**: "Latest updates" lists the three newest published releases of
  the selected game's repository (drafts left out), read from `GET /repos/<repo>/releases` by
  the `game_releases` command, independently of the installation. A game whose consoles are all
  `planned` has no repository yet, so GitHub isn't asked at all (`hasRepository`); the page
  keeps what it read for 15 minutes, so coming back to a game shows it at once. Each card shows the tag, the
  date, the release name and up to four entries of its notes, one line per list item and without
  headings, links or what GitHub generates ("What's Changed", "by @user in <pull request>", "Full
  Changelog"), then "+ N more" for the rest. A repository without a release
  (or one that doesn't exist) shows an empty message; a network or rate-limit error shows that
  the releases can't be read.
- **GitHub answer cache**: GitHub allows 60 unauthenticated API requests an hour, so every GET
  to the API is kept in `<app data>/github/<sha256 of the URL>.json` (body, `ETag`, fetch time).
  A repository costs **one** request: the notes, an install and an update check all read
  `releases?per_page=20` (the newest stable release is picked from it, a pre-release only when
  there is no stable one). Within 15 minutes the saved body is served with no request. After that
  a conditional request (`If-None-Match`) is made: a `304` (free of quota) renews the entry and a
  `200` replaces it. A `404` (a repository that doesn't exist yet) is kept too, for an hour, so it
  isn't asked about at every visit. When GitHub can't be asked (rate limit, network, other
  errors) the saved answer is served even if old; with nothing saved the error shows. Installing
  skips the wait (the newest release, its asset URL and checksum are always asked for, with the
  `ETag`, and a repository once missing is looked up again); `game_releases` accepts
  `refresh: true` to do the same for the notes.

States: `not prepared`, `ready`, `needs rebuild` (prepared for another release, or a file listed in
`done` is gone). The sidebar shows a quick play button for a ready game and nothing for the others.

```
<app data>/games/<game id>/installation.json
                          release/                 {install}
                          instances/<id>/instance.json, data/, prepare.log, game.log
```
`<id>` is the platform lowercased (`gamecube`, `xbox-series-x-s`).

Development: `TO_RECOMP_LIBRARY` moves the library, `TO_RECOMP_GITHUB_API` points at another
GitHub API (a fake release server). Not done: the manifest's `check` (the game's own `prepare`
verifies the dumps), `dumps`, `prepare.options`, `folders`/`actions`, a stuck-step timeout.
