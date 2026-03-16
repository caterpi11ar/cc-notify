use super::super::{lock_conn, Database};
use crate::error::AppError;
use crate::models::Routing;

impl Database {
    /// Get all routing entries
    pub fn get_all_routing(&self) -> Result<Vec<Routing>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT event_type_id, channel_id, enabled, priority
                 FROM routing ORDER BY priority DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Routing {
                    event_type_id: row.get(0)?,
                    channel_id: row.get(1)?,
                    enabled: row.get(2)?,
                    priority: row.get(3)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut routings = Vec::new();
        for row in rows {
            routings.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(routings)
    }

    /// Get routing entries for a specific event type
    pub fn get_routing_for_event(&self, event_type_id: &str) -> Result<Vec<Routing>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT event_type_id, channel_id, enabled, priority
                 FROM routing WHERE event_type_id = ?1 ORDER BY priority DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![event_type_id], |row| {
                Ok(Routing {
                    event_type_id: row.get(0)?,
                    channel_id: row.get(1)?,
                    enabled: row.get(2)?,
                    priority: row.get(3)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut routings = Vec::new();
        for row in rows {
            routings.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(routings)
    }

    /// Get routing entries for a specific channel
    pub fn get_routing_for_channel(&self, channel_id: &str) -> Result<Vec<Routing>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT event_type_id, channel_id, enabled, priority
                 FROM routing WHERE channel_id = ?1 ORDER BY priority DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![channel_id], |row| {
                Ok(Routing {
                    event_type_id: row.get(0)?,
                    channel_id: row.get(1)?,
                    enabled: row.get(2)?,
                    priority: row.get(3)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut routings = Vec::new();
        for row in rows {
            routings.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(routings)
    }

    /// Insert or replace a routing entry
    pub fn upsert_routing(&self, routing: &Routing) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT OR REPLACE INTO routing (event_type_id, channel_id, enabled, priority)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                routing.event_type_id,
                routing.channel_id,
                routing.enabled,
                routing.priority,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete a routing entry
    pub fn delete_routing(&self, event_type_id: &str, channel_id: &str) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute(
                "DELETE FROM routing WHERE event_type_id = ?1 AND channel_id = ?2",
                rusqlite::params![event_type_id, channel_id],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(affected > 0)
    }

    /// Toggle routing enabled status
    pub fn toggle_routing(
        &self,
        event_type_id: &str,
        channel_id: &str,
        enabled: bool,
    ) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "UPDATE routing SET enabled = ?1 WHERE event_type_id = ?2 AND channel_id = ?3",
            rusqlite::params![enabled, event_type_id, channel_id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete all routing entries for a specific event type
    pub fn delete_routing_for_event(&self, event_type_id: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "DELETE FROM routing WHERE event_type_id = ?1",
            rusqlite::params![event_type_id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete all routing entries for a specific channel
    pub fn delete_routing_for_channel(&self, channel_id: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "DELETE FROM routing WHERE channel_id = ?1",
            rusqlite::params![channel_id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get enabled routing entries for an event type (with enabled channels)
    pub fn get_enabled_routing_for_event(
        &self,
        event_type_id: &str,
    ) -> Result<Vec<Routing>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT r.event_type_id, r.channel_id, r.enabled, r.priority
                 FROM routing r
                 INNER JOIN channels c ON c.id = r.channel_id
                 WHERE r.event_type_id = ?1 AND r.enabled = 1 AND c.enabled = 1
                 ORDER BY r.priority DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![event_type_id], |row| {
                Ok(Routing {
                    event_type_id: row.get(0)?,
                    channel_id: row.get(1)?,
                    enabled: row.get(2)?,
                    priority: row.get(3)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut routings = Vec::new();
        for row in rows {
            routings.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(routings)
    }
}
