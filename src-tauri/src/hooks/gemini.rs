use crate::config;
use crate::error::AppError;
use super::{get_cc_notify_bin, backup_file, merge_hook_entry, is_cc_notify_entry};

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
/// Merges cc-notify entries into existing hooks, preserving user's other hook entries.
pub fn install() -> Result<(), AppError> {
    let settings_path = config::get_gemini_settings_path();
    let bin = get_cc_notify_bin();

    let mut settings: serde_json::Value = if settings_path.exists() {
        backup_file(&settings_path)?;
        config::read_json_file(&settings_path)?
    } else {
        serde_json::json!({})
    };

    // Ensure hooks object exists
    if !settings.get("hooks").is_some_and(|h| h.is_object()) {
        settings["hooks"] = serde_json::json!({});
    }

    let hooks = settings.get_mut("hooks").unwrap();

    let entries: Vec<(&str, &str, &str)> = vec![
        ("Notification", "idle_prompt|permission_prompt", "notification"),
        ("AfterAgent", "", "stop"),
    ];

    for (event_name, matcher, event_flag) in entries {
        let entry = serde_json::json!({
            "matcher": matcher,
            "hooks": [{
                "type": "command",
                "command": format!("{} send --event {} --tool gemini --silent", bin, event_flag)
            }]
        });
        merge_hook_entry(hooks, event_name, entry);
    }

    config::write_json_file(&settings_path, &settings)?;
    log::info!(
        "Gemini CLI hooks installed at {}",
        settings_path.display()
    );
    Ok(())
}

/// Uninstall Gemini CLI hooks from ~/.gemini/settings.json
/// Only removes cc-notify entries, preserving user's other hook entries.
pub fn uninstall() -> Result<(), AppError> {
    let settings_path = config::get_gemini_settings_path();
    if !settings_path.exists() {
        return Ok(());
    }

    backup_file(&settings_path)?;

    let mut settings: serde_json::Value = config::read_json_file(&settings_path)?;

    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        let keys: Vec<String> = hooks.keys().cloned().collect();
        for key in &keys {
            if let Some(arr) = hooks.get_mut(key).and_then(|v| v.as_array_mut()) {
                arr.retain(|entry| !is_cc_notify_entry(entry));
            }
        }

        // Clean up empty arrays
        let empty_keys: Vec<String> = hooks
            .iter()
            .filter(|(_, v)| v.as_array().is_some_and(|a| a.is_empty()))
            .map(|(k, _)| k.clone())
            .collect();
        for key in empty_keys {
            hooks.remove(&key);
        }

        if hooks.is_empty() {
            if let Some(obj) = settings.as_object_mut() {
                obj.remove("hooks");
            }
        }
    }

    config::write_json_file(&settings_path, &settings)?;
    log::info!("Gemini CLI hooks uninstalled");
    Ok(())
}
