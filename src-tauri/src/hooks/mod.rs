pub mod claude;
pub mod codex;
pub mod gemini;

use crate::config;
use crate::error::AppError;
use crate::models::HooksStatus;

/// Get the cc-notify CLI binary path for hooks commands.
/// Priority: 1) app-installed (~/.cc-notify/bin/), 2) PATH lookup, 3) bare fallback
pub(crate) fn get_cc_notify_bin() -> String {
    // 1. Check the app-installed location
    let installed = config::get_cli_bin_path();
    if installed.exists() {
        return installed.display().to_string();
    }

    // 2. Try to find cc-notify in PATH
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    if let Ok(output) = std::process::Command::new(which_cmd)
        .arg("cc-notify")
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    // 3. Fall back to bare name
    "cc-notify".to_string()
}

/// Backup a config file before modification
pub(crate) fn backup_file(path: &std::path::Path) -> Result<(), AppError> {
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

/// Check if a hook entry's command contains "cc-notify"
pub(crate) fn is_cc_notify_entry(entry: &serde_json::Value) -> bool {
    if let Some(hooks) = entry.get("hooks").and_then(|h| h.as_array()) {
        hooks.iter().any(|hook| {
            hook.get("command")
                .and_then(|c| c.as_str())
                .map(|c| c.contains("cc-notify"))
                .unwrap_or(false)
        })
    } else {
        false
    }
}

/// Merge a cc-notify hook entry into a specific event type array within hooks.
/// If an existing cc-notify entry is found, it is replaced; otherwise the new entry is appended.
pub(crate) fn merge_hook_entry(
    hooks: &mut serde_json::Value,
    event_name: &str,
    entry: serde_json::Value,
) {
    let arr = hooks
        .as_object_mut()
        .unwrap()
        .entry(event_name)
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .unwrap();

    // Replace existing cc-notify entry if present, otherwise append
    if let Some(pos) = arr.iter().position(|e| is_cc_notify_entry(e)) {
        arr[pos] = entry;
    } else {
        arr.push(entry);
    }
}

pub fn get_hooks_status() -> Result<HooksStatus, AppError> {
    Ok(HooksStatus {
        claude: claude::is_installed()?,
        codex: codex::is_installed()?,
        gemini: gemini::is_installed()?,
    })
}

pub fn install_hooks(tool: &str) -> Result<(), AppError> {
    match tool {
        "claude" => claude::install(),
        "codex" => codex::install(),
        "gemini" => gemini::install(),
        "all" => {
            claude::install()?;
            codex::install()?;
            gemini::install()?;
            Ok(())
        }
        _ => Err(AppError::InvalidInput(format!("Unknown tool: {tool}"))),
    }
}

pub fn uninstall_hooks(tool: &str) -> Result<(), AppError> {
    match tool {
        "claude" => claude::uninstall(),
        "codex" => codex::uninstall(),
        "gemini" => gemini::uninstall(),
        "all" => {
            claude::uninstall().ok();
            codex::uninstall().ok();
            gemini::uninstall().ok();
            Ok(())
        }
        _ => Err(AppError::InvalidInput(format!("Unknown tool: {tool}"))),
    }
}
