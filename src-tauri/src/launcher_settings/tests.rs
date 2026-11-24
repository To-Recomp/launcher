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

mod games_folder {
    use std::fs;

    use tempfile::TempDir;

    use super::super::folders;

    #[test]
    fn the_choice_is_kept_beside_other_settings_and_reset_to_the_default() {
        let config = TempDir::new().unwrap();
        let games = TempDir::new().unwrap();
        fs::write(
            config.path().join("launcher.json"),
            r#"{ "updates": { "auto": false }, "library": { "other": 1 } }"#,
        )
        .unwrap();
        assert_eq!(folders::configured(config.path()).unwrap(), None);

        let chosen = games.path().to_str().unwrap();
        folders::set(config.path(), Some(chosen)).unwrap();
        assert_eq!(
            folders::configured(config.path()).unwrap(),
            Some(games.path().to_path_buf())
        );
        let saved: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(config.path().join("launcher.json")).unwrap())
                .unwrap();
        assert_eq!(saved["updates"]["auto"], false);
        assert_eq!(saved["library"]["other"], 1);

        folders::set(config.path(), None).unwrap();
        assert_eq!(folders::configured(config.path()).unwrap(), None);
        assert!(
            games.path().read_dir().unwrap().next().is_none(),
            "no probe file is left behind"
        );
    }

    #[test]
    fn a_folder_that_can_not_hold_games_is_refused_and_nothing_is_saved() {
        let config = TempDir::new().unwrap();
        let file = config.path().join("a-file");
        fs::write(&file, "x").unwrap();
        for bad in [
            "relative/folder",
            "/no/such/folder/anywhere",
            file.to_str().unwrap(),
        ] {
            assert!(folders::set(config.path(), Some(bad)).is_err(), "{bad}");
        }
        assert!(!config.path().join("launcher.json").exists());
    }

    #[test]
    fn a_broken_launcher_json_is_reported_not_overwritten() {
        let config = TempDir::new().unwrap();
        let games = TempDir::new().unwrap();
        fs::write(config.path().join("launcher.json"), "{ not json").unwrap();
        let err = folders::set(config.path(), Some(games.path().to_str().unwrap())).unwrap_err();
        assert!(err.contains("isn't valid JSON"), "{err}");
        assert_eq!(
            fs::read_to_string(config.path().join("launcher.json")).unwrap(),
            "{ not json"
        );
    }
}

#[test]
fn update_preferences_default_to_a_stable_check_and_follow_what_the_settings_write() {
    let dir = TempDir::new().unwrap();
    assert_eq!(
        update_preferences(dir.path()).unwrap(),
        UpdatePreferences {
            auto: true,
            prerelease: false
        }
    );

    let manifest = manifest().unwrap();
    let instance = instance(dir.path());
    settings::apply_with(
        &instance,
        &manifest,
        "autoUpdate",
        SettingInput::Choice { index: 0 },
    )
    .unwrap();
    settings::apply_with(
        &instance,
        &manifest,
        "updateChannel",
        SettingInput::Choice { index: 1 },
    )
    .unwrap();
    assert_eq!(
        update_preferences(dir.path()).unwrap(),
        UpdatePreferences {
            auto: false,
            prerelease: true
        }
    );
}

#[test]
fn the_language_follows_the_system_until_one_is_chosen() {
    let dir = TempDir::new().unwrap();
    assert_eq!(language(dir.path()).unwrap(), None);

    let manifest = manifest().unwrap();
    let instance = instance(dir.path());
    // System, English, Français
    settings::apply_with(
        &instance,
        &manifest,
        "language",
        SettingInput::Choice { index: 2 },
    )
    .unwrap();
    assert_eq!(language(dir.path()).unwrap().as_deref(), Some("fr"));
    settings::apply_with(
        &instance,
        &manifest,
        "language",
        SettingInput::Choice { index: 0 },
    )
    .unwrap();
    assert_eq!(language(dir.path()).unwrap(), None);
}

#[test]
fn behavior_is_off_until_chosen_and_follows_what_the_settings_write() {
    let dir = TempDir::new().unwrap();
    assert_eq!(
        behavior(dir.path()).unwrap(),
        Behavior {
            tray: false,
            close_on_launch: false
        }
    );

    let manifest = manifest().unwrap();
    let instance = instance(dir.path());
    let index_of = |id: &str| manifest.settings.iter().position(|s| s.id == id).unwrap();
    assert!(index_of("minimizeToTray") != index_of("closeOnLaunch"));
    settings::apply_with(
        &instance,
        &manifest,
        "minimizeToTray",
        SettingInput::Choice { index: 1 },
    )
    .unwrap();
    settings::apply_with(
        &instance,
        &manifest,
        "closeOnLaunch",
        SettingInput::Choice { index: 1 },
    )
    .unwrap();
    assert_eq!(
        behavior(dir.path()).unwrap(),
        Behavior {
            tray: true,
            close_on_launch: true
        }
    );
}

#[test]
fn each_download_speed_choice_is_read_back_as_that_many_megabytes_per_second() {
    let dir = TempDir::new().unwrap();
    let manifest = manifest().unwrap();
    let instance = instance(dir.path());
    let setting = manifest.setting("downloadLimit").unwrap();
    let labels: Vec<&str> = setting
        .choices
        .as_ref()
        .unwrap()
        .iter()
        .map(|c| c.label.as_str())
        .collect();
    assert_eq!(labels[0], "Unlimited");

    let read = || {
        crate::net::limit::bytes_per_second(
            config::read(dir.path())
                .unwrap()
                .pointer("/download/limitMBps")
                .and_then(|v| v.as_f64()),
        )
    };
    assert_eq!(read(), 0, "unlimited until chosen");
    for (index, label) in labels.iter().enumerate().skip(1) {
        settings::apply_with(
            &instance,
            &manifest,
            "downloadLimit",
            SettingInput::Choice { index },
        )
        .unwrap();
        let megabytes: f64 = label.trim_end_matches(" MB/s").parse().unwrap();
        assert_eq!(read(), (megabytes * 1_000_000.0) as u64, "{label}");
    }
    settings::apply_with(
        &instance,
        &manifest,
        "downloadLimit",
        SettingInput::Choice { index: 0 },
    )
    .unwrap();
    assert_eq!(read(), 0);
}

fn ids(view: &SettingsView) -> Vec<&str> {
    view.settings.iter().map(|s| s.id.as_str()).collect()
}

#[test]
fn a_change_shows_the_same_settings_as_loading_whether_or_not_the_launcher_updates_itself() {
    for updater in [false, true] {
        let dir = TempDir::new().unwrap();
        let loaded = load_view(dir.path(), updater).unwrap();
        // Tray: Off, On.
        let changed = change_view(
            dir.path(),
            "minimizeToTray",
            SettingInput::Choice { index: 1 },
            updater,
        )
        .unwrap();
        assert_eq!(ids(&changed), ids(&loaded), "updater: {updater}");

        let update_rows = ["autoUpdate", "updateChannel"];
        assert_eq!(
            update_rows.iter().all(|id| ids(&changed).contains(id)),
            updater,
            "the update settings are offered only where the launcher updates itself"
        );
    }
}
