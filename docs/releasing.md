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

## Cutting a release

1. Set the version in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` (then
   `cargo update -p to-recomp-launcher` for `Cargo.lock`) and `Version:` of
   `packaging/rpm/to-recomp-launcher.spec` (numeric part only, plus a `%changelog` entry).
   A Rust test fails if they disagree.
2. Merge, then tag the merge commit: `git tag vX.Y.Z && git push origin vX.Y.Z`. The workflow
   fails if the tag differs from the version in `tauri.conf.json`.
3. Check the release page. A pre-release never reaches users of the updater, which follows
   `releases/latest`.

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
