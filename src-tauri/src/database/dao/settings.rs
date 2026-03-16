use super::super::{lock_conn, Database};
use crate::error::AppError;
use rusqlite::OptionalExtension;
use std::collections::HashMap;

impl Database {
    /// Get a single setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = lock_conn!(self.conn);
        let result = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result)
    }

    /// Set a setting value (insert or update)
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get all settings as a HashMap
    pub fn get_all_settings(&self) -> Result<HashMap<String, String>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings ORDER BY key")
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut settings = HashMap::new();
        for row in rows {
            let (key, value) = row.map_err(|e| AppError::Database(e.to_string()))?;
            settings.insert(key, value);
        }
        Ok(settings)
    }

    /// Delete a setting
    pub fn delete_setting(&self, key: &str) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let affected = conn
            .execute(
                "DELETE FROM settings WHERE key = ?1",
                rusqlite::params![key],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(affected > 0)
    }

    /// Get a setting with a default value if not found
    pub fn get_setting_or_default(&self, key: &str, default: &str) -> Result<String, AppError> {
        match self.get_setting(key)? {
            Some(value) => Ok(value),
            None => Ok(default.to_string()),
        }
    }

    /// Set multiple settings at once
    pub fn set_settings(&self, settings: &HashMap<String, String>) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare("INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)")
            .map_err(|e| AppError::Database(e.to_string()))?;

        for (key, value) in settings {
            stmt.execute(rusqlite::params![key, value])
                .map_err(|e| AppError::Database(e.to_string()))?;
        }

        Ok(())
    }
}
