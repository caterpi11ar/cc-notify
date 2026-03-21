use std::path::PathBuf;

/// Get the cc-notify binary path for hooks.
/// Priority: 1) app-installed (~/.cc-notify/bin/), 2) PATH lookup, 3) current_exe() fallback
fn get_cc_notify_bin() -> String {
    // 1. Check the app-installed location
    let installed = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cc-notify")
        .join("bin")
        .join(if cfg!(windows) { "cc-notify.exe" } else { "cc-notify" });
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

    // 3. Fall back to the current binary path
    std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "cc-notify".to_string())
}

fn get_claude_settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("settings.json")
}

fn get_codex_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
        .join("config.toml")
}

fn get_gemini_settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".gemini")
        .join("settings.json")
}

fn get_backups_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cc-notify")
        .join("backups")
}

/// Backup a config file before modification
fn backup_file(path: &std::path::Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let backups_dir = get_backups_dir();
    std::fs::create_dir_all(&backups_dir)
        .map_err(|e| format!("Failed to create backups dir: {e}"))?;

    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}", filename, timestamp);
    let backup_path = backups_dir.join(backup_name);

    std::fs::copy(path, &backup_path)
        .map_err(|e| format!("Failed to backup {}: {e}", path.display()))?;

    Ok(())
}

/// Atomic write: write to temp file, then rename
fn atomic_write(path: &std::path::Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "No parent directory".to_string())?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create directory: {e}"))?;

    let tmp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| format!("Failed to create temp file: {e}"))?;
    std::fs::write(tmp.path(), content)
        .map_err(|e| format!("Failed to write temp file: {e}"))?;
    tmp.persist(path)
        .map_err(|e| format!("Failed to rename temp file: {e}"))?;

    Ok(())
}

