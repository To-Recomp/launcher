use std::fs;

use tempfile::TempDir;

use super::*;

const MANIFEST: &str = r#"{
  "name": "unknown top-level fields are ignored",
  "game": { "whatever": true },
  "config": [
    { "id": "main", "format": "toml", "file": "{data}/game.toml" },
    { "id": "extra", "file": "{data}/extra.json" }
  ],
  "groups": [ { "id": "Tweaks", "advanced": true } ],
  "settings": [
    { "id": "vsync", "label": "VSync", "tab": "Display", "group": "Frame rate",
      "futureField": 1,
      "choices": [ { "label": "Off", "set": { "vsync": "false" } },
                   { "label": "On",  "set": { "vsync": "true" } } ],
      "defaults": { "vsync": "true" } },
    { "id": "window", "label": "Window", "tab": "Display",
      "choices": [ { "label": "Windowed",   "set": { "window/mode": "\"windowed\"" } },
                   { "label": "Borderless", "set": { "window/mode": "\"borderless\"" } },
                   { "label": "Fullscreen", "set": { "window/mode": "\"fullscreen\"" } } ],
      "defaults": { "window/mode": "\"windowed\"" } },
    { "id": "size", "label": "Window size", "tab": "Display",
      "when": { "window/mode": "windowed" },
      "choices": [ { "label": "1280 x 720",  "set": { "window/width": "1280", "window/height": "720" } },
                   { "label": "1600 x 900",  "set": { "window/width": "1600", "window/height": "900" } },
                   { "label": "1920 x 1080", "set": { "window/width": "1920", "window/height": "1080" } },
                   { "label": "2560 x 1440", "set": { "window/width": "2560", "window/height": "1440" } } ],
      "defaults": { "window/width": "1280", "window/height": "720" } },
    { "id": "volume", "label": "Volume", "tab": "Audio",
      "slider": { "key": "audio/volume", "min": 0, "max": 1, "step": 0.05, "format": "%.2f" },
      "defaults": { "audio/volume": "0.5" } },
    { "id": "name", "label": "Player name", "tab": "Game",
      "text": { "key": "player/name" } },
    { "id": "keys", "label": "Keyboard", "tab": "Controls",
      "bindings": { "separator": "|", "max": 2, "names": { "Return": "Enter" },
        "actions": [ { "label": "Cross", "key": "key.cross" },
                     { "label": "Pad", "key": "key.pad", "pad": true } ] },
      "defaults": { "key.cross": "Space", "key.pad": "Cross" } },
    { "id": "profile", "label": "Profile", "tab": "Game", "in": "extra",
      "choices": [ { "label": "Low", "set": { "profile": "\"low\"" } },
                   { "label": "High", "set": { "profile": "\"high\"" } } ] },
    { "id": "debug", "label": "Debug overlay", "group": "Tweaks", "in": "args",
      "choices": [ { "label": "Game's option", "args": [] },
                   { "label": "Show", "args": ["--debug-overlay"] } ] },
    { "id": "sens", "label": "Sensitivity", "tab": "Controls", "in": "args",
      "slider": { "min": 0.1, "max": 5, "step": 0.05, "format": "%.2f" },
      "args": ["--sens", "{value}"] }
  ]
}"#;

const GAME_TOML: &str = "# my settings\nvsync = true # keep on\nunknown = \"kept\"\n\n[window]\nmode = \"windowed\"\nwidth = 1600\nheight = 900\n\n[audio]\nvolume = 0.25\n";

fn instance(manifest: &str, game_toml: Option<&str>) -> (TempDir, Instance) {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("recomplauncher.json"), manifest).unwrap();
    let instance = Instance::new(dir.path().to_path_buf(), None);
    if let Some(toml) = game_toml {
        fs::create_dir_all(&instance.data).unwrap();
        fs::write(instance.data.join("game.toml"), toml).unwrap();
    }
    (dir, instance)
}

