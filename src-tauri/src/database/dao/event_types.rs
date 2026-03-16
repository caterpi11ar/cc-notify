use super::super::{lock_conn, to_json_string, Database};
use crate::error::AppError;
use crate::models::EventType;
use rusqlite::OptionalExtension;

impl Database {
    /// Get all event types
    pub fn get_all_event_types(&self) -> Result<Vec<EventType>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, category, is_builtin, config, enabled
                 FROM event_types ORDER BY category, name",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let config_str: String = row.get(4)?;
                Ok(EventType {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    is_builtin: row.get(3)?,
                    config: serde_json::from_str(&config_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                    enabled: row.get(5)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut event_types = Vec::new();
        for row in rows {
            event_types.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(event_types)
    }

    /// Get a single event type by ID
    pub fn get_event_type(&self, id: &str) -> Result<Option<EventType>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, category, is_builtin, config, enabled
                 FROM event_types WHERE id = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![id], |row| {
                let config_str: String = row.get(4)?;
                Ok(EventType {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    is_builtin: row.get(3)?,
                    config: serde_json::from_str(&config_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                    enabled: row.get(5)?,
                })
            })
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result)
    }

    /// Insert a new event type
    pub fn insert_event_type(&self, event_type: &EventType) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let config_str = to_json_string(&event_type.config)?;

        conn.execute(
            "INSERT INTO event_types (id, name, category, is_builtin, config, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                event_type.id,
                event_type.name,
                event_type.category,
                event_type.is_builtin,
                config_str,
                event_type.enabled,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update an existing event type
    pub fn update_event_type(&self, event_type: &EventType) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let config_str = to_json_string(&event_type.config)?;

        conn.execute(
            "UPDATE event_types SET name = ?1, category = ?2, config = ?3, enabled = ?4
             WHERE id = ?5",
            rusqlite::params![
                event_type.name,
                event_type.category,
                config_str,
                event_type.enabled,
                event_type.id,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete an event type by ID (only non-builtin)
    pub fn delete_event_type(&self, id: &str) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute(
                "DELETE FROM event_types WHERE id = ?1 AND is_builtin = 0",
                rusqlite::params![id],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(affected > 0)
    }

    /// Toggle event type enabled status
    pub fn toggle_event_type(&self, id: &str, enabled: bool) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "UPDATE event_types SET enabled = ?1 WHERE id = ?2",
            rusqlite::params![enabled, id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get enabled event types only
    pub fn get_enabled_event_types(&self) -> Result<Vec<EventType>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, category, is_builtin, config, enabled
                 FROM event_types WHERE enabled = 1 ORDER BY category, name",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let config_str: String = row.get(4)?;
                Ok(EventType {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    is_builtin: row.get(3)?,
                    config: serde_json::from_str(&config_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                    enabled: row.get(5)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut event_types = Vec::new();
        for row in rows {
            event_types.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(event_types)
    }

    /// Get event types by category
    pub fn get_event_types_by_category(&self, category: &str) -> Result<Vec<EventType>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, category, is_builtin, config, enabled
                 FROM event_types WHERE category = ?1 ORDER BY name",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![category], |row| {
                let config_str: String = row.get(4)?;
                Ok(EventType {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    is_builtin: row.get(3)?,
                    config: serde_json::from_str(&config_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                    enabled: row.get(5)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut event_types = Vec::new();
        for row in rows {
            event_types.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(event_types)
    }
}