/// Check if a hook entry's command contains "cc-notify"
fn is_cc_notify_entry(entry: &serde_json::Value) -> bool {
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
fn merge_hook_entry(
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

    if let Some(pos) = arr.iter().position(|e| is_cc_notify_entry(e)) {
        arr[pos] = entry;
    } else {
        arr.push(entry);
    }
}

/// Selectively remove cc-notify entries from a hooks JSON object.
/// Cleans up empty arrays and returns whether hooks object is now empty.
fn remove_cc_notify_from_hooks(hooks: &mut serde_json::Map<String, serde_json::Value>) {
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
}

fn value_contains_cc_notify(value: &toml_edit::Value) -> bool {
    if let Some(s) = value.as_str() {
        return s.contains("cc-notify");
    }

    value
        .as_array()
        .is_some_and(|arr| {
            arr.iter()
                .filter_map(|value| value.as_str())
                .any(|s| s.contains("cc-notify"))
        })
}

fn notify_item_contains_cc_notify(item: &toml_edit::Item) -> bool {
    item.as_value().is_some_and(value_contains_cc_notify)
}

fn codex_doc_has_cc_notify_hook(doc: &toml_edit::DocumentMut) -> bool {
    doc.get("notify")
        .is_some_and(notify_item_contains_cc_notify)
}

/// Install hooks for specified tool(s)
pub fn install(tool: &str) -> Result<(), String> {
    match tool {
        "claude" => install_claude_hooks(),
        "codex" => install_codex_hooks(),
        "gemini" => install_gemini_hooks(),
        "all" => {
            let mut errors = Vec::new();
            if let Err(e) = install_claude_hooks() {
                errors.push(format!("Claude: {e}"));
            }
            if let Err(e) = install_codex_hooks() {
                errors.push(format!("Codex: {e}"));
            }
            if let Err(e) = install_gemini_hooks() {
                errors.push(format!("Gemini: {e}"));
            }
            if errors.is_empty() {
                println!("All hooks installed successfully");
                Ok(())
            } else {
                Err(errors.join("\n"))
            }
        }
        _ => Err(format!("Unknown tool: {tool}. Use: claude, codex, gemini, or all")),
    }
}

/// Uninstall hooks for specified tool(s)
pub fn uninstall(tool: &str) -> Result<(), String> {
    match tool {
        "claude" => uninstall_claude_hooks(),
        "codex" => uninstall_codex_hooks(),
        "gemini" => uninstall_gemini_hooks(),
        "all" => {
            uninstall_claude_hooks().ok();
            uninstall_codex_hooks().ok();
            uninstall_gemini_hooks().ok();
            println!("All hooks uninstalled");
            Ok(())
        }
        _ => Err(format!("Unknown tool: {tool}")),
    }
}

/// Show hooks installation status
pub fn status() -> Result<(), String> {
    println!("Hooks Status:");

    // Check Claude
    let claude_path = get_claude_settings_path();
    let claude_installed = if claude_path.exists() {
        let content = std::fs::read_to_string(&claude_path).unwrap_or_default();
        content.contains("cc-notify")
    } else {
        false
    };
    println!(
        "  Claude Code: {}",
        if claude_installed {
            "installed"
        } else {
            "not installed"
        }
    );

    // Check Codex
    let codex_path = get_codex_config_path();
    let codex_installed = if codex_path.exists() {
        let content = std::fs::read_to_string(&codex_path).unwrap_or_default();
        content
            .parse::<toml_edit::DocumentMut>()
            .map(|doc| codex_doc_has_cc_notify_hook(&doc))
            .unwrap_or(false)
    } else {
        false
    };
    println!(
        "  Codex: {}",
        if codex_installed {
            "installed"
        } else {
            "not installed"
        }
    );

    // Check Gemini
    let gemini_path = get_gemini_settings_path();
    let gemini_installed = if gemini_path.exists() {
        let content = std::fs::read_to_string(&gemini_path).unwrap_or_default();
        content.contains("cc-notify")
    } else {
        false
    };
    println!(
        "  Gemini CLI: {}",
        if gemini_installed {
            "installed"
        } else {
            "not installed"
        }
    );

    Ok(())
}

/// Send a test hooks event
pub fn test() -> Result<(), String> {
    println!("Sending test notification...");
    let bin = get_cc_notify_bin();

    let output = std::process::Command::new(&bin)
        .args(["send", "--event", "test", "--message", "Hooks test notification", "--tool", "cc-notify"])
        .output()
        .map_err(|e| format!("Failed to run cc-notify: {e}"))?;

    if output.status.success() {
        println!("Test notification sent successfully");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Test failed: {}", stderr);
    }

    Ok(())
}

// ============================================================
// Claude Code hooks
// ============================================================

fn install_claude_hooks() -> Result<(), String> {
    let settings_path = get_claude_settings_path();
    let bin = get_cc_notify_bin();

    // Read existing settings or create empty
    let mut settings: serde_json::Value = if settings_path.exists() {
        backup_file(&settings_path)?;
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read Claude settings: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse Claude settings: {e}"))?
    } else {
        serde_json::json!({})
    };

    // Ensure hooks object exists
    if !settings.get("hooks").is_some_and(|h| h.is_object()) {
        settings["hooks"] = serde_json::json!({});
    }

    let hooks = settings.get_mut("hooks").unwrap();

    let entries: Vec<(&str, &str, &str)> = vec![
        ("Stop", "", "stop"),
        ("Notification", "idle_prompt|permission_prompt|auth_success|elicitation_dialog", "notification"),
        ("SubagentStop", "", "subagent-stop"),
        ("SessionStart", "", "session-start"),
        ("SessionEnd", "", "session-end"),
    ];

    for (event_name, matcher, event_flag) in entries {
        let entry = serde_json::json!({
            "matcher": matcher,
            "hooks": [{
                "type": "command",
                "command": format!("{} send --event {} --tool claude --silent", bin, event_flag)
            }]
        });
        merge_hook_entry(hooks, event_name, entry);
    }

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    atomic_write(&settings_path, &content)?;

    println!("Claude Code hooks installed at {}", settings_path.display());
    Ok(())
}

fn uninstall_claude_hooks() -> Result<(), String> {
    let settings_path = get_claude_settings_path();
    if !settings_path.exists() {
        return Ok(());
    }

    backup_file(&settings_path)?;

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings: {e}"))?;
    let mut settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {e}"))?;

    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        remove_cc_notify_from_hooks(hooks);

        if hooks.is_empty() {
            if let Some(obj) = settings.as_object_mut() {
                obj.remove("hooks");
            }
        }
    }

    let new_content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    atomic_write(&settings_path, &new_content)?;

    println!("Claude Code hooks uninstalled");
    Ok(())
}

