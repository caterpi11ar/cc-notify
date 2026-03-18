mod cli_installer;
mod commands;
mod config;
mod database;
mod error;
mod hooks;
mod models;
mod presets;
mod store;
mod tray;

pub use database::Database;
pub use error::AppError;
pub use store::AppState;

use std::sync::Arc;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // Auto-install CLI binary to ~/.cc-notify/bin/
            cli_installer::install_cli(app);

            let db = Database::init().map_err(|e| {
                log::error!("Database init failed: {e}");
                e.to_string()
            })?;
            let db = Arc::new(db);
            let app_state = AppState::new(db);
            app.manage(app_state);

            // Create system tray
            let handle = app.handle().clone();
            tray::create_tray(&handle)?;

            // Show window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Channel commands
            commands::channel::get_channels,
            commands::channel::create_channel,
            commands::channel::update_channel,
            commands::channel::delete_channel,
            commands::channel::test_channel,
            // Event type commands
            commands::event_type::get_event_types,
            commands::event_type::create_event_type,
            commands::event_type::update_event_type,
            commands::event_type::delete_event_type,
            // Rule commands
            commands::rule::get_rules,
            commands::rule::create_rule,
            commands::rule::update_rule,
            commands::rule::delete_rule,
            // Routing commands
            commands::routing::get_routings,
            commands::routing::get_routings_by_event_type,
            commands::routing::set_routing,
            commands::routing::delete_routing,
            // Template commands
            commands::template::get_templates,
            commands::template::create_template,
            commands::template::update_template,
            commands::template::delete_template,
            // History commands
            commands::history::get_history,
            commands::history::get_history_by_event_type,
            commands::history::clear_history,
            // Settings commands
            commands::settings::get_settings,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::delete_setting,
            // Hooks commands
            commands::hooks::get_hooks_status,
            commands::hooks::install_hook,
            commands::hooks::uninstall_hook,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
