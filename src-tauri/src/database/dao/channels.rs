use super::super::{lock_conn, to_json_string, Database};
use crate::error::AppError;
use crate::models::{Channel, ChannelConfig};

impl Database {
    /// Get all channels ordered by sort_index
    pub fn get_all_channels(&self) -> Result<Vec<Channel>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, channel_type, config, enabled, sort_index, created_at, updated_at
                 FROM channels ORDER BY sort_index ASC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let config_str: String = row.get(3)?;
                Ok(Channel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    channel_type: row.get(2)?,
                    config: serde_json::from_str(&config_str).unwrap_or(ChannelConfig {
                        params: serde_json::Value::Object(serde_json::Map::new()),
                    }),
                    enabled: row.get(4)?,
                    sort_index: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut channels = Vec::new();
        for row in rows {
            channels.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(channels)
    }

    /// Get a single channel by ID
    pub fn get_channel(&self, id: &str) -> Result<Option<Channel>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, channel_type, config, enabled, sort_index, created_at, updated_at
                 FROM channels WHERE id = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![id], |row| {
                let config_str: String = row.get(3)?;
                Ok(Channel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    channel_type: row.get(2)?,
                    config: serde_json::from_str(&config_str).unwrap_or(ChannelConfig {
                        params: serde_json::Value::Object(serde_json::Map::new()),
                    }),
                    enabled: row.get(4)?,
                    sort_index: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result)
    }

    /// Insert a new channel
    pub fn insert_channel(&self, channel: &Channel) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let config_str = to_json_string(&channel.config)?;

        conn.execute(
            "INSERT INTO channels (id, name, channel_type, config, enabled, sort_index, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                channel.id,
                channel.name,
                channel.channel_type,
                config_str,
                channel.enabled,
                channel.sort_index,
                channel.created_at,
                channel.updated_at,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update an existing channel
    pub fn update_channel(&self, channel: &Channel) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let config_str = to_json_string(&channel.config)?;

        conn.execute(
            "UPDATE channels SET name = ?1, channel_type = ?2, config = ?3, enabled = ?4, sort_index = ?5, updated_at = ?6
             WHERE id = ?7",
            rusqlite::params![
                channel.name,
                channel.channel_type,
                config_str,
                channel.enabled,
                channel.sort_index,
                channel.updated_at,
                channel.id,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete a channel by ID
    pub fn delete_channel(&self, id: &str) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute("DELETE FROM channels WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(affected > 0)
    }

    /// Toggle channel enabled status
    pub fn toggle_channel(&self, id: &str, enabled: bool) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "UPDATE channels SET enabled = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
            rusqlite::params![enabled, id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get enabled channels only
    pub fn get_enabled_channels(&self) -> Result<Vec<Channel>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, channel_type, config, enabled, sort_index, created_at, updated_at
                 FROM channels WHERE enabled = 1 ORDER BY sort_index ASC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let config_str: String = row.get(3)?;
                Ok(Channel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    channel_type: row.get(2)?,
                    config: serde_json::from_str(&config_str).unwrap_or(ChannelConfig {
                        params: serde_json::Value::Object(serde_json::Map::new()),
                    }),
                    enabled: row.get(4)?,
                    sort_index: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut channels = Vec::new();
        for row in rows {
            channels.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(channels)
    }
}

use rusqlite::OptionalExtension;
