use std::sync::Arc;
use crate::channels::ChannelRegistry;
use crate::database::Database;
use crate::error::AppError;
use crate::models::{NotificationHistory, NotificationMessage, SendResult};
use crate::config;
use super::rate_limiter::RateLimiter;
use super::quiet_hours::QuietHoursConfig;
use super::template::TemplateEngine;

pub struct NotificationService {
    db: Arc<Database>,
    registry: ChannelRegistry,
    rate_limiter: RateLimiter,
}

impl NotificationService {
    pub fn new(db: Arc<Database>, app_handle: tauri::AppHandle) -> Self {
        Self {
            db,
            registry: ChannelRegistry::new(app_handle),
            rate_limiter: RateLimiter::new(),
        }
    }

    pub async fn process(
        &self,
        message: &NotificationMessage,
    ) -> Result<Vec<SendResult>, AppError> {
        // 1. Kill switch check
        if config::is_kill_switch_active() {
            return Err(AppError::KillSwitch);
        }

        // 2. Quiet hours check
        let quiet_config = self.load_quiet_hours_config()?;
        if quiet_config.is_active() {
            return Err(AppError::QuietHours);
        }

        // 3. Check if event type is enabled
        let event_type = self.db.get_event_type(&message.event)?;
        if let Some(ref et) = event_type {
            if !et.enabled {
                return Ok(vec![]);
            }
        }

        // 4. Rate limit check
        let max_per_min: u32 = self
            .db
            .get_setting_or_default("rate_limit_per_minute", "10")?
            .parse()
            .unwrap_or(10);
        let cooldown: u64 = self
            .db
            .get_setting_or_default("rate_limit_cooldown_seconds", "10")?
            .parse()
            .unwrap_or(10);
        if !self
            .rate_limiter
            .check(&message.event, max_per_min, cooldown)
        {
            return Err(AppError::RateLimited);
        }

        // 5. Get routing for this event
        let routings = self.db.get_enabled_routing_for_event(&message.event)?;
        if routings.is_empty() {
            // No routing, fall back to native
            if let Some(adapter) = self.registry.get("native") {
                let cfg = crate::models::ChannelConfig {
                    params: serde_json::json!({}),
                };
                let result = adapter.send(&cfg, message).await?;
                // Record history
                self.record_history(&message.event, "native_fallback", &result, message)
                    .ok();
                return Ok(vec![result]);
            }
            return Ok(vec![]);
        }

        // 6. Send to each routed channel
        let mut results = Vec::new();
        let channels = self.db.get_enabled_channels()?;

        for routing in &routings {
            if let Some(channel) = channels.iter().find(|c| c.id == routing.channel_id) {
                if let Some(adapter) = self.registry.get(&channel.channel_type) {
                    // Get template and render
                    let template = self.db.get_default_template(&channel.channel_type)?;
                    let mut rendered_message = message.clone();
                    if let Some(tmpl) = &template {
                        let rendered = TemplateEngine::render(&tmpl.body_template, message);
                        rendered_message.message = Some(rendered);
                    }

                    match adapter.send(&channel.config, &rendered_message).await {
                        Ok(result) => {
                            self.record_history(
                                &message.event,
                                &channel.id,
                                &result,
                                message,
                            )
                            .ok();
                            results.push(result);
                        }
                        Err(e) => {
                            let error_result = SendResult {
                                success: false,
                                channel_type: channel.channel_type.clone(),
                                message: Some(e.to_string()),
                            };
                            self.record_history(
                                &message.event,
                                &channel.id,
                                &error_result,
                                message,
                            )
                            .ok();
                            results.push(error_result);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn load_quiet_hours_config(&self) -> Result<QuietHoursConfig, AppError> {
        let enabled =
            self.db
                .get_setting_or_default("quiet_hours_enabled", "false")?
                == "true";
        let start = self
            .db
            .get_setting_or_default("quiet_hours_start", "22:00")?;
        let end = self
            .db
            .get_setting_or_default("quiet_hours_end", "08:00")?;
        let days_str = self
            .db
            .get_setting_or_default("quiet_hours_days", "[1,2,3,4,5,6,7]")?;

        let (start_hour, start_minute) = parse_time(&start);
        let (end_hour, end_minute) = parse_time(&end);
        let days: Vec<u32> =
            serde_json::from_str(&days_str).unwrap_or_else(|_| vec![1, 2, 3, 4, 5, 6, 7]);

        Ok(QuietHoursConfig {
            enabled,
            start_hour,
            start_minute,
            end_hour,
            end_minute,
            days,
        })
    }

    fn record_history(
        &self,
        event_type_id: &str,
        channel_id: &str,
        result: &SendResult,
        message: &NotificationMessage,
    ) -> Result<(), AppError> {
        let history = NotificationHistory {
            id: 0,
            event_type_id: event_type_id.to_string(),
            channel_id: channel_id.to_string(),
            status: if result.success { "sent" } else { "failed" }.to_string(),
            message_body: message.message.clone().unwrap_or_default(),
            error_message: if result.success {
                None
            } else {
                result.message.clone()
            },
            metadata: message.metadata.clone(),
            created_at: chrono::Utc::now().timestamp(),
        };
        self.db.insert_history(&history)?;
        Ok(())
    }
}

fn parse_time(time_str: &str) -> (u32, u32) {
    let parts: Vec<&str> = time_str.split(':').collect();
    let hour = parts.first().and_then(|h| h.parse().ok()).unwrap_or(0);
    let minute = parts.get(1).and_then(|m| m.parse().ok()).unwrap_or(0);
    (hour, minute)
}
