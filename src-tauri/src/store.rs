use crate::database::Database;
use crate::gateway::GatewayRuntime;
use std::sync::Arc;

/// Global application state
pub struct AppState {
    pub db: Arc<Database>,
    pub gateway: Arc<GatewayRuntime>,
}

impl AppState {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            gateway: Arc::new(GatewayRuntime::new()),
        }
    }
}
