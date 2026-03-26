use std::fs;
use std::path::Path;

use tauri::Manager;

use crate::config;

/// Install bundled CLI binary on app startup:
/// - versioned binary at ~/.cc-notify/bin/cc-notify-<version>
/// - compatibility binary at ~/.cc-notify/bin/cc-notify
/// This is non-fatal: errors are logged but do not prevent the app from starting.
pub fn install_cli(app: &tauri::App) {
    if let Err(e) = try_install_cli(app) {
        log::warn!("CLI auto-install skipped: {e}");
    }
}

/// Install bundled sound files to ~/.cc-notify/sounds/ on app startup.
/// This is non-fatal: errors are logged but do not prevent the app from starting.
pub fn install_sounds(app: &tauri::App) {
    if let Err(e) = try_install_sounds(app) {
        log::warn!("Sounds auto-install skipped: {e}");
    }
}

fn try_install_cli(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let bin_name = if cfg!(windows) { "cc-notify.exe" } else { "cc-notify" };

    // Resolve the bundled resource
    let resource_path = app
        .path()
        .resolve(format!("resources/{bin_name}"), tauri::path::BaseDirectory::Resource)?;

    if !resource_path.exists() {
        return Err(format!("Bundled CLI not found at {}", resource_path.display()).into());
    }

    let versioned_dest = config::get_current_versioned_cli_bin_path();
    let compat_dest = config::get_cli_bin_path();

    // Ensure parent directory exists
    if let Some(parent) = versioned_dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // Install versioned binary once per app version.
    if !versioned_dest.exists() {
        fs::copy(&resource_path, &versioned_dest)?;

        // Set executable permission on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&versioned_dest, fs::Permissions::from_mode(0o755))?;
        }

        log::info!("Versioned CLI installed to {}", versioned_dest.display());
    }

    // Always refresh the legacy fixed path to the current version for old hooks compatibility.
    if let Some(parent) = compat_dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = fs::remove_file(&compat_dest);
    fs::copy(&versioned_dest, &compat_dest)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&compat_dest, fs::Permissions::from_mode(0o755))?;
    }

    cleanup_old_versioned_cli_bins(&versioned_dest)?;

    log::info!("CLI binary installed to {}", compat_dest.display());
    Ok(())
}

fn cleanup_old_versioned_cli_bins(current_versioned_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let Some(bin_dir) = current_versioned_path.parent() else {
        return Ok(());
    };
    let Some(current_name) = current_versioned_path.file_name().and_then(|n| n.to_str()) else {
        return Ok(());
    };

    for entry in fs::read_dir(bin_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path == current_versioned_path || !path.is_file() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if name == current_name {
            continue;
        }

        if parse_versioned_cli_name(name).is_some() {
            match fs::remove_file(&path) {
                Ok(()) => log::info!("Removed old versioned CLI {}", path.display()),
                Err(err) => log::warn!(
                    "Failed to remove old versioned CLI {}: {}",
                    path.display(),
                    err
                ),
            }
        }
    }

    Ok(())
}

fn parse_versioned_cli_name(name: &str) -> Option<&str> {
    let suffix = name.strip_prefix("cc-notify-")?;
    let version = if cfg!(windows) {
        suffix.strip_suffix(".exe")?
    } else {
        suffix
    };

    if version.is_empty() {
        return None;
    }

    let has_digit = version.chars().any(|c| c.is_ascii_digit());
    let valid_chars = version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'));

    if has_digit && valid_chars {
        Some(version)
    } else {
        None
    }
}

fn try_install_sounds(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let resource_path = app
        .path()
        .resolve("resources/sounds/default.mp3", tauri::path::BaseDirectory::Resource)?;

    if !resource_path.exists() {
        return Err(format!("Bundled sound not found at {}", resource_path.display()).into());
    }

    let dest_dir = config::get_sounds_dir();
    fs::create_dir_all(&dest_dir)?;

    let dest = dest_dir.join("default.mp3");

    // Skip if already up-to-date (same file size)
    if dest.exists() {
        let src_meta = fs::metadata(&resource_path)?;
        let dst_meta = fs::metadata(&dest)?;
        if src_meta.len() == dst_meta.len() {
            log::info!("Sound files already up-to-date at {}", dest_dir.display());
            return Ok(());
        }
    }

    fs::copy(&resource_path, &dest)?;
    log::info!("Sound files installed to {}", dest_dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_versioned_cli_name;

    #[test]
    fn parses_versioned_cli_name() {
        if cfg!(windows) {
            assert_eq!(
                parse_versioned_cli_name("cc-notify-0.2.3.exe"),
                Some("0.2.3")
            );
        } else {
            assert_eq!(parse_versioned_cli_name("cc-notify-0.2.3"), Some("0.2.3"));
        }
    }

    #[test]
    fn rejects_non_versioned_or_invalid_names() {
        assert_eq!(parse_versioned_cli_name("cc-notify"), None);
        assert_eq!(parse_versioned_cli_name("cc-notify-"), None);
        assert_eq!(parse_versioned_cli_name("cc-notify-???"), None);
    }
}
