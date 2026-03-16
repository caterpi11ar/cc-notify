use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;

pub struct TelegramChannel;

#[async_trait]
impl NotificationChannel for TelegramChannel {
    fn channel_type(&self) -> &'static str {
        "telegram"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), AppError> {
        let bot_token = config
            .params
            .get("bot_token")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let chat_id = config
            .params
            .get("chat_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if bot_token.is_empty() {
            return Err(AppError::InvalidInput(
                "Telegram bot_token is required".to_string(),
            ));
        }

        if chat_id.is_empty() {
            return Err(AppError::InvalidInput(
                "Telegram chat_id is required".to_string(),
            ));
        }

        Ok(())
    }

    async fn send(
        &self,
        config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        let bot_token = config
            .params
            .get("bot_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AppError::InvalidInput("Telegram bot_token is required".to_string())
            })?;

        let chat_id = config
            .params
            .get("chat_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AppError::InvalidInput("Telegram chat_id is required".to_string())
            })?;

        let parse_mode = config
            .params
            .get("parse_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("HTML");

        let disable_notification = config
            .params
            .get("disable_notification")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let header = message.event_header();
        let body = message.message_body();
        let footer = message.context_footer();

        let formatted_text = if body.is_empty() {
            format!(
                "<b>{}</b>\n\n<i>{}</i>",
                html_escape(&header),
                html_escape(&footer),
            )
        } else {
            format!(
                "<b>{}</b>\n\n{}\n\n<i>{}</i>",
                html_escape(&header),
                html_escape(&body),
                html_escape(&footer),
            )
        };

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            bot_token
        );

        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": formatted_text,
            "parse_mode": parse_mode,
            "disable_notification": disable_notification,
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::Channel(format!("Telegram request failed: {e}")))?;

        if response.status().is_success() {
            let body: serde_json::Value = response
                .json()
                .await
                .unwrap_or(serde_json::Value::Null);

            if body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                Ok(SendResult {
                    success: true,
                    channel_type: "telegram".to_string(),
                    message: None,
                })
            } else {
                let description = body
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                Err(AppError::Channel(format!(
                    "Telegram API error: {description}"
                )))
            }
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(AppError::Channel(format!(
                "Telegram API returned {status}: {body}"
            )))
        }
    }

    async fn test(&self, config: &ChannelConfig) -> Result<SendResult, AppError> {
        self.validate_config(config)?;
        self.send(config, &test_message()).await
    }
}

/// Simple HTML escape for Telegram messages
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn test_message() -> NotificationMessage {
    NotificationMessage {
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
    }
}
