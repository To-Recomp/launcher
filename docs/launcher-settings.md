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
  `launcher.json`. Settings the engine has no control for (folder picker, action button) get a
  dedicated component with the ticket that needs it.
