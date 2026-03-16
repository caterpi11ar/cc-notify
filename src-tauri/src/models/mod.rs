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
    // Rich context fields (all backward-compatible via serde(default))
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub last_assistant_message: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub agent_type: Option<String>,
}

impl NotificationMessage {
    /// Human-readable event display name
    pub fn event_display_name(&self) -> &str {
        match self.event.as_str() {
            "stop" => "Task Completed",
            "subagent_stop" => "Subagent Completed",
            "session_start" => "Session Started",
            "session_end" => "Session Ended",
            "notification.idle_prompt" => "Waiting for Input",
            "notification.permission_prompt" => "Permission Request",
            "notification.auth_success" => "Auth Success",
            "notification.elicitation_dialog" => "MCP Input Required",
            "error" => "Error",
            "test" => "Test",
            _ => "Notification",
        }
    }

    /// Build event header: "Task Completed - my-project"
    pub fn event_header(&self) -> String {
        let display = self
            .title
            .as_deref()
            .unwrap_or(self.event_display_name());
        match &self.project {
            Some(p) if !p.is_empty() => format!("{} - {}", display, p),
            _ => display.to_string(),
        }
    }

    /// Build the body text from context fields
    pub fn message_body(&self) -> String {
        // If an explicit message is set, use it
        if let Some(msg) = &self.message {
            return msg.clone();
        }
        let tool = self.tool.as_deref().unwrap_or("claude");
        match self.event.as_str() {
            "stop" => self
                .last_assistant_message
                .as_deref()
                .map(|s| format!("Summary: {}", truncate_str(s, 200)))
                .unwrap_or_else(|| format!("{} task completed", tool)),

            "subagent_stop" => {
                let agent = self.agent_type.as_deref().unwrap_or("unknown");
                self.last_assistant_message
                    .as_deref()
                    .map(|s| {
                        format!(
                            "Agent: {} | Summary: {}",
                            agent,
                            truncate_str(s, 200)
                        )
                    })
                    .unwrap_or_else(|| format!("Subagent ({}) completed", agent))
            }

            e if e.starts_with("notification.") => match e {
                "notification.idle_prompt" => format!("{} is waiting for input", tool),
                "notification.permission_prompt" => format!("{} needs permission", tool),
                "notification.auth_success" => "Authentication successful".to_string(),
                "notification.elicitation_dialog" => "MCP input required".to_string(),
                _ => format!("{} notification", tool),
            },

            "session_start" => {
                let mut parts = Vec::new();
                if let Some(model) = &self.model {
                    parts.push(format!("Model: {}", model));
                }
                if let Some(source) = &self.source {
                    parts.push(format!("Source: {}", source));
                }
                if parts.is_empty() {
                    format!("{} session started", tool)
                } else {
                    parts.join(" | ")
                }
            }

            "session_end" => self
                .reason
                .as_deref()
                .map(|r| format!("Reason: {}", r))
                .unwrap_or_else(|| format!("{} session ended", tool)),

            "error" => format!("{} encountered an error", tool),
            "test" => "Test notification from CC Notify".to_string(),
            _ => self.event.clone(),
        }
    }

    /// Build context footer: "tool | project | model | timestamp"
    pub fn context_footer(&self) -> String {
        let mut parts = Vec::new();
        if let Some(tool) = &self.tool {
            parts.push(tool.clone());
        }
        if let Some(project) = &self.project {
            if !project.is_empty() {
                parts.push(project.clone());
            }
        }
        if let Some(model) = &self.model {
            parts.push(model.clone());
        }
        parts.push(
            chrono::Local::now()
                .format("%Y-%m-%d %H:%M")
                .to_string(),
        );
        parts.join(" | ")
    }

    /// Return a Discord/embed color for the event type
    pub fn event_color(&self) -> u64 {
        match self.event.as_str() {
            "stop" | "subagent_stop" => 0x00C853,
            "session_start" => 0x2196F3,
            "session_end" => 0x9E9E9E,
            "error" => 0xFF1744,
            "test" => 0x5865F2,
            e if e.starts_with("notification.") => 0xFF9800,
            _ => 0x5865F2,
        }
    }

    /// Feishu card header color template
    pub fn feishu_header_color(&self) -> &str {
        match self.event.as_str() {
            "stop" | "subagent_stop" => "green",
            "session_start" => "blue",
            "session_end" | "test" => "grey",
            "error" => "red",
            e if e.starts_with("notification.") => "orange",
            _ => "blue",
        }
    }
}

/// Truncate to max characters, appending "..." if truncated
fn truncate_str(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let truncated: String = chars[..max].iter().collect();
        format!("{}...", truncated)
    }
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
