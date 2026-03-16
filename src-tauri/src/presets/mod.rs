use serde::{Deserialize, Serialize};

/// Built-in notification preset definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Channel types to create
    pub channels: Vec<PresetChannel>,
    /// Event routing configuration
    pub routing: Vec<PresetRouting>,
    /// Rate limit per minute
    pub rate_limit_per_minute: u32,
    /// Quiet hours config (optional)
    pub quiet_hours: Option<PresetQuietHours>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetChannel {
    pub channel_type: String,
    pub name: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetRouting {
    pub event_id: String,
    pub channel_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetQuietHours {
    pub enabled: bool,
    pub start: String,
    pub end: String,
    pub days: Vec<u32>,
}

/// Get all built-in presets
pub fn get_builtin_presets() -> Vec<Preset> {
    vec![
        basic_preset(),
        developer_preset(),
        team_preset(),
        quiet_preset(),
    ]
}

/// Basic preset: system native notifications for task_complete and error
fn basic_preset() -> Preset {
    Preset {
        id: "basic".to_string(),
        name: "Basic".to_string(),
        description: "System notifications for task completion and errors".to_string(),
        channels: vec![PresetChannel {
            channel_type: "native".to_string(),
            name: "System Notifications".to_string(),
            config: serde_json::json!({}),
        }],
        routing: vec![
            PresetRouting {
                event_id: "stop".to_string(),
                channel_type: "native".to_string(),
            },
            PresetRouting {
                event_id: "error".to_string(),
                channel_type: "native".to_string(),
            },
        ],
        rate_limit_per_minute: 5,
        quiet_hours: None,
    }
}

/// Developer preset: native + slack, covers more events
fn developer_preset() -> Preset {
    Preset {
        id: "developer".to_string(),
        name: "Developer".to_string(),
        description: "Native + Slack notifications for all key events".to_string(),
        channels: vec![
            PresetChannel {
                channel_type: "native".to_string(),
                name: "System Notifications".to_string(),
                config: serde_json::json!({}),
            },
            PresetChannel {
                channel_type: "slack".to_string(),
                name: "Slack".to_string(),
                config: serde_json::json!({ "webhook_url": "" }),
            },
        ],
        routing: vec![
            // Native: core events
            PresetRouting { event_id: "stop".to_string(), channel_type: "native".to_string() },
            PresetRouting { event_id: "error".to_string(), channel_type: "native".to_string() },
            PresetRouting { event_id: "notification.idle_prompt".to_string(), channel_type: "native".to_string() },
            PresetRouting { event_id: "notification.permission_prompt".to_string(), channel_type: "native".to_string() },
            PresetRouting { event_id: "session_start".to_string(), channel_type: "native".to_string() },
            PresetRouting { event_id: "session_end".to_string(), channel_type: "native".to_string() },
            PresetRouting { event_id: "long_running".to_string(), channel_type: "native".to_string() },
            // Slack: important events
            PresetRouting { event_id: "stop".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "error".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "token_threshold".to_string(), channel_type: "slack".to_string() },
        ],
        rate_limit_per_minute: 10,
        quiet_hours: None,
    }
}

/// Team preset: slack + discord/teams for all events
fn team_preset() -> Preset {
    Preset {
        id: "team".to_string(),
        name: "Team".to_string(),
        description: "Slack + Discord/Teams for team-wide notifications".to_string(),
        channels: vec![
            PresetChannel {
                channel_type: "slack".to_string(),
                name: "Slack".to_string(),
                config: serde_json::json!({ "webhook_url": "" }),
            },
            PresetChannel {
                channel_type: "discord".to_string(),
                name: "Discord".to_string(),
                config: serde_json::json!({ "webhook_url": "" }),
            },
        ],
        routing: vec![
            // Slack: all events
            PresetRouting { event_id: "stop".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "error".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "notification.idle_prompt".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "notification.permission_prompt".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "session_start".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "session_end".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "long_running".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "token_threshold".to_string(), channel_type: "slack".to_string() },
            PresetRouting { event_id: "cost_threshold".to_string(), channel_type: "slack".to_string() },
            // Discord: critical events only
            PresetRouting { event_id: "error".to_string(), channel_type: "discord".to_string() },
            PresetRouting { event_id: "cost_threshold".to_string(), channel_type: "discord".to_string() },
        ],
        rate_limit_per_minute: 20,
        quiet_hours: None,
    }
}

/// Quiet preset: minimal native notifications with quiet hours
fn quiet_preset() -> Preset {
    Preset {
        id: "quiet".to_string(),
        name: "Quiet".to_string(),
        description: "Minimal notifications with quiet hours (22:00-08:00)".to_string(),
        channels: vec![PresetChannel {
            channel_type: "native".to_string(),
            name: "System Notifications".to_string(),
            config: serde_json::json!({}),
        }],
        routing: vec![
            PresetRouting {
                event_id: "error".to_string(),
                channel_type: "native".to_string(),
            },
            PresetRouting {
                event_id: "cost_threshold".to_string(),
                channel_type: "native".to_string(),
            },
        ],
        rate_limit_per_minute: 3,
        quiet_hours: Some(PresetQuietHours {
            enabled: true,
            start: "22:00".to_string(),
            end: "08:00".to_string(),
            days: vec![1, 2, 3, 4, 5, 6, 7],
        }),
    }
}
