mod catalog;
mod diagnostics;
mod fs_util;
mod launcher_settings;
mod library;
mod settings;
mod updater;

pub fn run() {
    diagnostics::panic::install();
    let builder = tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;
            if let Ok(dir) = app.path().app_log_dir() {
                diagnostics::panic::arm(dir, diagnostics::app_scrub(app));
            }
            Ok(())
        })
        .manage(library::Jobs::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init());
    #[cfg(feature = "updater")]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    builder
        .invoke_handler(tauri::generate_handler![
            catalog::cached_catalog,
            diagnostics::build_report,
            diagnostics::panic::pending_panic,
            diagnostics::panic::dismiss_panic,
            library::library_snapshot,
            library::install_game,
            library::game_releases,
            library::game_license,
            library::prepare_instance,
            library::start_game,
            library::cancel_job,
            library::read_instance_log,
            library::instance_location,
            catalog::refresh_catalog,
            settings::load_settings,
            settings::set_setting,
            settings::settings_launch_args,
            launcher_settings::load_launcher_settings,
            launcher_settings::set_launcher_setting,
            launcher_settings::restart_launcher,
            updater::updater_available,
            updater::update_launcher
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

        let spec = include_str!("../../packaging/rpm/to-recomp-launcher.spec");
        let spec_version = spec
            .lines()
            .find_map(|line| line.strip_prefix("Version:"))
            .expect("Version: in the RPM spec")
            .trim();
        // A pre-release 1.2.3-rc.1 is packaged as 1.2.3 with a `~rc.1` Release part, so the
        // spec only has to carry the numeric part.
        let numeric = env!("CARGO_PKG_VERSION").split('-').next().unwrap();
        assert_eq!(spec_version, numeric);
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
