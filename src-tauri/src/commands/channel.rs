use tauri::State;
use crate::store::AppState;
use crate::models::Channel;

#[tauri::command]
pub fn get_channels(state: State<'_, AppState>) -> Result<Vec<Channel>, String> {
    state.db.get_all_channels().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_channel(state: State<'_, AppState>, channel: Channel) -> Result<Channel, String> {
    let mut ch = channel;
    if ch.id.is_empty() {
        ch.id = uuid::Uuid::new_v4().to_string();
    }
    let now = chrono::Utc::now().timestamp();
    if ch.created_at == 0 {
        ch.created_at = now;
    }
    if ch.updated_at == 0 {
        ch.updated_at = now;
    }
    state.db.insert_channel(&ch).map_err(|e| e.to_string())?;
    Ok(ch)
}

#[tauri::command]
pub fn update_channel(
    state: State<'_, AppState>,
    id: String,
    channel: serde_json::Value,
) -> Result<(), String> {
    let existing = state
        .db
        .get_channel(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Channel not found: {id}"))?;

    let mut merged = serde_json::to_value(&existing).map_err(|e| e.to_string())?;
    if let (Some(base), Some(patch)) = (merged.as_object_mut(), channel.as_object()) {
        for (k, v) in patch {
            base.insert(k.clone(), v.clone());
        }
    }
    // Always update the updated_at timestamp
    if let Some(obj) = merged.as_object_mut() {
        obj.insert(
            "updated_at".to_string(),
            serde_json::json!(chrono::Utc::now().timestamp()),
        );
    }

    let updated: Channel = serde_json::from_value(merged).map_err(|e| e.to_string())?;
    state.db.update_channel(&updated).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_channel(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_channel(&id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn test_channel(
    state: State<'_, AppState>,
    id: String,
) -> Result<crate::models::SendResult, String> {
    let channel = state
        .db
        .get_channel(&id)
        .map_err(|e| e.to_string())?
        .ok_or("Channel not found")?;

    let adapter = state
        .registry
        .get(&channel.channel_type)
        .ok_or_else(|| format!("Unsupported channel type: {}", channel.channel_type))?;

    adapter
        .test(&channel.config)
        .await
        .map_err(|e| e.to_string())
}
