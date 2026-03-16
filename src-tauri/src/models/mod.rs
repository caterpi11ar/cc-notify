use serde::{Deserialize, Serialize};

/// Notification channel configuration stored as JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    #[serde(flatten)]
    pub params: serde_json::Value,
}

/// A notification channel record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub config: ChannelConfig,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub sort_index: i32,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

/// Event type record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventType {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default)]
    pub enabled: bool,
}

/// Custom rule record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub rule_type: String,
    pub pattern: String,
    pub event_type_id: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub created_at: i64,
}

/// Event -> Channel routing record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Routing {
    pub event_type_id: String,
    pub channel_id: String,
    pub enabled: bool,
    pub priority: i32,
}

/// Message template record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub body_template: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub is_default: bool,
}

/// Notification history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationHistory {
    pub id: i64,
    pub event_type_id: String,
    pub channel_id: String,
    pub status: String,
    pub message_body: String,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

/// The message passed through the notification pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub event: String,
    pub event_type: Option<String>,
    pub message: Option<String>,
    pub tool: Option<String>,
    pub session_id: Option<String>,
    pub project: Option<String>,
    pub metadata: serde_json::Value,
    pub timestamp: i64,
}

/// Result of sending a notification through a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub success: bool,
    pub channel_type: String,
    pub message: Option<String>,
}

/// Status of hooks installation for each supported tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksStatus {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
}