fn setting<'a>(view: &'a SettingsView, id: &str) -> &'a SettingView {
    view.settings.iter().find(|s| s.id == id).unwrap()
}

fn read_toml(instance: &Instance) -> String {
    fs::read_to_string(instance.data.join("game.toml")).unwrap()
}

#[test]
fn renders_each_control_type_from_the_files() {
    let (_dir, inst) = instance(MANIFEST, Some(GAME_TOML));
    let view = load_view(&inst).unwrap();

    assert_eq!(view.settings.len(), 9);
    assert_eq!(
        setting(&view, "vsync").control,
        Control::Choices {
            style: ChoiceStyle::Toggle,
            options: vec!["Off".into(), "On".into()],
            value: Some(1),
            custom: None
        }
    );
    assert!(matches!(
        &setting(&view, "window").control,
        Control::Choices {
            style: ChoiceStyle::Segmented,
            value: Some(0),
            ..
        }
    ));
    // Four choices: a dropdown. Two keys per choice: matched together.
    assert!(matches!(
        &setting(&view, "size").control,
        Control::Choices {
            style: ChoiceStyle::Dropdown,
            value: Some(1),
            ..
        }
    ));
    assert_eq!(
        setting(&view, "volume").control,
        Control::Slider {
            min: 0.0,
            max: 1.0,
            step: 0.05,
            value: 0.25,
            format: Some("%.2f".into())
        }
    );
    // Missing in the file and without a default.
    assert_eq!(
        setting(&view, "name").control,
        Control::Text {
            value: String::new()
        }
    );
    // Missing in the file: the default; "Enter" is the file's name for "Return".
    assert_eq!(
        setting(&view, "keys").control,
        Control::Bindings {
            max: 2,
            actions: vec![
                ActionView {
                    label: "Cross".into(),
                    key: "key.cross".into(),
                    keys: vec!["Space".into()],
                    default: vec!["Space".into()],
                    pad: false
                },
                ActionView {
                    label: "Pad".into(),
                    key: "key.pad".into(),
                    keys: vec!["Cross".into()],
                    default: vec!["Cross".into()],
                    pad: true
                },
            ]
        }
    );
    assert_eq!(setting(&view, "vsync").tab, "Display");
    assert_eq!(setting(&view, "profile").tab, "Game");
    assert_eq!(setting(&view, "vsync").group.as_deref(), Some("Frame rate"));
}

#[test]
fn a_value_that_matches_no_choice_is_shown_as_it_is() {
    let (_dir, inst) = instance(MANIFEST, Some("[window]\nmode = \"exotic\"\n"));
    let view = load_view(&inst).unwrap();
    assert_eq!(
        setting(&view, "window").control,
        Control::Choices {
            style: ChoiceStyle::Segmented,
            options: vec!["Windowed".into(), "Borderless".into(), "Fullscreen".into()],
            value: None,
            custom: Some("exotic".into())
        }
    );
}

#[test]
fn when_greys_a_setting_out_and_blocks_writing_it() {
    let (_dir, inst) = instance(MANIFEST, Some("[window]\nmode = \"fullscreen\"\n"));
    assert!(!setting(&load_view(&inst).unwrap(), "size").enabled);
    let err = apply(&inst, "size", SettingInput::Choice { index: 0 }).unwrap_err();
    assert!(err.contains("isn't available"), "{err}");
    assert!(!read_toml(&inst).contains("width"));
}

#[test]
fn advanced_comes_from_the_setting_or_its_group() {
    let (_dir, inst) = instance(MANIFEST, None);
    let view = load_view(&inst).unwrap();
    assert!(setting(&view, "debug").advanced);
    assert!(!setting(&view, "vsync").advanced);
}

