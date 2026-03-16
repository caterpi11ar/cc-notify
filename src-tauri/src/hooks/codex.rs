use crate::config;
use crate::error::AppError;
use super::{get_cc_notify_bin, backup_file};

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
