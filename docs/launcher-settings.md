# Launcher settings

The launcher's own settings page is opened from the gear in the title bar. It doesn't depend on
the selected game: the sidebar shows no selection and the game's colors are not applied. Picking
a game in the sidebar goes back to Home.

- **Engine:** the same one as a game's Settings (`docs/settings-manifest.md`), fed with a
  manifest built into the launcher (`src-tauri/src/launcher_settings/manifest.json`) instead of
  one read from a release. Controls, `group` headings, `when`, validation and messages are the
  same. `tab` is ignored (one page, sections by `group`), and `in: "args"` isn't used.
- **Where:** values live in `launcher.json` in the app's config folder (`config` of the
  manifest, `{data}` is that folder).
- **Commands:** `load_launcher_settings` and `set_launcher_setting` (`{ id, input }`), returning
  the refreshed view like `load_settings` and `set_setting`.
- **Adding a setting:** add it to the built-in manifest; whatever it controls reads
  `launcher.json`. What the engine has no control for uses the dedicated rows, placed in
  `LauncherSettingsView` next to the manifest's sections:
  - `FolderRow`: shows a path (or "Default") and a "Choose…" button opening the folder dialog.
  - `ActionRow`: a button that runs an async action, disabled while it runs, with its result
    or failure next to it.
  - `Row`: the shared layout (label, help, control) that `SettingRow` also uses.
- **Restart to apply:** a setting that only applies on the next start calls
  `restartNeeded.mark()` (`src/lib/launcher/restart.svelte.ts`); the page then shows a banner
  with "Restart now" (`restart_launcher` command).

## About

- **Launcher** (bottom of the launcher page): version, license, repository, "Report a bug" for
  the launcher's own issues, credits of what it is built with (`CREDITS` in `src/lib/data.ts`) and
  the non-affiliation notice.
- **Game** (Settings → About of the selected game): name, console, installed release, repository,
  license (read from GitHub by `game_license`; "Not published" when it has none or GitHub can't be
  reached), install and data folders with an Open button, and "Report a bug" for the game's own
  repository (the diagnostic file is about that instance). Works for a game that isn't installed
  yet: it then shows no folders.
- Links open github.com pages only (`opener:allow-open-url` scope in `capabilities/default.json`),
  plus steamgriddb.com for the artwork credit.

## Storage

- **Games folder** (`library.folder` in `launcher.json`): where installations and each
  instance's data live; the default is `games/` in the app data folder. Chosen with a folder
  dialog and checked (absolute, existing, writable) before it is saved; "Use default" removes it.
- It is read once at startup (`library::LibraryRoot`), so a change applies after a restart (the
  banner offers it). Games already installed are **not** moved: after a change they are no
  longer listed, and moving their folder by hand puts them back. If the chosen folder can't be
  used at startup (a drive that is gone), the library says why instead of falling back to the
  default, so games never look uninstalled by accident.
- The launcher's own data (catalog copy, logs, settings) stays in the usual app folders.
- `TO_RECOMP_LIBRARY` still overrides everything, for development and tests.

## Updates

Only where the launcher can replace itself (an AppImage, or a Windows installation made by the
installer: `updater_available`); elsewhere the settings below are hidden, since a package or ZIP
is updated another way.

- **Check for updates** (`updates.auto`, default on): when the launcher starts it looks for a newer
  release and the sidebar button then reads "Update to vX". Nothing is installed without a click.
  A failed startup check is silent; "Check now" in the settings reports why it failed.
- **Releases** (`updates.channel`): Stable (GitHub's latest release, the endpoint built into the
  app) or Pre-release (the newest release that has a `latest.json`, found through the GitHub
  releases listing). Changing it discards what the other channel had found.
- **Launcher version** row: "Check now" and, when something newer exists, "Install and restart".
  The sidebar button keeps its own check-and-install click.
- Commands: `check_launcher_update` (no install), `update_launcher`, `launcher_update_preferences`.

## Game list

"Game catalog" shows how the last refresh of the game list went (updated, already up to date,
or why it failed, in which case the saved list is used) and has a Refresh button, which is the
same refresh as the sidebar's Retry. The time is kept for the session only. The catalog source is
not a setting: the launcher only reads the To-Recomp catalog (`TO_RECOMP_CATALOG_URL` stays a
development switch, see `docs/catalog.md`).

## Behavior

- **Start with the system** (Linux and Windows; hidden elsewhere): registers or removes the
  launcher through `tauri-plugin-autostart` (an `.desktop` entry in `~/.config/autostart`, or the
  `Run` key of the registry; an AppImage registers its own path). The switch shows what the
  system says, not a saved value, so an entry removed by hand shows as off.
- **Keep running in the notification area** (`behavior.tray`): closing the window hides it and a
  tray icon (Show / Quit) brings it back; turning it off shows the window again. It needs a
  notification area: GNOME has none without an AppIndicator extension, so the window can look
  lost there; it is off by default. On Linux the icon uses `libayatana-appindicator` (loaded at
  run time; the Fedora package requires it, the AppImage bundles it from the build machine).
- **Close the launcher when a game starts** (`behavior.closeOnLaunch`): `start_game` quits the
  launcher once the game has been spawned. The game keeps running; its exit code and log summary
  are no longer reported by the launcher.
- Applied at once (the tray is synced at startup and on change, with its menu in the interface
  language). A failure to add the icon is shown above the settings.
