use crate::config;
use crate::error::AppError;

/// Get the cc-notify CLI binary path for hooks commands
fn get_cc_notify_bin() -> String {
    // Try to find cc-notify in PATH first
    if let Ok(output) = std::process::Command::new("which")
        .arg("cc-notify")
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    // Fall back to a default name
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

/// Check if Claude Code hooks are installed
pub fn is_installed() -> Result<bool, AppError> {
    let settings_path = config::get_claude_settings_path();
    if !settings_path.exists() {
        return Ok(false);
    }
    let content =
        std::fs::read_to_string(&settings_path).map_err(|e| AppError::io(&settings_path, e))?;
    Ok(content.contains("cc-notify"))
}

/// Install Claude Code hooks into ~/.claude/settings.json
pub fn install() -> Result<(), AppError> {
    let settings_path = config::get_claude_settings_path();
    let bin = get_cc_notify_bin();

    // Read existing settings or create empty object
    let mut settings: serde_json::Value = if settings_path.exists() {
        backup_file(&settings_path)?;
        config::read_json_file(&settings_path)?
    } else {
        serde_json::json!({})
    };

    // Only modify the "hooks" key, preserve everything else
    settings["hooks"] = serde_json::json!({
        "Stop": [{
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": format!("{} send --event stop --tool claude --silent", bin)
            }]
        }],
        "Notification": [{
            "matcher": "idle_prompt|permission_prompt|auth_success|elicitation_dialog",
            "hooks": [{
                "type": "command",
                "command": format!("{} send --event notification --type $HOOK_MATCH --tool claude --silent", bin)
            }]
        }],
        "SubagentStop": [{
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": format!("{} send --event subagent-stop --tool claude --silent", bin)
            }]
        }],
        "SessionStart": [{
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": format!("{} send --event session-start --tool claude --silent", bin)
            }]
        }],
        "SessionEnd": [{
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": format!("{} send --event session-end --tool claude --silent", bin)
            }]
        }]
    });

    config::write_json_file(&settings_path, &settings)?;
    log::info!(
        "Claude Code hooks installed at {}",
        settings_path.display()
    );
    Ok(())
}

/// Uninstall Claude Code hooks from ~/.claude/settings.json
pub fn uninstall() -> Result<(), AppError> {
    let settings_path = config::get_claude_settings_path();
    if !settings_path.exists() {
        return Ok(());
    }

    backup_file(&settings_path)?;

    let mut settings: serde_json::Value = config::read_json_file(&settings_path)?;

    // Only remove hooks key, preserve everything else
    if let Some(obj) = settings.as_object_mut() {
        obj.remove("hooks");
    }

    config::write_json_file(&settings_path, &settings)?;
    log::info!("Claude Code hooks uninstalled");
    Ok(())
}
