# Diagnostic file

A local Markdown file the player asks for, reviews, and pastes into a public GitHub issue.
**Nothing is sent over the network**, and nothing is written unless the player clicks (or the
launcher panics).

## When

- Game page, *Action status* panel, after a failed install, a failed `prepare`, or a game that exits
  with a non-zero code: **Create diagnostic file**. The issues opened are the game's repository's.
- Settings → About → **Report a bug**: context `manual`, no game, `To-Recomp/launcher`.
- Launcher panic: the panic hook writes the file at once (message and location only, no backtrace,
  first panic of a run only). The next start shows a banner with **View file** / **Copy**.

The preview says *Review this file before you publish it: it will be public.* **Copy** puts exactly
the file's content on the clipboard; **Open folder** reveals it; **Open issues** opens a new issue
of the repository.

## File

`<game>-diagnostic-<date>.md` (`launcher-diagnostic-<date>.md` for manual and panic), English,
at most 48 KiB, in the launcher's log folder (`app_log_dir`). The last 10 are kept. It holds
`Diagnostic format: 1`, a table (launcher version and build, OS family/name/arch, GPU names, game
id/release/platform/region/state, context), the error message, and the tail (at most 200 lines) of
`prepare.log` or `game.log` in a `<details>` block. An install failure has no log.

Not collected: user and host names, paths outside the placeholders, IP address, hardware serials,
game files or their names, environment variables, the catalog, other games, timestamps in the
content, screenshots, crash dumps.

## Scrubbing

`diagnostics::scrub` (pure, table-tested), in this order: known paths longest first (`{library}`,
`{install}`, `{data}`, `{input:<id>}`, `{app}`); home and user areas (`~`, removable media, UNC,
WSL); the user and host names (whole words; inside longer words only from 6 characters); e-mails,
IPs, MACs, URL credentials and query strings, tokens and long hex/base64 runs; control characters
and size caps.

Known limits: a *foreign* user name with spaces keeps its tail, a volume label with spaces keeps its
tail, a user named `dev` leaves `devbox` alone, and a dotted quad such as a four-part driver version
is taken for an IP.

## Backend commands

`build_report`, `pending_panic`, `dismiss_panic` (`src-tauri/src/diagnostics/`). The opener plugin
may only open `https://github.com/*/*/issues[/new]`.
