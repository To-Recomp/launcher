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
  `RECOMPLAUNCHER*` variables and a scratch `TMPDIR`, shows the step's `progress 0.4 text` lines,
  logs everything to `prepare.log`, needs 1 GB free, and can cancel (the program and what it
  started are killed). Afterwards **the game starts without the files**, unless its `start`
  arguments name an input.
- **Update**: installing a newer release replaces the release folder only. Instances stay, and
  show `needs rebuild`. With `prepare.keeps_copy` the rebuild starts from what the last run kept
  in the instance, not from the dump (the launcher leaves out arguments naming the copy);
  otherwise the earlier files are reused if they still exist, and asked for if not.
- **Play**: `start` with the manifest's arguments, then the player's `args` settings; the output
  goes to `game.log`.

States: `not prepared`, `ready`, `needs rebuild` (prepared for another release, or a file listed in
`done` is gone). The sidebar shows download, settings or play icons for not installed,
installed, ready.

```
<app data>/games/<game id>/installation.json
                          release/                 {install}
                          instances/<id>/instance.json, data/, prepare.log, game.log
```
`<id>` is the platform lowercased (`gamecube`, `xbox-series-x-s`).

Development: `TO_RECOMP_LIBRARY` moves the library, `TO_RECOMP_GITHUB_API` points at another
GitHub API (a fake release server). Not done: the manifest's `check` (the game's own `prepare`
verifies the dumps), `dumps`, `prepare.options`, `folders`/`actions`, a stuck-step timeout.
