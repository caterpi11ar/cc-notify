use crate::models::{ApiKeyStatus, GatewayLog, GatewayStatus, Provider};
use crate::store::AppState;
use tauri::State;

#[tauri::command]
pub fn get_gateway_status(state: State<'_, AppState>) -> Result<GatewayStatus, String> {
    state.gateway.status(&state.db).map_err(Into::into)
}

#[tauri::command]
pub fn start_gateway(state: State<'_, AppState>) -> Result<GatewayStatus, String> {
    state.gateway.start(state.db.clone()).map_err(Into::into)
}

#[tauri::command]
pub fn stop_gateway(state: State<'_, AppState>) -> Result<(), String> {
    state.gateway.stop().map_err(Into::into)
}

#[tauri::command]
pub fn restart_gateway(state: State<'_, AppState>) -> Result<GatewayStatus, String> {
    state.gateway.restart(state.db.clone()).map_err(Into::into)
}

#[tauri::command]
pub fn get_local_api_key_status(state: State<'_, AppState>) -> Result<ApiKeyStatus, String> {
    state.db.get_local_api_key_status().map_err(Into::into)
}

#[tauri::command]
pub fn regenerate_local_api_key(state: State<'_, AppState>) -> Result<String, String> {
    state.db.regenerate_local_api_key().map_err(Into::into)
}

#[tauri::command]
pub fn get_local_api_key_once(state: State<'_, AppState>) -> Result<Option<String>, String> {
    state.db.get_local_api_key_once().map_err(Into::into)
}

#[tauri::command]
pub fn get_providers(state: State<'_, AppState>) -> Result<Vec<Provider>, String> {
    state.db.get_providers().map_err(Into::into)
}

#[tauri::command]
pub fn get_gateway_logs(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<GatewayLog>, String> {
    state
        .db
        .get_gateway_logs(limit.unwrap_or(100))
        .map_err(Into::into)
}

#[tauri::command]
pub fn clear_gateway_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.db.clear_gateway_logs().map_err(Into::into)
}
