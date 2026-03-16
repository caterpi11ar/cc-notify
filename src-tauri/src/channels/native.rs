use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;
use tauri_plugin_notification::NotificationExt;

pub struct NativeChannel {
    app_handle: tauri::AppHandle,
}

impl NativeChannel {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl NotificationChannel for NativeChannel {
    fn channel_type(&self) -> &'static str {
        "native"
    }

    fn validate_config(&self, _config: &ChannelConfig) -> Result<(), AppError> {
        Ok(()) // Native notifications need no config
    }

    async fn send(
        &self,
        _config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        let header = message.event_header();
        let title = format!("CC Notify: {}", header);
        let body = message.message_body();

        self.app_handle
            .notification()
            .builder()
            .title(&title)
            .body(&body)
            .show()
            .map_err(|e| AppError::Channel(format!("Native notification failed: {e}")))?;

        Ok(SendResult {
            success: true,
            channel_type: "native".to_string(),
            message: None,
        })
    }

    async fn test(&self, config: &ChannelConfig) -> Result<SendResult, AppError> {
        let test_msg = NotificationMessage {
            event: "test".to_string(),
            event_type: None,
            message: Some("Test notification from CC Notify".to_string()),
            tool: Some("cc-notify".to_string()),
            session_id: None,
            project: None,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now().timestamp(),
            title: None,
            model: None,
            cwd: None,
            last_assistant_message: None,
            source: None,
            reason: None,
            agent_type: None,
        };
        self.send(config, &test_msg).await
    }
}
