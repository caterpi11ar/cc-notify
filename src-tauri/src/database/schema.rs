use super::{lock_conn, Database, SCHEMA_VERSION};
use crate::error::AppError;
use rusqlite::Connection;

impl Database {
    pub(crate) fn create_tables(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        Self::create_tables_on_conn(&conn)
    }

    pub(crate) fn create_tables_on_conn(conn: &Connection) -> Result<(), AppError> {
        // 1. Channels table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                channel_type TEXT NOT NULL,
                config TEXT NOT NULL DEFAULT '{}',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                sort_index INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 2. Event types table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS event_types (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL DEFAULT 'builtin',
                is_builtin BOOLEAN NOT NULL DEFAULT 0,
                config TEXT NOT NULL DEFAULT '{}',
                enabled BOOLEAN NOT NULL DEFAULT 1
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 3. Rules table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                rule_type TEXT NOT NULL,
                pattern TEXT NOT NULL,
                event_type_id TEXT NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                FOREIGN KEY (event_type_id) REFERENCES event_types(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 4. Routing table (event -> channel mapping)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS routing (
                event_type_id TEXT NOT NULL,
                channel_id TEXT NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                priority INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (event_type_id, channel_id),
                FOREIGN KEY (event_type_id) REFERENCES event_types(id) ON DELETE CASCADE,
                FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 5. Templates table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                channel_type TEXT NOT NULL,
                body_template TEXT NOT NULL,
                format TEXT NOT NULL DEFAULT 'text',
                is_default BOOLEAN NOT NULL DEFAULT 0
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 6. Notification history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notification_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type_id TEXT NOT NULL,
                channel_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'sent',
                message_body TEXT NOT NULL DEFAULT '',
                error_message TEXT,
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 7. Settings table (key-value)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_created_at ON notification_history(created_at)",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_event_type ON notification_history(event_type_id)",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub(crate) fn get_user_version(conn: &Connection) -> Result<i32, AppError> {
        conn.query_row("PRAGMA user_version;", [], |row| row.get(0))
            .map_err(|e| AppError::Database(format!("Failed to get user_version: {e}")))
    }

    fn set_user_version(conn: &Connection, version: i32) -> Result<(), AppError> {
        conn.execute(&format!("PRAGMA user_version = {version};"), [])
            .map_err(|e| AppError::Database(format!("Failed to set user_version: {e}")))?;
        Ok(())
    }

    pub(crate) fn apply_schema_migrations(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let current = Self::get_user_version(&conn)?;

        if current >= SCHEMA_VERSION {
            return Ok(());
        }

        // Future migrations go here:
        // if current < 3 { ... }

        // v2: Merge feishu into webhook as a template
        if current < 2 {
            conn.execute(
                "UPDATE channels SET channel_type = 'webhook',
                 config = json_set(config, '$.template', 'feishu')
                 WHERE channel_type = 'feishu'",
                [],
            )
            .map_err(|e| AppError::Database(format!("Migration v2 failed: {e}")))?;
        }

        // v3: Seed default tray_badge channel for existing users
        if current < 3 {
            conn.execute(
                "INSERT OR IGNORE INTO channels (id, name, channel_type, config, enabled, sort_index)
                 VALUES ('builtin-tray-badge', 'Tray Badge', 'tray_badge', '{}', 1, 0)",
                [],
            )
            .map_err(|e| AppError::Database(format!("Migration v3 failed: {e}")))?;
        }

        // v4: Disable noisy event types by default
        if current < 4 {
            conn.execute(
                "UPDATE event_types SET enabled = 0
                 WHERE id IN ('subagent_stop','session_start','session_end',
                              'notification.auth_success',
                              'long_running','error','token_threshold','cost_threshold')
                   AND is_builtin = 1",
                [],
            )
            .map_err(|e| AppError::Database(format!("Migration v4 failed: {e}")))?;
        }

        Self::set_user_version(&conn, SCHEMA_VERSION)?;
        Ok(())
    }

    /// Seed builtin event types on first run
    pub(crate) fn seed_builtin_data(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM event_types WHERE is_builtin = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        if count > 0 {
            return Ok(());
        }

        let builtin_events: Vec<(&str, &str, &str, bool)> = vec![
            ("stop", "Task Complete", "claude_hook", true),
            ("notification.idle_prompt", "Idle Prompt", "claude_hook", true),
            (
                "notification.permission_prompt",
                "Permission Request",
                "claude_hook",
                true,
            ),
            ("notification.auth_success", "Auth Success", "claude_hook", false),
            (
                "notification.elicitation_dialog",
                "MCP Input",
                "claude_hook",
                true,
            ),
            ("subagent_stop", "Subagent Stop", "claude_hook", false),
            ("session_start", "Session Start", "claude_hook", false),
            ("session_end", "Session End", "claude_hook", false),
            ("long_running", "Long Running", "extended", false),
            ("error", "Error", "extended", false),
            ("token_threshold", "Token Threshold", "extended", false),
            ("cost_threshold", "Cost Threshold", "extended", false),
        ];

        let mut stmt = conn
            .prepare("INSERT OR IGNORE INTO event_types (id, name, category, is_builtin, enabled) VALUES (?1, ?2, ?3, 1, ?4)")
            .map_err(|e| AppError::Database(e.to_string()))?;

        for (id, name, category, enabled) in &builtin_events {
            stmt.execute(rusqlite::params![id, name, category, enabled])
                .map_err(|e| AppError::Database(e.to_string()))?;
        }

        // Seed default templates
        let templates = vec![
            (
                "default_native",
                "Default Native",
                "native",
                "{{event}}: {{message}}{{summary}}",
                "text",
            ),
            (
                "default_slack",
                "Default Slack",
                "slack",
                "*{{event}}*\n{{message}}{{summary}}\n_{{tool}} | {{project}} | {{model}} | {{timestamp}}_",
                "markdown",
            ),
            (
                "default_discord",
                "Default Discord",
                "discord",
                "**{{event}}**\n{{message}}{{summary}}\n*{{tool}} | {{project}} | {{model}} | {{timestamp}}*",
                "markdown",
            ),
            (
                "default_telegram",
                "Default Telegram",
                "telegram",
                "<b>{{event}}</b>\n{{message}}{{summary}}\n<i>{{tool}} | {{project}} | {{model}} | {{timestamp}}</i>",
                "html",
            ),
            (
                "default_webhook",
                "Default Webhook",
                "webhook",
                "{\"event\":\"{{event}}\",\"message\":\"{{message}}\",\"tool\":\"{{tool}}\",\"model\":\"{{model}}\",\"project\":\"{{project}}\",\"summary\":\"{{summary}}\",\"timestamp\":\"{{timestamp}}\"}",
                "json",
            ),
            (
                "default_feishu",
                "Default Feishu",
                "webhook",
                "{\"msg_type\":\"text\",\"content\":{\"text\":\"CC Notify: {{event}}\\n{{message}}{{summary}}\"}}",
                "json",
            ),
        ];

        let mut tmpl_stmt = conn
            .prepare("INSERT OR IGNORE INTO templates (id, name, channel_type, body_template, format, is_default) VALUES (?1, ?2, ?3, ?4, ?5, 1)")
            .map_err(|e| AppError::Database(e.to_string()))?;

        for (id, name, channel_type, body, format) in templates {
            tmpl_stmt
                .execute(rusqlite::params![id, name, channel_type, body, format])
                .map_err(|e| AppError::Database(e.to_string()))?;
        }

        // Seed default settings
        let settings = vec![
            ("quiet_hours_enabled", "false"),
            ("quiet_hours_start", "22:00"),
            ("quiet_hours_end", "08:00"),
            ("quiet_hours_days", "[1,2,3,4,5,6,7]"),
            ("rate_limit_per_minute", "10"),
            ("rate_limit_cooldown_seconds", "10"),
            ("kill_switch", "false"),
            ("sound_enabled", "true"),
            ("sound_volume", "80"),
            ("voice_enabled", "false"),
            ("voice_name", "Samantha"),
            ("language", "system"),
            ("history_retention_days", "30"),
        ];

        let mut settings_stmt = conn
            .prepare("INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)")
            .map_err(|e| AppError::Database(e.to_string()))?;

        for (key, value) in settings {
            settings_stmt
                .execute(rusqlite::params![key, value])
                .map_err(|e| AppError::Database(e.to_string()))?;
        }

        // Seed default tray_badge channel
        conn.execute(
            "INSERT OR IGNORE INTO channels (id, name, channel_type, config, enabled, sort_index)
             VALUES ('builtin-tray-badge', 'Tray Badge', 'tray_badge', '{}', 1, 0)",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
