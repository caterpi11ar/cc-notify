use super::super::{lock_conn, Database};
use crate::error::AppError;
use rusqlite::OptionalExtension;

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
}
