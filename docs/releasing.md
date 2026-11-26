# Releasing

## What a release contains

Pushing a tag `vX.Y.Z` (or `vX.Y.Z-rc.1`, published as a pre-release) runs
`.github/workflows/release.yml`, which publishes:

| Asset | For |
| --- | --- |
| `To-Recomp-Launcher_<v>_windows-x86_64-portable.zip` | Windows, unzip and run (unsigned; needs the WebView2 runtime, present on Windows 11) |
| `To-Recomp-Launcher_<v>_windows-x86_64-setup.exe` | Windows installer, per user; installs that can update themselves |
| `To-Recomp-Launcher_<v>_linux-x86_64.AppImage` | Linux, self-updating |
| `To-Recomp-Launcher_<v>_linux-x86_64.tar.gz` | Linux binary, `.desktop` file and icon; needs `webkit2gtk4.1`, `gtk3`, `libsoup3` |
| `SHA256SUMS` | checksums of every asset above |
| `latest.json` | what the in-app updater reads (only when the build is signed) |

The release description is built by `scripts/release-notes.mjs` from the commits since the previous
release (every commit for the first one; a stable release is compared with the previous
stable one, so its notes also cover its release candidates), grouped by conventional-commit type, with the
pull request numbers and a link to the comparison. Preview it locally from a checkout with the
tags: `node scripts/release-notes.mjs --tag vX.Y.Z --repo To-Recomp/launcher` (the tag must exist;
use a temporary local tag to preview).

Binaries are unsigned for the operating system (no Windows code signing certificate); the
checksums are the way to verify a download: `sha256sum -c SHA256SUMS --ignore-missing`.

Fedora users install from COPR (below). The in-app "Update launcher" button appears only where
the updater can replace the running program: the AppImage and the Windows installer. Portable
ZIP, tarball and RPM installs are updated by downloading or with `dnf`.

A pull request that touches the packaging runs the same builds without publishing, and the
Linux/Windows artifacts can be downloaded from the run.

## Version numbers and cutting a release

Nobody edits a version by hand. [release-please](https://github.com/googleapis/release-please)
(`.github/workflows/release-please.yml`, configured by `release-please-config.json`,
`release-please-config.rc.json` and `.release-please-manifest.json`) reads the conventional commits
merged into `main` (the squashed pull request titles, see `CONTRIBUTING.md`) and keeps one pull
request titled `chore(main): release X.Y.Z` open:

| Commits since the last release | Next version (from 0.1.0) |
| --- | --- |
| `fix:`, `perf:` | 0.1.1 |
| `feat:` | 0.2.0 |
| `feat!:` / `BREAKING CHANGE:` (below 1.0, `bump-minor-pre-major`) | 0.2.0 |
| `docs:`, `chore:`, `ci:`, `test:`… only | no release PR |

The pull request rewrites the version in `package.json`, `package-lock.json`,
`src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` and the
`%global upstream_version` line of the RPM spec (between its `x-release-please` markers; the spec
derives `Version` and `Release` from it: `0.2.0-rc.1` is packaged as Version `0.2.0`, Release
`0.rc.1`), and adds the entries to `CHANGELOG.md`. The `%changelog` of the spec follows the
version by itself.

**Releasing**: review that pull request, merge it (squash; add a `Signed-off-by` line to the commit
message, the bot can't). Release-please then tags the merge commit `vX.Y.Z` (a draft release holds
the tag) and calls `release.yml`, which builds, then fills and publishes the release. The workflow
fails if the tag differs from the version in `tauri.conf.json`. Check the release page: a
pre-release never reaches users of the updater, which follows `releases/latest`. Until the pull
request is merged, no tag exists and nothing is published.

**Release candidates**: set the repository variable `RELEASE_CHANNEL` to `rc`
(`gh variable set RELEASE_CHANNEL --body rc --repo To-Recomp/launcher`) and run the *Release PR*
workflow: the pull request now proposes `0.2.0-rc.1`; each further merge gives `rc.2`, `rc.3`… The
tag is published as a pre-release. To graduate, remove the variable (or set it to `stable`) and
run the workflow again: the pending pull request becomes `0.2.0`, and the notes of a stable release
cover its candidates. To force an exact version (a first `1.0.0`), put `Release-As: 1.0.0` in the
body of a commit.

**Limits**: the pull request is opened with the default `GITHUB_TOKEN`, so GitHub doesn't run `ci`
or `dco` on it (the `dco` check wouldn't pass on the bot's commits anyway); the checks already ran
on every change merged before it. The `rc` and `stable` configs differ only by `prerelease`
(`scripts/release-please-config.test.ts` checks it). The Cargo.lock path uses `@.name.value`
because release-please's TOML values are wrapped objects (checked against release-please 17.6.0,
the version bundled in `release-please-action` v5).

Pushing a `vX.Y.Z` tag by hand still builds and publishes a release (the flow before
release-please), but then versions are yours to keep in step: a Rust test fails if the files
disagree.

## Updater signing keys

The updater only installs updates signed with the private key whose public half is built into
the launcher. One-time setup, done by the maintainer:

```
npm run tauri signer generate -- -w ~/.tauri/to-recomp-launcher.key
```

Then, in the GitHub repository settings:

- secret `TAURI_SIGNING_PRIVATE_KEY`: the content of `to-recomp-launcher.key`;
- secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the password chosen above;
- variable `TAURI_UPDATER_PUBKEY`: the content of `to-recomp-launcher.key.pub` (public).

**Keep a backup of the private key and its password** outside GitHub (password manager plus an
offline copy). Without the secrets the workflow still publishes everything but builds without
self-update and without `latest.json` (it prints a warning).

**If the private key is lost** (or leaks), no automatic update is possible: installed launchers
trust only the old public key and reject anything signed with another. Generate a new pair,
publish a release signed with it, and tell users to reinstall once (download the new AppImage
or installer); from that release on, updates work again. For a leak, also delete the old
secret so it can't sign anything else.

## Fedora COPR

The package is built from the tagged source with `packaging/rpm/to-recomp-launcher.spec`.

Maintainer setup (needs the maintainer's COPR account, see the `ready-for-human` follow-up):

1. Create a COPR project `to-recomp-launcher` with the Fedora chroots to support.
2. In its settings, enable **internet access during builds** (the build runs `npm ci` and
   downloads crates).
3. Add a package, source type **SCM**: clone URL `https://github.com/To-Recomp/launcher.git`,
   committish the tag, build method `make srpm` (it uses `.copr/Makefile`). Add a webhook for
   tags to build automatically on release.

Users: `sudo dnf copr enable <account>/to-recomp-launcher && sudo dnf install to-recomp-launcher`.

Local check in a mock chroot (what the acceptance criterion asks for):

```
git commit ...                           # the source tarball is `git archive HEAD`
packaging/rpm/build-srpm.sh ~/rpmout
mock -r fedora-$(rpm -E %fedora)-x86_64 --enable-network ~/rpmout/*.src.rpm
rpmlint ~/rpmout/*.rpm /var/lib/mock/*/result/*.rpm
```
