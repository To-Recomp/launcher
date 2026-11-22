use std::fs;

use tempfile::TempDir;

use super::*;

const TEST_MANIFEST: &str = r#"{
  "config": [ { "id": "main", "file": "{data}/launcher.json" } ],
  "settings": [
    { "id": "autoUpdate", "label": "Check for updates",
      "choices": [ { "label": "Off", "set": { "updates/auto": "false" } },
                   { "label": "On",  "set": { "updates/auto": "true" } } ],
      "defaults": { "updates/auto": "true" } }
  ]
}"#;

#[test]
fn the_built_in_manifest_is_valid_and_declares_nothing_that_is_launched() {
    let manifest = manifest().unwrap();
    assert!(manifest.settings.iter().all(|s| !s.is_args()));
    assert!(manifest.start.is_none() && manifest.prepare.is_none());
}

#[test]
fn a_change_is_kept_in_launcher_json_and_shown_again_after_a_reload() {
    let dir = TempDir::new().unwrap();
    let manifest = Manifest::parse(TEST_MANIFEST).unwrap();
    let instance = instance(dir.path());

    let before = settings::build_view(&instance, &manifest).unwrap();
    assert_eq!(
        serde_json::to_value(&before).unwrap()["settings"][0]["control"]["value"],
        1
    );

    settings::apply_with(
        &instance,
        &manifest,
        "autoUpdate",
        SettingInput::Choice { index: 0 },
    )
    .unwrap();

    assert!(fs::read_to_string(dir.path().join("launcher.json"))
        .unwrap()
        .contains("false"));
    let after = settings::build_view(&instance, &manifest).unwrap();
    assert_eq!(
        serde_json::to_value(&after).unwrap()["settings"][0]["control"]["value"],
        0
    );
}

#[test]
fn an_invalid_change_is_rejected_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let manifest = Manifest::parse(TEST_MANIFEST).unwrap();
    let instance = instance(dir.path());

    let err = settings::apply_with(
        &instance,
        &manifest,
        "autoUpdate",
        SettingInput::Choice { index: 7 },
    )
    .unwrap_err();
    assert!(err.contains("doesn't exist"), "{err}");
    let err = settings::apply_with(
        &instance,
        &manifest,
        "nope",
        SettingInput::Choice { index: 0 },
    )
    .unwrap_err();
    assert!(err.contains("Unknown setting"), "{err}");
    assert!(!dir.path().join("launcher.json").exists());
}
