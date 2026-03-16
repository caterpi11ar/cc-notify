use crate::config;
use crate::error::AppError;
use super::{get_cc_notify_bin, backup_file};

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
                "command": format!("{} send --event notification --tool claude --silent", bin)
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