#[test]
fn writing_keeps_comments_order_and_unknown_keys() {
    let (_dir, inst) = instance(MANIFEST, Some(GAME_TOML));
    apply(&inst, "vsync", SettingInput::Choice { index: 0 }).unwrap();
    apply(&inst, "window", SettingInput::Choice { index: 2 }).unwrap();
    apply(&inst, "volume", SettingInput::Number { value: 0.7 }).unwrap();
    let written = read_toml(&inst);
    assert_eq!(
        written,
        "# my settings\nvsync = false # keep on\nunknown = \"kept\"\n\n[window]\nmode = \"fullscreen\"\nwidth = 1600\nheight = 900\n\n[audio]\nvolume = 0.70\n"
    );
}

#[test]
fn creates_the_file_and_nested_tables_when_missing() {
    let (_dir, inst) = instance(MANIFEST, None);
    let view = apply(&inst, "size", SettingInput::Choice { index: 2 }).unwrap();
    assert!(matches!(
        &setting(&view, "size").control,
        Control::Choices { value: Some(2), .. }
    ));
    let written = read_toml(&inst);
    assert!(
        written.contains("width = 1920") && written.contains("height = 1080"),
        "{written}"
    );
}

#[test]
fn writes_text_and_key_bindings_with_the_files_own_names() {
    let (_dir, inst) = instance(MANIFEST, Some(GAME_TOML));
    apply(
        &inst,
        "name",
        SettingInput::Text {
            value: "Yuri \"the\" Lowell".into(),
        },
    )
    .unwrap();
    let view = apply(
        &inst,
        "keys",
        SettingInput::Bindings {
            action: "key.cross".into(),
            keys: vec!["Return".into(), "Mouse Left".into()],
        },
    )
    .unwrap();
    let written = read_toml(&inst);
    assert!(
        written.contains("\"key.cross\" = \"Enter|Mouse Left\""),
        "{written}"
    );
    assert!(
        matches!(&setting(&view, "name").control, Control::Text { value } if value == "Yuri \"the\" Lowell")
    );
    let Control::Bindings { actions, .. } = &setting(&view, "keys").control else {
        panic!()
    };
    assert_eq!(actions[0].keys, vec!["Return", "Mouse Left"]);
    // The other action is untouched: only what the player changes is written.
    assert!(
        !written.contains("key.pad") && !written.contains("pad"),
        "{written}"
    );
}

#[test]
fn json_configs_are_edited_too() {
    let (_dir, inst) = instance(MANIFEST, None);
    fs::create_dir_all(&inst.data).unwrap();
    fs::write(
        inst.data.join("extra.json"),
        "{\"zeta\": 1, \"profile\": \"low\"}",
    )
    .unwrap();
    let view = apply(&inst, "profile", SettingInput::Choice { index: 1 }).unwrap();
    assert!(matches!(
        &setting(&view, "profile").control,
        Control::Choices { value: Some(1), .. }
    ));
    let written = fs::read_to_string(inst.data.join("extra.json")).unwrap();
    assert_eq!(written, "{\n  \"zeta\": 1,\n  \"profile\": \"high\"\n}\n");
}

#[test]
fn rejects_invalid_values_without_touching_the_file() {
    let (_dir, inst) = instance(MANIFEST, Some(GAME_TOML));
    let cases: Vec<(&str, SettingInput, &str)> = vec![
        (
            "volume",
            SettingInput::Number { value: 1.5 },
            "between 0 and 1",
        ),
        (
            "volume",
            SettingInput::Number { value: f64::NAN },
            "between",
        ),
        ("vsync", SettingInput::Choice { index: 9 }, "doesn't exist"),
        (
            "vsync",
            SettingInput::Number { value: 1.0 },
            "isn't a slider",
        ),
        ("nope", SettingInput::Choice { index: 0 }, "Unknown setting"),
        (
            "name",
            SettingInput::Text {
                value: "a\nb".into(),
            },
            "plain text",
        ),
        (
            "keys",
            SettingInput::Bindings {
                action: "key.cross".into(),
                keys: vec!["Banana".into()],
            },
            "isn't a key",
        ),
        (
            "keys",
            SettingInput::Bindings {
                action: "key.cross".into(),
                keys: vec!["A".into(), "A".into()],
            },
            "twice",
        ),
        (
            "keys",
            SettingInput::Bindings {
                action: "key.cross".into(),
                keys: vec!["A".into(), "B".into(), "C".into()],
            },
            "at most 2",
        ),
        (
            "keys",
            SettingInput::Bindings {
                action: "key.cross".into(),
                keys: vec!["Cross".into()],
            },
            "isn't a key",
        ),
        (
            "keys",
            SettingInput::Bindings {
                action: "key.zzz".into(),
                keys: vec![],
            },
            "unknown action",
        ),
    ];
    for (id, input, expected) in cases {
        let err = apply(&inst, id, input).unwrap_err();
        assert!(err.contains(expected), "{id}: {err}");
    }
    assert_eq!(read_toml(&inst), GAME_TOML);
}

