use tauri::State;
use crate::store::AppState;
use crate::models::Routing;

#[tauri::command]
pub fn get_routings(state: State<'_, AppState>) -> Result<Vec<Routing>, String> {
    state.db.get_all_routing().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_routings_by_event_type(
    state: State<'_, AppState>,
    event_type_id: String,
) -> Result<Vec<Routing>, String> {
    state
        .db
        .get_routing_for_event(&event_type_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_routing(state: State<'_, AppState>, routing: Routing) -> Result<(), String> {
    state.db.upsert_routing(&routing).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_routing(
    state: State<'_, AppState>,
    event_type_id: String,
    channel_id: String,
) -> Result<(), String> {
    state
        .db
        .delete_routing(&event_type_id, &channel_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}
