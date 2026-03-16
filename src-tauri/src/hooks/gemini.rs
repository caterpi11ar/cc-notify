use crate::config;
use crate::error::AppError;
use super::{get_cc_notify_bin, backup_file};

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
                "command": format!("{} send --event notification --tool gemini --silent", bin)
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