#[test]
fn a_pad_action_accepts_controller_buttons() {
    let (_dir, inst) = instance(MANIFEST, None);
    apply(
        &inst,
        "keys",
        SettingInput::Bindings {
            action: "key.pad".into(),
            keys: vec!["Circle".into(), "F5".into()],
        },
    )
    .unwrap();
}

#[test]
fn args_settings_are_remembered_and_become_arguments() {
    let (_dir, inst) = instance(MANIFEST, None);
    // Nothing chosen yet: the first choice (empty), and no slider argument.
    assert_eq!(launch_args(&inst).unwrap(), Vec::<String>::new());

    apply(&inst, "debug", SettingInput::Choice { index: 1 }).unwrap();
    assert_eq!(launch_args(&inst).unwrap(), vec!["--debug-overlay"]);

    let view = apply(&inst, "sens", SettingInput::Number { value: 1.5 }).unwrap();
    assert!(
        matches!(&setting(&view, "sens").control, Control::Slider { value, .. } if *value == 1.5)
    );
    assert_eq!(
        launch_args(&inst).unwrap(),
        vec!["--debug-overlay", "--sens", "1.50"]
    );
    assert!(!inst.data.join("game.toml").exists());

    apply(&inst, "debug", SettingInput::Choice { index: 0 }).unwrap();
    assert_eq!(launch_args(&inst).unwrap(), vec!["--sens", "1.50"]);
}

#[test]
fn a_release_without_a_manifest_or_with_a_broken_one_reports_why() {
    let dir = TempDir::new().unwrap();
    let inst = Instance::new(dir.path().to_path_buf(), None);
    assert!(load_view(&inst)
        .unwrap_err()
        .contains("no recomplauncher.json"));

    for (manifest, expected) in [
        ("not json", "Invalid recomplauncher.json"),
        (r#"{"settings":[{"id":"a","label":"A"}]}"#, "exactly one of"),
        (
            r#"{"config":[{"file":"{data}/a.toml"}],"settings":[{"id":"a","label":"A","text":{"key":"k"}},{"id":"a","label":"B","text":{"key":"k"}}]}"#,
            "twice",
        ),
        (
            r#"{"settings":[{"id":"a","label":"A","in":"nowhere","text":{"key":"k"}}]}"#,
            "isn't declared",
        ),
        (r#"{"config":[{"format":"ini","file":"x.ini"}]}"#, "ini"),
        (
            r#"{"settings":[{"id":"a","label":"A","in":"args","slider":{"min":2,"max":1,"step":1},"args":["{value}"]}]}"#,
            "min < max",
        ),
        (
            r#"{"config":[{"file":"{data}/a.toml"}],"settings":[{"id":"a","label":"A","tab":"Nope","text":{"key":"k"}}]}"#,
            "unknown tab",
        ),
        (
            r#"{"config":[{"file":"{data}/a.toml"}],"settings":[{"id":"a","label":"A","defaults":{"k":[1]},"text":{"key":"k"}}]}"#,
            "must be a string",
        ),
        (
            r#"{"config":[{"file":"{data}/a.toml"}],"settings":[{"id":"a","label":"A","control":"toggle","choices":[{"label":"x","set":{"k":"1"}}]}]}"#,
            "exactly two",
        ),
    ] {
        fs::write(inst.manifest_path(), manifest).unwrap();
        let err = load_view(&inst).unwrap_err();
        assert!(err.contains(expected), "{manifest}: {err}");
    }
}

#[test]
fn unsupported_placeholders_are_reported_not_written() {
    let manifest = r#"{"config":[{"file":"{input:game}/a.toml"}],"settings":[{"id":"a","label":"A","text":{"key":"k"}}]}"#;
    let (_dir, inst) = instance(manifest, None);
    assert!(load_view(&inst).unwrap_err().contains("{input:game}"));
}

#[test]
fn config_candidates_pick_the_first_whose_condition_exists() {
    let manifest = r#"{"config":[{"format":"toml","file":[
        {"path":"{install}/portable.toml","if":"{install}/portable.txt"},
        {"path":"{data}/game.toml"}]}],
      "settings":[{"id":"a","label":"A","text":{"key":"k"}}]}"#;
    let (dir, inst) = instance(manifest, None);
    apply(&inst, "a", SettingInput::Text { value: "x".into() }).unwrap();
    assert!(inst.data.join("game.toml").exists());

    fs::write(dir.path().join("portable.txt"), "").unwrap();
    apply(&inst, "a", SettingInput::Text { value: "y".into() }).unwrap();
    assert_eq!(
        fs::read_to_string(dir.path().join("portable.toml")).unwrap(),
        "k = \"y\"\n"
    );
}

