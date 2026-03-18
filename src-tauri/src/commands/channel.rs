use tauri::State;
use crate::store::AppState;
use crate::models::{Channel, SendResult};

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
    _state: State<'_, AppState>,
    id: String,
) -> Result<SendResult, String> {
    let cli_path = dirs::home_dir()
        .ok_or("Cannot determine home directory")?
        .join(".cc-notify/bin/cc-notify");

    if !cli_path.exists() {
        return Err("CLI binary not found at ~/.cc-notify/bin/cc-notify".to_string());
    }

    let output = tokio::process::Command::new(&cli_path)
        .args([
            "send",
            "--event", "test",
            "--tool", "cc-notify",
            "--message", "Test notification from CC Notify",
            "--channel-id", &id,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to run CLI: {e}"))?;

    let success = output.status.success();
    let message = if success {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    };

    Ok(SendResult {
        success,
        channel_type: String::new(),
        message: if message.is_empty() { None } else { Some(message) },
    })
}
