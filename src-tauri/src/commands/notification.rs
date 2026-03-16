use tauri::State;
use crate::store::AppState;
use crate::models::{NotificationMessage, SendResult};

#[tauri::command]
pub async fn send_notification(
    state: State<'_, AppState>,
    event: String,
    event_type: Option<String>,
    message: Option<String>,
    tool: Option<String>,
    session_id: Option<String>,
    project: Option<String>,
) -> Result<Vec<SendResult>, String> {
    let msg = NotificationMessage {
        event,
        event_type,
        message,
        tool,
        session_id,
        project,
        metadata: serde_json::Value::Null,
        timestamp: chrono::Utc::now().timestamp(),
    };

    // Stub: until NotificationService is fully implemented, return empty results
    // When services::NotificationService is ready, replace with:
    // let service = crate::services::NotificationService::new(state.db.clone());
    // service.process(&msg).await.map_err(|e| e.to_string())
    let _ = (state, msg);
    Ok(vec![])
}

#[tauri::command]
pub async fn test_notification(
    state: State<'_, AppState>,
) -> Result<Vec<SendResult>, String> {
    send_notification(
        state,
        "test".to_string(),
        None,
        Some("Test notification from CC Notify".to_string()),
        Some("cc-notify".to_string()),
        None,
        None,
    )
    .await
}
