mod settings;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            settings::load_settings,
            settings::set_setting,
            settings::settings_launch_args
        ])
        .run(tauri::generate_context!())
        .expect("error while running To Recomp Launcher");
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    fn json(raw: &str) -> Value {
        serde_json::from_str(raw).expect("valid JSON")
    }

    #[test]
    fn app_version_is_the_same_everywhere() {
        let conf = json(include_str!("../tauri.conf.json"));
        let package = json(include_str!("../../package.json"));
        assert_eq!(conf["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(package["version"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn window_is_frameless_and_main_capability_covers_the_custom_title_bar() {
        let conf = json(include_str!("../tauri.conf.json"));
        assert_eq!(conf["app"]["windows"][0]["decorations"], false);

        let caps = json(include_str!("../capabilities/default.json"));
        let perms: Vec<&str> = caps["permissions"]
            .as_array()
            .expect("permissions array")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        for needed in [
            "core:window:allow-minimize",
            "core:window:allow-toggle-maximize",
            "core:window:allow-close",
            "core:window:allow-start-dragging",
        ] {
            assert!(perms.contains(&needed), "missing permission {needed}");
        }
    }
}
