pub(crate) mod dao;
pub(crate) mod schema;

use crate::config::get_app_config_dir;
use crate::error::AppError;
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;

/// Current schema version
pub(crate) const SCHEMA_VERSION: i32 = 5;

/// Safely serialize to JSON
pub(crate) fn to_json_string<T: Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value)
        .map_err(|e| AppError::Config(format!("JSON serialization failed: {e}")))
}

/// Safe mutex lock macro
macro_rules! lock_conn {
    ($mutex:expr) => {
        $mutex
            .lock()
            .map_err(|e| AppError::Database(format!("Mutex lock failed: {}", e)))?
    };
}
pub(crate) use lock_conn;

/// Database connection wrapper
pub struct Database {
    pub(crate) conn: Mutex<Connection>,
}

impl Database {
    /// Initialize database at ~/.cc-notify/cc-notify.db
    pub fn init() -> Result<Self, AppError> {
        let db_path = get_app_config_dir().join("cc-notify.db");

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute("PRAGMA foreign_keys = ON;", [])
            .map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute("PRAGMA auto_vacuum = INCREMENTAL;", [])
            .map_err(|e| AppError::Database(e.to_string()))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.create_tables()?;
        db.apply_schema_migrations()?;
        db.seed_builtin_data()?;

        Ok(db)
    }

    /// In-memory database for testing
    pub fn memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute("PRAGMA foreign_keys = ON;", [])
            .map_err(|e| AppError::Database(e.to_string()))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.create_tables()?;
        db.seed_builtin_data()?;

        Ok(db)
    }
}
