use crate::error::AppError;
use std::fs;
use std::io::Write;
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

/// Get local-ai-gateway CLI binary install path (~/.local-ai-gateway/bin/local-ai-gateway)
pub fn get_cli_bin_path() -> PathBuf {
    let name = if cfg!(windows) {
        "local-ai-gateway.exe"
    } else {
        "local-ai-gateway"
    };
    get_app_config_dir().join("bin").join(name)
}

/// Get versioned CLI binary path (~/.local-ai-gateway/bin/local-ai-gateway-<version>)
pub fn get_versioned_cli_bin_path(version: &str) -> PathBuf {
    let name = if cfg!(windows) {
        format!("local-ai-gateway-{version}.exe")
    } else {
        format!("local-ai-gateway-{version}")
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

/// Get local-ai-gateway backups directory
pub fn get_backups_dir() -> PathBuf {
    get_app_config_dir().join("backups")
}

/// Get local-ai-gateway sounds directory (~/.local-ai-gateway/sounds)
pub fn get_sounds_dir() -> PathBuf {
    get_app_config_dir().join("sounds")
}

/// Get Claude Code settings path
pub fn get_claude_settings_path() -> PathBuf {
    get_home_dir().join(".claude").join("settings.json")
}

/// Get Codex config path
pub fn get_codex_config_path() -> PathBuf {
    get_home_dir().join(".codex").join("config.toml")
}

/// Get Gemini settings path
pub fn get_gemini_settings_path() -> PathBuf {
    get_home_dir().join(".gemini").join("settings.json")
}

/// Atomic write: write to temp file then rename (safe against crashes)
pub fn atomic_write(path: &std::path::Path, content: &str) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Config(format!("No parent directory for {}", path.display())))?;
    fs::create_dir_all(parent).map_err(|e| AppError::io(parent, e))?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(|e| AppError::io(parent, e))?;
    tmp.write_all(content.as_bytes())
        .map_err(|e| AppError::io(path, e))?;
    tmp.persist(path).map_err(|e| AppError::io(path, e.error))?;
    Ok(())
}

/// Read and parse a JSON file
pub fn read_json_file<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
) -> Result<T, AppError> {
    let content = fs::read_to_string(path).map_err(|e| AppError::io(path, e))?;
    serde_json::from_str(&content).map_err(|e| AppError::json(path, e))
}

/// Write a value as JSON to a file (atomic)
pub fn write_json_file<T: serde::Serialize>(
    path: &std::path::Path,
    value: &T,
) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(value).map_err(|e| AppError::json(path, e))?;
    atomic_write(path, &content)
}
