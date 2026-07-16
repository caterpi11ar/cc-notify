mod commands;
mod config;
mod database;
mod error;
mod gateway;
mod models;
mod protocol;
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
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                let _ = app.show();
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
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
            commands::gateway::get_gateway_status,
            commands::gateway::start_gateway,
            commands::gateway::stop_gateway,
            commands::gateway::restart_gateway,
            commands::gateway::get_local_api_key_status,
            commands::gateway::regenerate_local_api_key,
            commands::gateway::get_local_api_key_once,
            commands::gateway::get_providers,
            commands::gateway::get_gateway_logs,
            commands::gateway::clear_gateway_logs,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } => {
                api.prevent_close();
                if let Some(window) = app_handle.get_webview_window(&label) {
                    let _ = window.hide();
                }
            }
            tauri::RunEvent::ExitRequested { code, api, .. } => {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = app_handle.show();
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        });
}
