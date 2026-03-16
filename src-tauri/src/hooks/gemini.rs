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

/// Check if Gemini CLI hooks are installed
pub fn is_installed() -> Result<bool, AppError> {
    let settings_path = config::get_gemini_settings_path();
    if !settings_path.exists() {
        return Ok(false);
    }
    let content =
        std::fs::read_to_string(&settings_path).map_err(|e| AppError::io(&settings_path, e))?;
    Ok(content.contains("cc-notify"))
}

/// Install Gemini CLI hooks into ~/.gemini/settings.json
pub fn install() -> Result<(), AppError> {
    let settings_path = config::get_gemini_settings_path();
    let bin = get_cc_notify_bin();

    let mut settings: serde_json::Value = if settings_path.exists() {
        backup_file(&settings_path)?;
        config::read_json_file(&settings_path)?
    } else {
        serde_json::json!({})
    };

    // Only modify hooks-related keys, preserve everything else
    settings["hooks"] = serde_json::json!({
        "Notification": [{
            "matcher": "idle_prompt|permission_prompt",
            "hooks": [{
                "type": "command",
                "command": format!("{} send --event notification --type $HOOK_MATCH --tool gemini --silent", bin)
            }]
        }],
        "AfterAgent": [{
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": format!("{} send --event stop --tool gemini --silent", bin)
            }]
        }]
    });

    config::write_json_file(&settings_path, &settings)?;
    log::info!(
        "Gemini CLI hooks installed at {}",
        settings_path.display()
    );
    Ok(())
}

/// Uninstall Gemini CLI hooks from ~/.gemini/settings.json
pub fn uninstall() -> Result<(), AppError> {
    let settings_path = config::get_gemini_settings_path();
    if !settings_path.exists() {
        return Ok(());
    }

    backup_file(&settings_path)?;

    let mut settings: serde_json::Value = config::read_json_file(&settings_path)?;

    if let Some(obj) = settings.as_object_mut() {
        obj.remove("hooks");
    }

    config::write_json_file(&settings_path, &settings)?;
    log::info!("Gemini CLI hooks uninstalled");
    Ok(())
}
