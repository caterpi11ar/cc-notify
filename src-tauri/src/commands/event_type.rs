use tauri::State;
use crate::store::AppState;
use crate::models::EventType;

#[tauri::command]
pub fn get_event_types(state: State<'_, AppState>) -> Result<Vec<EventType>, String> {
    state.db.get_all_event_types().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_event_type(
    state: State<'_, AppState>,
    event_type: EventType,
) -> Result<EventType, String> {
    let mut et = event_type;
    if et.id.is_empty() {
        et.id = uuid::Uuid::new_v4().to_string();
    }
    state.db.insert_event_type(&et).map_err(|e| e.to_string())?;
    Ok(et)
}

#[tauri::command]
pub fn update_event_type(
    state: State<'_, AppState>,
    id: String,
    event_type: serde_json::Value,
) -> Result<(), String> {
    let existing = state
        .db
        .get_event_type(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Event type not found: {id}"))?;

    let mut merged = serde_json::to_value(&existing).map_err(|e| e.to_string())?;
    if let (Some(base), Some(patch)) = (merged.as_object_mut(), event_type.as_object()) {
        for (k, v) in patch {
            base.insert(k.clone(), v.clone());
        }
    }

    let updated: EventType = serde_json::from_value(merged).map_err(|e| e.to_string())?;
    state
        .db
        .update_event_type(&updated)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_event_type(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_event_type(&id).map_err(|e| e.to_string())?;
    Ok(())
}
