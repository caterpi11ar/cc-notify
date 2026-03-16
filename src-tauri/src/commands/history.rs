use tauri::State;
use crate::store::AppState;
use crate::models::NotificationHistory;

#[tauri::command]
pub fn get_history(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<NotificationHistory>, String> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    state.db.get_history(limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_history_by_event_type(
    state: State<'_, AppState>,
    event_type_id: String,
) -> Result<Vec<NotificationHistory>, String> {
    state
        .db
        .get_history_by_event_type(&event_type_id, 100, 0)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    state.db.clear_history().map_err(|e| e.to_string())
}
