use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;

pub struct TeamsChannel;

#[async_trait]
impl NotificationChannel for TeamsChannel {
    fn channel_type(&self) -> &'static str {
        "teams"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), AppError> {
        let webhook_url = config
            .params
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if webhook_url.is_empty() {
            return Err(AppError::InvalidInput(
                "Teams webhook_url is required".to_string(),
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
                AppError::InvalidInput("Teams webhook_url is required".to_string())
            })?;

        let text = message
            .message
            .as_deref()
            .unwrap_or(&message.event);

        // Use Adaptive Card format for Teams
        let payload = serde_json::json!({
            "type": "message",
            "attachments": [
                {
                    "contentType": "application/vnd.microsoft.card.adaptive",
                    "contentUrl": null,
                    "content": {
                        "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
                        "type": "AdaptiveCard",
                        "version": "1.4",
                        "body": [
                            {
                                "type": "TextBlock",
                                "size": "Medium",
                                "weight": "Bolder",
                                "text": format!("CC Notify: {}", message.event)
                            },
                            {
                                "type": "TextBlock",
                                "text": text,
                                "wrap": true
                            },
                            {
                                "type": "FactSet",
                                "facts": build_facts(message)
                            }
                        ]
                    }
                }
            ]
        });

        let client = reqwest::Client::new();
        let response = client
            .post(webhook_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::Channel(format!("Teams request failed: {e}")))?;

        if response.status().is_success() {
            Ok(SendResult {
                success: true,
                channel_type: "teams".to_string(),
                message: None,
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(AppError::Channel(format!(
                "Teams webhook returned {status}: {body}"
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

fn build_facts(message: &NotificationMessage) -> Vec<serde_json::Value> {
    let mut facts = Vec::new();

    if let Some(tool) = &message.tool {
        facts.push(serde_json::json!({
            "title": "Tool",
            "value": tool
        }));
    }

    if let Some(project) = &message.project {
        facts.push(serde_json::json!({
            "title": "Project",
            "value": project
        }));
    }

    if let Some(event_type) = &message.event_type {
        facts.push(serde_json::json!({
            "title": "Event Type",
            "value": event_type
        }));
    }

    facts.push(serde_json::json!({
        "title": "Time",
        "value": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }));

    facts
}
