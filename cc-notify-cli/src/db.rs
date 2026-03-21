use rusqlite::Connection;
use std::path::Path;

/// Open the cc-notify database read-only
pub fn open_db(path: &Path) -> Result<Connection, String> {
    if !path.exists() {
        return Err(format!("Database not found at {}", path.display()));
    }
    Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Failed to open database: {e}"))
}

/// Open the cc-notify database read-write
pub fn open_db_rw(path: &Path) -> Result<Connection, String> {
    if !path.exists() {
        return Err(format!(
            "Database not found at {}. Run the CC Notify app first to initialize.",
            path.display()
        ));
    }
    let conn = Connection::open(path).map_err(|e| format!("Failed to open database: {e}"))?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| format!("Failed to set busy timeout: {e}"))?;
    conn.execute("PRAGMA foreign_keys = ON;", [])
        .map_err(|e| format!("Failed to enable foreign keys: {e}"))?;
    // WAL mode allows concurrent reads/writes; non-fatal if another process holds the DB
    let _ = conn.execute("PRAGMA journal_mode = WAL;", []);
    Ok(conn)
}

/// Get a setting value from the database
pub fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get(0),
    )
    .ok()
}

/// Get all enabled channels with their configs
pub fn get_enabled_channels(conn: &Connection) -> Result<Vec<(String, String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT id, channel_type, config FROM channels WHERE enabled = 1 ORDER BY sort_index")
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let channels = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("Failed to query channels: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read channels: {e}"))?;

    Ok(channels)
}

/// Get routing for an event type
pub fn get_routing_for_event(
    conn: &Connection,
    event_type_id: &str,
) -> Result<Vec<(String, i32)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT channel_id, priority FROM routing WHERE event_type_id = ?1 AND enabled = 1 ORDER BY priority",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let routes = stmt
        .query_map(rusqlite::params![event_type_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
        })
        .map_err(|e| format!("Failed to query routing: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read routing: {e}"))?;

    Ok(routes)
}

/// Record notification in history
pub fn record_history(
    conn: &Connection,
    event_type_id: &str,
    channel_id: &str,
    status: &str,
    message_body: &str,
    error_message: Option<&str>,
    metadata: &serde_json::Value,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO notification_history (event_type_id, channel_id, status, message_body, error_message, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            event_type_id,
            channel_id,
            status,
            message_body,
            error_message,
            serde_json::to_string(metadata).unwrap_or_else(|_| "{}".to_string()),
        ],
    )
    .map_err(|e| format!("Failed to record history: {e}"))?;
    Ok(())
}
