use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;

pub struct VoiceChannel;

#[async_trait]
impl NotificationChannel for VoiceChannel {
    fn channel_type(&self) -> &'static str {
        "voice"
    }

    fn validate_config(&self, _config: &ChannelConfig) -> Result<(), AppError> {
        if !cfg!(target_os = "macos") {
            return Err(AppError::InvalidInput(
                "Voice channel is only available on macOS".to_string(),
            ));
        }
        Ok(())
    }

    async fn send(
        &self,
        config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        if !cfg!(target_os = "macos") {
            return Err(AppError::Channel(
                "Voice channel is only available on macOS".to_string(),
            ));
        }

        let voice = config
            .params
            .get("voice")
            .and_then(|v| v.as_str())
            .unwrap_or("Samantha");

        let rate = config
            .params
            .get("rate")
            .and_then(|v| v.as_u64())
            .unwrap_or(200);

        let text = message
            .message
            .as_deref()
            .unwrap_or(&message.event);

        let speech_text = format!("CC Notify: {}", text);

        std::process::Command::new("say")
            .args(["-v", voice, "-r", &rate.to_string()])
            .arg(&speech_text)
            .spawn()
            .map_err(|e| AppError::Channel(format!("Failed to run say command: {e}")))?;

        Ok(SendResult {
            success: true,
            channel_type: "voice".to_string(),
            message: None,
        })
    }

    async fn test(&self, config: &ChannelConfig) -> Result<SendResult, AppError> {
        self.validate_config(config)?;
        let test_msg = NotificationMessage {
            event: "test".to_string(),
            event_type: None,
            message: Some("Test voice notification from CC Notify".to_string()),
            tool: Some("cc-notify".to_string()),
            session_id: None,
            project: None,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now().timestamp(),
        };
        self.send(config, &test_msg).await
    }
}
