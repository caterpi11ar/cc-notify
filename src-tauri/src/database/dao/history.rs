use super::super::{lock_conn, to_json_string, Database};
use crate::error::AppError;
use crate::models::NotificationHistory;

impl Database {
    /// Insert a notification history record
    pub fn insert_history(&self, history: &NotificationHistory) -> Result<i64, AppError> {
        let conn = lock_conn!(self.conn);
        let metadata_str = to_json_string(&history.metadata)?;

        conn.execute(
            "INSERT INTO notification_history (event_type_id, channel_id, status, message_body, error_message, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                history.event_type_id,
                history.channel_id,
                history.status,
                history.message_body,
                history.error_message,
                metadata_str,
                history.created_at,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();
        Ok(id)
    }

    /// Get notification history with pagination
    pub fn get_history(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NotificationHistory>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, event_type_id, channel_id, status, message_body, error_message, metadata, created_at
                 FROM notification_history
                 ORDER BY created_at DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![limit, offset], |row| {
                let metadata_str: String = row.get(6)?;
                Ok(NotificationHistory {
                    id: row.get(0)?,
                    event_type_id: row.get(1)?,
                    channel_id: row.get(2)?,
                    status: row.get(3)?,
                    message_body: row.get(4)?,
                    error_message: row.get(5)?,
                    metadata: serde_json::from_str(&metadata_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(history)
    }

    /// Get notification history filtered by event type
    pub fn get_history_by_event_type(
        &self,
        event_type_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NotificationHistory>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, event_type_id, channel_id, status, message_body, error_message, metadata, created_at
                 FROM notification_history
                 WHERE event_type_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![event_type_id, limit, offset], |row| {
                let metadata_str: String = row.get(6)?;
                Ok(NotificationHistory {
                    id: row.get(0)?,
                    event_type_id: row.get(1)?,
                    channel_id: row.get(2)?,
                    status: row.get(3)?,
                    message_body: row.get(4)?,
                    error_message: row.get(5)?,
                    metadata: serde_json::from_str(&metadata_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut history_list = Vec::new();
        for row in rows {
            history_list.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(history_list)
    }

    /// Get notification history filtered by channel
    pub fn get_history_by_channel(
        &self,
        channel_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<NotificationHistory>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, event_type_id, channel_id, status, message_body, error_message, metadata, created_at
                 FROM notification_history
                 WHERE channel_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![channel_id, limit, offset], |row| {
                let metadata_str: String = row.get(6)?;
                Ok(NotificationHistory {
                    id: row.get(0)?,
                    event_type_id: row.get(1)?,
                    channel_id: row.get(2)?,
                    status: row.get(3)?,
                    message_body: row.get(4)?,
                    error_message: row.get(5)?,
                    metadata: serde_json::from_str(&metadata_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut history_list = Vec::new();
        for row in rows {
            history_list.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(history_list)
    }

    /// Get total history count
    pub fn get_history_count(&self) -> Result<i64, AppError> {
        let conn = lock_conn!(self.conn);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM notification_history", [], |row| {
                row.get(0)
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(count)
    }

    /// Cleanup old history records (older than given timestamp)
    pub fn cleanup_history(&self, older_than: i64) -> Result<i64, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute(
                "DELETE FROM notification_history WHERE created_at < ?1",
                rusqlite::params![older_than],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(affected as i64)
    }

    /// Clear all history
    pub fn clear_history(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute("DELETE FROM notification_history", [])
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete a single history record
    pub fn delete_history(&self, id: i64) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute(
                "DELETE FROM notification_history WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(affected > 0)
    }
}
