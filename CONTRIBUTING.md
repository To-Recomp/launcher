# Contributing

Thanks for helping. Short rules:

- **Language**: English for code, commits, issues and documentation.
- **License**: this project is GPL v3 (see `LICENSE`). By contributing you agree your work is released under it.
- **Sign-off (DCO)**: every commit must carry a `Signed-off-by` line certifying the [Developer Certificate of Origin](DCO). Use `git commit -s`, or enable the repository's hook once per clone with `git config core.hooksPath .githooks`: it adds the line (your `user.name` and `user.email`) to every commit message, so it can't be forgotten. A CI check rejects pull requests with unsigned commits.
- **Pull request titles**: pull requests are squash-merged and the title becomes the commit on `main`, so it must be a [Conventional Commit](https://www.conventionalcommits.org): `feat: …` (next minor version), `fix: …` (patch), `perf:`, `docs:`, `chore:`, `ci:`, `build:`, `test:`, `refactor:`, `style:`, `revert:`; add `!` (`feat!: …`) or a `BREAKING CHANGE:` footer for a breaking change. Version numbers are computed from these titles, never edited by hand (`docs/releasing.md`). A CI check rejects other titles.
- **No game data, ever**: no ISOs, ROMs, extracted files, textures, audio, or code translated from a game binary. The `.gitignore` blocks common formats, but you are responsible for what you commit.
- **No code from incompatible licenses**: do not copy code under non-commercial or otherwise GPL v3-incompatible terms. Do not read or derive from PolyForm Noncommercial projects (for example snesrecomp) when working on the SNES recompiler.
- **Behavior**: see `CODE_OF_CONDUCT.md`.
