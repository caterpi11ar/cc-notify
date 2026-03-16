use crate::config;
use crate::error::AppError;

/// Get the cc-notify CLI binary path for hooks commands
fn get_cc_notify_bin() -> String {
    if let Ok(output) = std::process::Command::new("which")
        .arg("cc-notify")
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    "cc-notify".to_string()
}

/// Backup a config file before modification
fn backup_file(path: &std::path::Path) -> Result<(), AppError> {
    if !path.exists() {
        return Ok(());
    }
    let backups_dir = config::get_backups_dir();
    std::fs::create_dir_all(&backups_dir).map_err(|e| AppError::io(&backups_dir, e))?;

    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}", filename, timestamp);
    let backup_path = backups_dir.join(backup_name);

    std::fs::copy(path, &backup_path).map_err(|e| AppError::io(path, e))?;
    Ok(())
}

/// Check if Codex hooks are installed
pub fn is_installed() -> Result<bool, AppError> {
    let config_path = config::get_codex_config_path();
    if !config_path.exists() {
        return Ok(false);
    }
    let content =
        std::fs::read_to_string(&config_path).map_err(|e| AppError::io(&config_path, e))?;
    Ok(content.contains("cc-notify"))
}

/// Install Codex hooks into ~/.codex/config.toml
pub fn install() -> Result<(), AppError> {
    let config_path = config::get_codex_config_path();
    let bin = get_cc_notify_bin();

    // Read existing config or create empty
    let content = if config_path.exists() {
        backup_file(&config_path)?;
        std::fs::read_to_string(&config_path).map_err(|e| AppError::io(&config_path, e))?
    } else {
        String::new()
    };

    // Parse with toml_edit to preserve formatting
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| AppError::Config(format!("Failed to parse Codex config: {e}")))?;

    // Set the notify key
    let mut notify_array = toml_edit::Array::new();
    notify_array.push(bin.as_str());
    notify_array.push("send");
    notify_array.push("--event");
    notify_array.push("stop");
    notify_array.push("--tool");
    notify_array.push("codex");
    notify_array.push("--silent");
    doc["notify"] = toml_edit::value(notify_array);

    config::atomic_write(&config_path, &doc.to_string())?;
    log::info!("Codex hooks installed at {}", config_path.display());
    Ok(())
}

/// Uninstall Codex hooks from ~/.codex/config.toml
pub fn uninstall() -> Result<(), AppError> {
    let config_path = config::get_codex_config_path();
    if !config_path.exists() {
        return Ok(());
    }

    backup_file(&config_path)?;

    let content =
        std::fs::read_to_string(&config_path).map_err(|e| AppError::io(&config_path, e))?;
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| AppError::Config(format!("Failed to parse Codex config: {e}")))?;

    doc.remove("notify");

    config::atomic_write(&config_path, &doc.to_string())?;
    log::info!("Codex hooks uninstalled");
    Ok(())
}
