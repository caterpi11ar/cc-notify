use std::path::PathBuf;

/// Get user home directory with test isolation support
pub fn get_home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("LOCAL_AI_GATEWAY_TEST_HOME") {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    dirs::home_dir().unwrap_or_else(|| {
        log::warn!("Cannot get home directory, falling back to current directory");
        PathBuf::from(".")
    })
}

/// Get local-ai-gateway config directory (~/.local-ai-gateway)
pub fn get_app_config_dir() -> PathBuf {
    get_home_dir().join(".local-ai-gateway")
}
