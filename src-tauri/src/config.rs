use std::fs;
use std::io::Write;
use std::path::PathBuf;
use crate::error::AppError;

/// Get user home directory with test isolation support
pub fn get_home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("CC_NOTIFY_TEST_HOME") {
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

/// Get cc-notify config directory (~/.cc-notify)
pub fn get_app_config_dir() -> PathBuf {
    get_home_dir().join(".cc-notify")
}

/// Get cc-notify CLI binary install path (~/.cc-notify/bin/cc-notify)
pub fn get_cli_bin_path() -> PathBuf {
    let name = if cfg!(windows) { "cc-notify.exe" } else { "cc-notify" };
    get_app_config_dir().join("bin").join(name)
}

/// Get versioned CLI binary path (~/.cc-notify/bin/cc-notify-<version>)
pub fn get_versioned_cli_bin_path(version: &str) -> PathBuf {
    let name = if cfg!(windows) {
        format!("cc-notify-{version}.exe")
    } else {
        format!("cc-notify-{version}")
    };
    get_app_config_dir().join("bin").join(name)
}

/// Current app version used for versioned CLI filename.
pub fn current_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get current app version's CLI binary path.
pub fn get_current_versioned_cli_bin_path() -> PathBuf {
    get_versioned_cli_bin_path(current_app_version())
}

/// Get cc-notify database path
pub fn get_db_path() -> PathBuf {
    get_app_config_dir().join("cc-notify.db")
}

/// Get cc-notify backups directory
pub fn get_backups_dir() -> PathBuf {
    get_app_config_dir().join("backups")
}

/// Get cc-notify logs directory
pub fn get_logs_dir() -> PathBuf {
    get_app_config_dir().join("logs")
}

/// Get cc-notify sounds directory (~/.cc-notify/sounds)
pub fn get_sounds_dir() -> PathBuf {
    get_app_config_dir().join("sounds")
}

/// Get Claude Code settings path
pub fn get_claude_settings_path() -> PathBuf {
    get_home_dir().join(".claude").join("settings.json")
}

/// Get Claude Code notifications directory (for kill switch)
pub fn get_claude_notifications_dir() -> PathBuf {
    get_home_dir().join(".claude").join("notifications")
}

/// Get Codex config path
pub fn get_codex_config_path() -> PathBuf {
    get_home_dir().join(".codex").join("config.toml")
}

/// Get Gemini settings path
pub fn get_gemini_settings_path() -> PathBuf {
    get_home_dir().join(".gemini").join("settings.json")
}

/// Get kill switch file path
pub fn get_kill_switch_path() -> PathBuf {
    get_app_config_dir().join("disabled")
}

/// Check if kill switch is active
pub fn is_kill_switch_active() -> bool {
    get_kill_switch_path().exists()
}

/// Atomic write: write to temp file then rename (safe against crashes)
pub fn atomic_write(path: &std::path::Path, content: &str) -> Result<(), AppError> {
    let parent = path.parent().ok_or_else(|| {
        AppError::Config(format!("No parent directory for {}", path.display()))
    })?;
    fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| AppError::io(parent, e))?;
    tmp.write_all(content.as_bytes())
        .map_err(|e| AppError::io(path, e))?;
    tmp.persist(path)
        .map_err(|e| AppError::io(path, e.error))?;
    Ok(())
}

/// Read and parse a JSON file
pub fn read_json_file<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T, AppError> {
    let content = fs::read_to_string(path).map_err(|e| AppError::io(path, e))?;
    serde_json::from_str(&content).map_err(|e| AppError::json(path, e))
}

/// Write a value as JSON to a file (atomic)
pub fn write_json_file<T: serde::Serialize>(path: &std::path::Path, value: &T) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(value)
        .map_err(|e| AppError::json(path, e))?;
    atomic_write(path, &content)
}