#[test]
fn number_formats() {
    use format::{parse, render, NumberFormat::*};
    assert_eq!(parse("%g"), Some(General));
    assert_eq!(parse("%.2f"), Some(Fixed(2)));
    assert_eq!(parse("%d"), Some(Integer));
    assert_eq!(parse("%s"), None);
    assert_eq!(render(General, 1.0), "1");
    assert_eq!(render(General, 0.25), "0.25");
    assert_eq!(render(Fixed(2), 1.0), "1.00");
    assert_eq!(render(Integer, 2.6), "3");
}

#[test]
fn json_wire_format_matches_the_frontend_types() {
    let (_dir, inst) = instance(MANIFEST, Some(GAME_TOML));
    let json = serde_json::to_value(load_view(&inst).unwrap()).unwrap();
    let by_id = |id: &str| {
        json["settings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == id)
            .unwrap()
            .clone()
    };

    let vsync = by_id("vsync");
    assert_eq!(vsync["help"], serde_json::Value::Null);
    assert_eq!(vsync["enabled"], true);
    assert_eq!(
        vsync["control"],
        serde_json::json!({ "type": "choices", "style": "toggle", "options": ["Off", "On"], "value": 1, "custom": null })
    );
    assert_eq!(by_id("volume")["control"]["type"], "slider");
    assert_eq!(
        by_id("name")["control"],
        serde_json::json!({ "type": "text", "value": "" })
    );
    assert_eq!(
        by_id("keys")["control"]["actions"][0]["default"],
        serde_json::json!(["Space"])
    );
}

#[test]
fn accepts_the_exact_json_the_frontend_sends() {
    let location: Location = serde_json::from_str(r#"{"installDir":"/g","dataDir":"/d"}"#).unwrap();
    assert_eq!(
        Instance::from(location).data,
        std::path::PathBuf::from("/d")
    );
    let location: Location = serde_json::from_str(r#"{"installDir":"/g"}"#).unwrap();
    assert_eq!(
        Instance::from(location).data,
        std::path::PathBuf::from("/g/recompdata")
    );

    for raw in [
        r#"{"type":"choice","index":1}"#,
        r#"{"type":"number","value":0.5}"#,
        r#"{"type":"text","value":"x"}"#,
        r#"{"type":"bindings","action":"key.cross","keys":["A"]}"#,
    ] {
        serde_json::from_str::<SettingInput>(raw).unwrap_or_else(|e| panic!("{raw}: {e}"));
    }
}
