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
