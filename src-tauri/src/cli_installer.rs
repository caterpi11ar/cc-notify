use std::fs;

use tauri::Manager;

use crate::config;

/// Install the bundled CLI binary to ~/.cc-notify/bin/cc-notify on app startup.
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

    let dest = config::get_cli_bin_path();

    // Skip if already up-to-date (same file size)
    if dest.exists() {
        let src_meta = fs::metadata(&resource_path)?;
        let dst_meta = fs::metadata(&dest)?;
        if src_meta.len() == dst_meta.len() {
            log::info!("CLI binary already up-to-date at {}", dest.display());
            return Ok(());
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&resource_path, &dest)?;

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;
    }

    log::info!("CLI binary installed to {}", dest.display());
    Ok(())
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
