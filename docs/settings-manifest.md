# Settings from the manifest

The launcher builds a game's Settings screens from the `config` and `settings` parts of the
`recomplauncher.json` in its release. The format is [RecompLauncher's](https://github.com/elripalda/RecompLauncher/blob/main/docs/DEVELOPERS.md#settings-in-the-launcher),
so one manifest serves both launchers. Fields the launcher doesn't know are ignored.

## Supported

| Manifest | Control |
| --- | --- |
| `choices` with labels Off/On (or No/Yes, Disabled/Enabled, False/True) | toggle |
| `choices`, at most 3, short labels | segmented buttons |
| `choices`, otherwise | dropdown |
| `slider` (`format`: `%g`, `%d`, `%.Nf`) | slider |
| `bindings` (`separator`, `max`, `names`, `pad`) | key chips |
| `text` (launcher extension) | text field |

Also: `group` (heading), `advanced` and `groups[].advanced` ("Show advanced options", remembered
per game), `when` (greyed out unless the keys match), `in: "args"` (the pick is remembered in
`<data>/launcher-settings.json` and added to the command line by `launch_args`), `defaults`.
Config files: `toml` (edited in place, comments and unknown keys kept) and `json`; `file` may be
a path or a list of `{path, if}`. Placeholders: `{install}`, `{data}`, `{home}`, `{documents}`,
`{local}`, `{config}`. `ini` is rejected with a message for now.

## Launcher extensions (ignored by RecompLauncher)

- `tab`: sub-tab of the setting, one of `Display`, `Graphics`, `Audio`, `Controls`, `Saves`,
  `Game` (default `Game`). `group` is the heading inside the tab. The launcher adds one tab of its own that manifests don't fill, `About`, and a **Files** section at the top of `Game` (each file the game was prepared from, with a "Change" button that picks another file for that input only and rebuilds; shown even before the game is prepared, and not at all for a game whose manifest asks for none).
- `control`: `"toggle"` (exactly two choices), `"segmented"` or `"dropdown"`, to override the
  automatic choice above.
- `text`: `{ "key": "player/name" }`, a free-text value of at most 256 characters.

## Validation

A manifest with duplicate ids, several control kinds in one setting, an undeclared config, an
unknown tab, a slider with `min >= max`, an unsupported number format, and so on is rejected as
a whole with a message shown in place of the settings. A rejected change (value out of range,
unknown key name, too many keys, a setting whose `when` doesn't hold) is not written and shows
its message above the settings.

## Wiring

Commands (`src-tauri/src/settings`): `load_settings`, `set_setting`, `settings_launch_args`,
each taking `{ installDir, dataDir? }`. The library (`docs/library.md`) supplies that location
for the selected game and platform (`instance_location`), and adds the `args` settings to the
command line when it starts the game.