// ============================================================
// Codex hooks
// ============================================================

fn install_codex_hooks() -> Result<(), String> {
    let config_path = get_codex_config_path();
    let bin = get_cc_notify_bin();

    // Read existing config or create empty
    let content = if config_path.exists() {
        backup_file(&config_path)?;
        std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read Codex config: {e}"))?
    } else {
        String::new()
    };

    // Parse with toml_edit to preserve formatting
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| format!("Failed to parse Codex config: {e}"))?;

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

    atomic_write(&config_path, &doc.to_string())?;

    println!("Codex hooks installed at {}", config_path.display());
    Ok(())
}

fn uninstall_codex_hooks() -> Result<(), String> {
    let config_path = get_codex_config_path();
    if !config_path.exists() {
        return Ok(());
    }

    backup_file(&config_path)?;

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {e}"))?;
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| format!("Failed to parse config: {e}"))?;

    // Only remove notify if it contains cc-notify
    if doc
        .get("notify")
        .is_some_and(notify_item_contains_cc_notify)
    {
        doc.remove("notify");
    }

    atomic_write(&config_path, &doc.to_string())?;
    println!("Codex hooks uninstalled");
    Ok(())
}

// ============================================================
// Gemini CLI hooks
// ============================================================

fn install_gemini_hooks() -> Result<(), String> {
    let settings_path = get_gemini_settings_path();
    let bin = get_cc_notify_bin();

    let mut settings: serde_json::Value = if settings_path.exists() {
        backup_file(&settings_path)?;
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read Gemini settings: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse Gemini settings: {e}"))?
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

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    atomic_write(&settings_path, &content)?;

    println!("Gemini CLI hooks installed at {}", settings_path.display());
    Ok(())
}

fn uninstall_gemini_hooks() -> Result<(), String> {
    let settings_path = get_gemini_settings_path();
    if !settings_path.exists() {
        return Ok(());
    }

    backup_file(&settings_path)?;

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings: {e}"))?;
    let mut settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {e}"))?;

    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        remove_cc_notify_from_hooks(hooks);

        if hooks.is_empty() {
            if let Some(obj) = settings.as_object_mut() {
                obj.remove("hooks");
            }
        }
    }

    let new_content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    atomic_write(&settings_path, &new_content)?;

    println!("Gemini CLI hooks uninstalled");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::codex_doc_has_cc_notify_hook;

    #[test]
    fn detects_cc_notify_in_notify_array() {
        let doc: toml_edit::DocumentMut =
            r#"notify = ["/Users/test/.cc-notify/bin/cc-notify", "send", "--event", "stop"]"#
                .parse()
                .expect("valid toml");
        assert!(codex_doc_has_cc_notify_hook(&doc));
    }

    #[test]
    fn ignores_cc_notify_in_unrelated_project_path() {
        let doc: toml_edit::DocumentMut = r#"
            model = "gpt-5.3-codex"

            [projects."/Users/test/repos/cc-notify"]
            trust_level = "trusted"
        "#
        .parse()
        .expect("valid toml");
        assert!(!codex_doc_has_cc_notify_hook(&doc));
    }
}
