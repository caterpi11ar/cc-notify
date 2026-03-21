use crate::config;
use crate::error::AppError;
use super::{get_cc_notify_bin, backup_file};

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

/// Check if Codex hooks are installed
pub fn is_installed() -> Result<bool, AppError> {
    let config_path = config::get_codex_config_path();
    if !config_path.exists() {
        return Ok(false);
    }
    let content =
        std::fs::read_to_string(&config_path).map_err(|e| AppError::io(&config_path, e))?;
    let doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| AppError::Config(format!("Failed to parse Codex config: {e}")))?;
    Ok(codex_doc_has_cc_notify_hook(&doc))
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
/// Only removes notify if it contains "cc-notify", preserving user's own notify config.
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

    if doc
        .get("notify")
        .is_some_and(notify_item_contains_cc_notify)
    {
        doc.remove("notify");
    }

    config::atomic_write(&config_path, &doc.to_string())?;
    log::info!("Codex hooks uninstalled");
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
