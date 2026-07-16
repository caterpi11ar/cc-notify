use super::{lock_conn, Database, SCHEMA_VERSION};
use crate::error::AppError;
use rusqlite::Connection;

impl Database {
    pub(crate) fn create_tables(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        Self::create_tables_on_conn(&conn)
    }

    pub(crate) fn create_tables_on_conn(conn: &Connection) -> Result<(), AppError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                base_url TEXT NOT NULL,
                protocol TEXT NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                default_model TEXT NOT NULL DEFAULT '',
                models TEXT NOT NULL DEFAULT '[]',
                model_prefixes TEXT NOT NULL DEFAULT '[]',
                timeout_ms INTEGER NOT NULL DEFAULT 60000,
                max_tokens_field TEXT NOT NULL DEFAULT 'max_tokens',
                secret_ref TEXT,
                has_api_key BOOLEAN NOT NULL DEFAULT 0,
                health_status TEXT NOT NULL DEFAULT 'unknown',
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS provider_secrets (
                secret_ref TEXT PRIMARY KEY,
                provider_id TEXT NOT NULL,
                secret TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS gateway_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                time INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                client_type TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                requested_model TEXT NOT NULL DEFAULT '',
                resolved_provider TEXT,
                resolved_model TEXT,
                protocol_in TEXT NOT NULL,
                protocol_out TEXT,
                status_code INTEGER NOT NULL,
                latency_ms INTEGER NOT NULL,
                stream BOOLEAN NOT NULL DEFAULT 0,
                token_estimate INTEGER,
                input_tokens INTEGER,
                output_tokens INTEGER,
                total_tokens INTEGER,
                cache_creation_input_tokens INTEGER,
                cache_read_input_tokens INTEGER,
                fidelity_mode TEXT NOT NULL DEFAULT 'strict',
                error_message TEXT
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_gateway_logs_time ON gateway_logs(time)",
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

        if current < 8 {
            Self::create_tables_on_conn(&conn)?;
            add_column_if_missing(&conn, "gateway_logs", "input_tokens", "INTEGER")?;
            add_column_if_missing(&conn, "gateway_logs", "output_tokens", "INTEGER")?;
            add_column_if_missing(&conn, "gateway_logs", "total_tokens", "INTEGER")?;
            add_column_if_missing(
                &conn,
                "gateway_logs",
                "cache_creation_input_tokens",
                "INTEGER",
            )?;
            add_column_if_missing(&conn, "gateway_logs", "cache_read_input_tokens", "INTEGER")?;

            for table in [
                "routing",
                "rules",
                "templates",
                "notification_history",
                "event_types",
                "channels",
            ] {
                conn.execute(&format!("DROP TABLE IF EXISTS {table}"), [])
                    .map_err(|e| AppError::Database(format!("Drop {table} failed: {e}")))?;
            }

            conn.execute(
                "DELETE FROM settings WHERE key IN (
                    'quiet_hours_enabled',
                    'quiet_hours_start',
                    'quiet_hours_end',
                    'quiet_hours_days',
                    'rate_limit_per_minute',
                    'rate_limit_cooldown_seconds',
                    'kill_switch',
                    'language',
                    'history_retention_days',
                    'sound_enabled',
                    'sound_volume',
                    'voice_enabled',
                    'voice_name'
                )",
                [],
            )
            .map_err(|e| AppError::Database(format!("Delete legacy settings failed: {e}")))?;
        }

        Self::set_user_version(&conn, SCHEMA_VERSION)?;
        Ok(())
    }
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    column_type: &str,
) -> Result<(), AppError> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| AppError::Database(e.to_string()))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| AppError::Database(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Database(e.to_string()))?;

    if !columns.iter().any(|existing| existing == column) {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {column_type}"),
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    Ok(())
}
