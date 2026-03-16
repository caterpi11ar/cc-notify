pub mod claude;
pub mod codex;
pub mod gemini;

use crate::error::AppError;
use crate::models::HooksStatus;

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
