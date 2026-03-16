use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod db;
mod hooks;
mod notify;

/// CC Notify CLI - Notification manager for AI CLI tools
#[derive(Parser)]
#[command(name = "cc-notify", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a notification
    Send {
        /// Event type (stop, notification, session-start, session-end, error, etc.)
        #[arg(long)]
        event: String,

        /// Notification type for Notification events (idle_prompt, permission_prompt, etc.)
        #[arg(long, name = "type")]
        notification_type: Option<String>,

        /// Message body
        #[arg(long)]
        message: Option<String>,

        /// Source tool (claude, codex, gemini)
        #[arg(long, default_value = "claude")]
        tool: String,

        /// Session ID
        #[arg(long)]
        session_id: Option<String>,

        /// Project path
        #[arg(long)]
        project: Option<String>,

        /// Token count (for token-threshold event)
        #[arg(long)]
        tokens: Option<u64>,

        /// Additional metadata as JSON
        #[arg(long)]
        metadata: Option<String>,

        /// Silent mode - suppress CLI output
        #[arg(long)]
        silent: bool,
    },

    /// Manage hooks integration with AI CLI tools
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },

    /// Enable all notifications (remove kill switch)
    On,

    /// Disable all notifications (set kill switch)
    Off,

    /// Show current status
    Status,

    /// Send a test notification to all enabled channels
    Test,
}

#[derive(Subcommand)]
enum HooksAction {
    /// Install hooks into AI CLI tool configuration
    Install {
        /// Tool to install hooks for
        #[arg(long, default_value = "all")]
        tool: String,
    },
    /// Remove hooks from AI CLI tool configuration
    Uninstall {
        /// Tool to uninstall hooks from
        #[arg(long, default_value = "all")]
        tool: String,
    },
    /// Show hooks installation status
    Status,
    /// Send a test event to verify hooks work
    Test,
}

fn get_db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cc-notify")
        .join("cc-notify.db")
}

fn get_kill_switch_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cc-notify")
        .join("disabled")
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Send {
            event,
            notification_type,
            message,
            tool,
            session_id,
            project,
            tokens,
            metadata,
            silent,
        } => {
            let metadata_value = metadata
                .as_deref()
                .map(|m| serde_json::from_str(m).unwrap_or(serde_json::Value::Null))
                .unwrap_or(serde_json::Value::Null);

            notify::send_notification(
                &get_db_path(),
                &event,
                notification_type.as_deref(),
                message.as_deref(),
                &tool,
                session_id.as_deref(),
                project.as_deref(),
                tokens,
                &metadata_value,
                silent,
            )
        }
        Commands::Hooks { action } => match action {
            HooksAction::Install { tool } => hooks::install(&tool),
            HooksAction::Uninstall { tool } => hooks::uninstall(&tool),
            HooksAction::Status => hooks::status(),
            HooksAction::Test => hooks::test(),
        },
        Commands::On => {
            let path = get_kill_switch_path();
            if path.exists() {
                std::fs::remove_file(&path).ok();
            }
            println!("Notifications enabled");
            Ok(())
        }
        Commands::Off => {
            let path = get_kill_switch_path();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&path, "").ok();
            println!("Notifications disabled (kill switch active)");
            Ok(())
        }
        Commands::Status => {
            let kill_switch = get_kill_switch_path().exists();
            let db_exists = get_db_path().exists();
            println!("CC Notify Status:");
            println!("  Kill switch: {}", if kill_switch { "ACTIVE (notifications disabled)" } else { "off" });
            println!("  Database: {}", if db_exists { "found" } else { "not found" });

            if db_exists {
                if let Ok(db) = db::open_db(&get_db_path()) {
                    let channel_count: i64 = db
                        .query_row("SELECT COUNT(*) FROM channels WHERE enabled = 1", [], |row| row.get(0))
                        .unwrap_or(0);
                    let history_count: i64 = db
                        .query_row("SELECT COUNT(*) FROM notification_history", [], |row| row.get(0))
                        .unwrap_or(0);
                    println!("  Active channels: {}", channel_count);
                    println!("  Total notifications sent: {}", history_count);
                }
            }

            // Check hooks status
            hooks::status().ok();
            Ok(())
        }
        Commands::Test => {
            notify::send_notification(
                &get_db_path(),
                "test",
                None,
                Some("This is a test notification from CC Notify"),
                "cc-notify",
                None,
                None,
                None,
                &serde_json::Value::Null,
                false,
            )
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
