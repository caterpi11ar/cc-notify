use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;

pub struct SlackChannel;

#[async_trait]
impl NotificationChannel for SlackChannel {
    fn channel_type(&self) -> &'static str {
        "slack"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), AppError> {
        let webhook_url = config
            .params
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if webhook_url.is_empty() {
            return Err(AppError::InvalidInput(
                "Slack webhook_url is required".to_string(),
            ));
        }

        if !webhook_url.starts_with("https://hooks.slack.com/") {
            return Err(AppError::InvalidInput(
                "Invalid Slack webhook URL format".to_string(),
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
            .ok_or_else(|| AppError::InvalidInput("Slack webhook_url is required".to_string()))?;

        let mention = config
            .params
            .get("mention")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let header = message.event_header();
        let body = message.message_body();
        let footer = message.context_footer();

        let header_text = if mention.is_empty() {
            header.clone()
        } else {
            format!("{} {}", mention, header)
        };

        let mut blocks = vec![
            // Header block
            serde_json::json!({
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": header_text,
                    "emoji": true
                }
            }),
        ];

        // Body section (skip if empty)
        if !body.is_empty() {
            blocks.push(serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": body
                }
            }));
        }

        // Context footer
        blocks.push(serde_json::json!({
            "type": "context",
            "elements": [{
                "type": "mrkdwn",
                "text": footer
            }]
        }));

        let fallback = format!("{}\n{}", header_text, body);
        let payload = serde_json::json!({
            "text": fallback,
            "blocks": blocks
        });

        let client = reqwest::Client::new();
        let response = client
            .post(webhook_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::Channel(format!("Slack request failed: {e}")))?;

        if response.status().is_success() {
            Ok(SendResult {
                success: true,
                channel_type: "slack".to_string(),
                message: None,
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(AppError::Channel(format!(
                "Slack webhook returned {status}: {body}"
            )))
        }
    }

    async fn test(&self, config: &ChannelConfig) -> Result<SendResult, AppError> {
        self.validate_config(config)?;
        self.send(config, &test_message()).await
    }
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
