use serde::{Deserialize, Serialize};

/// Bundle identifier for CC Notify app.
/// Must match `identifier` in src-tauri/tauri.conf.json.
pub const BUNDLE_ID: &str = "com.ccnotify.desktop";

/// Rich notification context built from CLI args + stdin JSON.
pub struct NotificationContext {
    pub event: String,
    pub notification_type: Option<String>,
    /// Explicit message from CLI --message flag. If set, used as-is (override).
    pub message: Option<String>,
    pub tool: String,
    pub session_id: Option<String>,
    pub project: Option<String>,
    pub cwd: Option<String>,
    pub tokens: Option<u64>,
    pub metadata: serde_json::Value,
    pub silent: bool,
    // Rich context from stdin JSON
    pub last_assistant_message: Option<String>,
    pub model: Option<String>,
    pub source: Option<String>,
    pub reason: Option<String>,
    pub agent_type: Option<String>,
    pub title: Option<String>,
    /// The `message` field from stdin JSON (kept separate from CLI --message).
    /// Used as notification body text for Notification events.
    pub stdin_message: Option<String>,
    /// Send to a specific channel by ID, bypassing the routing table.
    pub channel_id: Option<String>,
    /// Terminal context captured from CLI args / hook stdin / environment.
    pub terminal: TerminalContext,
    /// Shell command template used for "click notification -> jump terminal".
    pub terminal_jump_command: Option<String>,
}

/// Terminal identifiers attached to a notification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerminalContext {
    pub program: Option<String>,
    pub terminal_id: Option<String>,
    pub window_id: Option<String>,
    pub tab_id: Option<String>,
    pub pane_id: Option<String>,
}

impl TerminalContext {
    pub(crate) fn has_any_identifier(&self) -> bool {
        self.terminal_id.is_some()
            || self.window_id.is_some()
            || self.tab_id.is_some()
            || self.pane_id.is_some()
    }
}
