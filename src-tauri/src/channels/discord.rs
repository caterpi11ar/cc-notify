use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;

pub struct DiscordChannel;

#[async_trait]
impl NotificationChannel for DiscordChannel {
    fn channel_type(&self) -> &'static str {
        "discord"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), AppError> {
        let webhook_url = config
            .params
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if webhook_url.is_empty() {
            return Err(AppError::InvalidInput(
                "Discord webhook_url is required".to_string(),
            ));
        }

        if !webhook_url.starts_with("https://discord.com/api/webhooks/")
            && !webhook_url.starts_with("https://discordapp.com/api/webhooks/")
        {
            return Err(AppError::InvalidInput(
                "Invalid Discord webhook URL format".to_string(),
            ));
        }

        Ok(())
    }

    async fn send(
        &self,
        config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        let webhook_url = config
            .params
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AppError::InvalidInput("Discord webhook_url is required".to_string())
            })?;

        let username = config
            .params
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("CC Notify");

        let avatar_url = config
            .params
            .get("avatar_url")
            .and_then(|v| v.as_str());

        let embed_color = config
            .params
            .get("embed_color")
            .and_then(|v| v.as_u64())
            .unwrap_or(5814783); // Default blue color

        let text = message
            .message
            .as_deref()
            .unwrap_or(&message.event);

        let mut embed = serde_json::json!({
            "title": format!("CC Notify: {}", message.event),
            "description": text,
            "color": embed_color,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        // Add tool field if present
        if let Some(tool) = &message.tool {
            embed["fields"] = serde_json::json!([
                {
                    "name": "Tool",
                    "value": tool,
                    "inline": true
                }
            ]);
        }

        let mut payload = serde_json::json!({
            "username": username,
            "embeds": [embed]
        });

        if let Some(url) = avatar_url {
            payload["avatar_url"] = serde_json::Value::String(url.to_string());
        }

        let client = reqwest::Client::new();
        let response = client
            .post(webhook_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::Channel(format!("Discord request failed: {e}")))?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(SendResult {
                success: true,
                channel_type: "discord".to_string(),
                message: None,
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(AppError::Channel(format!(
                "Discord webhook returned {status}: {body}"
            )))
        }
    }

    async fn test(&self, config: &ChannelConfig) -> Result<SendResult, AppError> {
        self.validate_config(config)?;
        let test_msg = NotificationMessage {
            event: "test".to_string(),
            event_type: None,
            message: Some("Test notification from CC Notify".to_string()),
            tool: Some("cc-notify".to_string()),
            session_id: None,
            project: None,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now().timestamp(),
        };
        self.send(config, &test_msg).await
    }
}
