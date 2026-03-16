use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;
use tauri::Manager;

pub struct TrayBadgeChannel {
    app_handle: tauri::AppHandle,
    count: AtomicU64,
}

impl TrayBadgeChannel {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            app_handle,
            count: AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl NotificationChannel for TrayBadgeChannel {
    fn channel_type(&self) -> &'static str {
        "tray_badge"
    }

    fn validate_config(&self, _config: &ChannelConfig) -> Result<(), AppError> {
        Ok(())
    }

    async fn send(
        &self,
        _config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        let new_count = self.count.fetch_add(1, Ordering::Relaxed) + 1;

        if let Some(window) = self.app_handle.get_webview_window("main") {
            window
                .set_badge_count(Some(new_count as i64))
                .map_err(|e| AppError::Channel(format!("Failed to set badge count: {e}")))?;
        } else {
            return Err(AppError::Channel("Main window not found".to_string()));
        }

        log::info!(
            "Tray badge updated to {} for event '{}'",
            new_count,
            message.event
        );

        Ok(SendResult {
            success: true,
            channel_type: "tray_badge".to_string(),
            message: Some(format!("Badge count: {new_count}")),
        })
    }

    async fn test(&self, config: &ChannelConfig) -> Result<SendResult, AppError> {
        let test_msg = NotificationMessage {
            event: "test".to_string(),
            event_type: None,
            message: Some("Test tray badge from CC Notify".to_string()),
            tool: Some("cc-notify".to_string()),
            session_id: None,
            project: None,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now().timestamp(),
        };
        self.send(config, &test_msg).await
    }
}
