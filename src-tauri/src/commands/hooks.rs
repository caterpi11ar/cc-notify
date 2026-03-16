use crate::models::HooksStatus;

#[tauri::command]
pub fn get_hooks_status() -> Result<HooksStatus, String> {
    crate::hooks::get_hooks_status().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn install_hook(tool: String) -> Result<(), String> {
    crate::hooks::install_hooks(&tool).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn uninstall_hook(tool: String) -> Result<(), String> {
    crate::hooks::uninstall_hooks(&tool).map_err(|e| e.to_string())
}
