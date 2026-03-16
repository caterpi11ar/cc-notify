use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;

pub struct WebhookChannel;

#[async_trait]
impl NotificationChannel for WebhookChannel {
    fn channel_type(&self) -> &'static str {
        "webhook"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), AppError> {
        let template = config
            .params
            .get("template")
            .and_then(|v| v.as_str())
            .unwrap_or("generic");

        match template {
            "feishu" => {
                let webhook_url = config
                    .params
                    .get("webhook_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if webhook_url.is_empty() {
                    return Err(AppError::InvalidInput(
                        "Feishu webhook_url is required".to_string(),
                    ));
                }

                if !webhook_url.starts_with("https://open.feishu.cn/open-apis/bot/")
                    && !webhook_url.starts_with("https://open.larksuite.com/open-apis/bot/")
                {
                    return Err(AppError::InvalidInput(
                        "Invalid Feishu webhook URL format".to_string(),
                    ));
                }

                Ok(())
            }
            _ => {
                let url = config
                    .params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if url.is_empty() {
                    return Err(AppError::InvalidInput(
                        "Webhook url is required".to_string(),
                    ));
                }

                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(AppError::InvalidInput(
                        "Webhook url must start with http:// or https://".to_string(),
                    ));
                }

                // Validate method if provided
                let method = config
                    .params
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("POST")
                    .to_uppercase();

                if method != "POST" && method != "GET" && method != "PUT" && method != "PATCH" {
                    return Err(AppError::InvalidInput(
                        "Webhook method must be POST, GET, PUT, or PATCH".to_string(),
                    ));
                }

                Ok(())
            }
        }
    }

    async fn send(
        &self,
        config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        let template = config
            .params
            .get("template")
            .and_then(|v| v.as_str())
            .unwrap_or("generic");

        match template {
            "feishu" => self.send_feishu(config, message).await,
            _ => self.send_generic(config, message).await,
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

impl WebhookChannel {
    async fn send_feishu(
        &self,
        config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        let webhook_url = config
            .params
            .get("webhook_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Feishu webhook_url is required".to_string()))?;

        let text = message
            .message
            .as_deref()
            .unwrap_or(&message.event);

        let content = format!("CC Notify: {}\n{}", message.event, text);

        let payload = serde_json::json!({
            "msg_type": "text",
            "content": {
                "text": content
            }
        });

        let client = reqwest::Client::new();
        let response = client
            .post(webhook_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::Channel(format!("Feishu request failed: {e}")))?;

        if response.status().is_success() {
            let body: serde_json::Value = response
                .json()
                .await
                .unwrap_or(serde_json::json!({}));

            let code = body.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
            if code == 0 {
                Ok(SendResult {
                    success: true,
                    channel_type: "webhook".to_string(),
                    message: None,
                })
            } else {
                let msg = body.get("msg").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                Err(AppError::Channel(format!("Feishu API error: {msg}")))
            }
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(AppError::Channel(format!(
                "Feishu webhook returned {status}: {body}"
            )))
        }
    }

    async fn send_generic(
        &self,
        config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        let url = config
            .params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidInput("Webhook url is required".to_string()))?;

        let method = config
            .params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("POST")
            .to_uppercase();

        let timeout_secs = config
            .params
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        let client = reqwest::Client::new();

        // Build the request body
        let body = if let Some(body_template) = config.params.get("body_template").and_then(|v| v.as_str()) {
            let rendered = render_template(body_template, message);
            serde_json::from_str::<serde_json::Value>(&rendered).unwrap_or_else(|_| {
                serde_json::json!({ "text": rendered })
            })
        } else {
            serde_json::json!({
                "event": message.event,
                "event_type": message.event_type,
                "message": message.message,
                "tool": message.tool,
                "session_id": message.session_id,
                "project": message.project,
                "metadata": message.metadata,
                "timestamp": message.timestamp,
            })
        };

        let mut request = match method.as_str() {
            "GET" => client.get(url),
            "PUT" => client.put(url).json(&body),
            "PATCH" => client.patch(url).json(&body),
            _ => client.post(url).json(&body),
        };

        // Add custom headers if provided
        if let Some(headers) = config.params.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key.as_str(), val_str);
                }
            }
        }

        request = request.timeout(std::time::Duration::from_secs(timeout_secs));

        let response = request
            .send()
            .await
            .map_err(|e| AppError::Channel(format!("Webhook request failed: {e}")))?;

        if response.status().is_success() {
            Ok(SendResult {
                success: true,
                channel_type: "webhook".to_string(),
                message: None,
            })
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(AppError::Channel(format!(
                "Webhook returned {status}: {body}"
            )))
        }
    }
}

/// Simple template rendering for webhook body templates
fn render_template(template: &str, message: &NotificationMessage) -> String {
    let mut result = template.to_string();
    result = result.replace("{{event}}", &message.event);
    result = result.replace("{{message}}", message.message.as_deref().unwrap_or(""));
    result = result.replace("{{tool}}", message.tool.as_deref().unwrap_or(""));
    result = result.replace("{{session_id}}", message.session_id.as_deref().unwrap_or(""));
    result = result.replace("{{project}}", message.project.as_deref().unwrap_or(""));
    result = result.replace("{{timestamp}}", &message.timestamp.to_string());
    if let Some(event_type) = &message.event_type {
        result = result.replace("{{event_type}}", event_type);
    }
    result
}
