use crate::channels::ChannelRegistry;
use crate::database::Database;
use std::sync::Arc;

/// Global application state
pub struct AppState {
    pub db: Arc<Database>,
    pub registry: ChannelRegistry,
}

impl AppState {
    pub fn new(db: Arc<Database>, app_handle: tauri::AppHandle) -> Self {
        Self {
            db,
            registry: ChannelRegistry::new(app_handle),
        }
    }
}
