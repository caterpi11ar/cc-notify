use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;

mod context_builder;
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

        /// Send to a specific channel by ID (bypass routing table)
        #[arg(long)]
        channel_id: Option<String>,

        /// Terminal program name (e.g. iTerm.app, WezTerm, kitty)
        #[arg(long)]
        terminal_program: Option<String>,

        /// Terminal session identifier
        #[arg(long)]
        terminal_id: Option<String>,

        /// Terminal window identifier
        #[arg(long)]
        terminal_window_id: Option<String>,

        /// Terminal tab identifier
        #[arg(long)]
        terminal_tab_id: Option<String>,

        /// Terminal pane identifier
        #[arg(long)]
        terminal_pane_id: Option<String>,

        /// Jump command template executed when clicking a native notification.
        /// Supports placeholders: {terminal_id}, {terminal_window_id}, {terminal_tab_id},
        /// {terminal_pane_id}, {terminal_program}, {session_id}, {project}, {cwd}, {tool}, {event}
        #[arg(long)]
        terminal_jump_command: Option<String>,
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

    #[command(name = "__native-click-worker", hide = true)]
    NativeClickWorker {
        #[arg(long)]
        summary: String,
        #[arg(long)]
        body: String,
        #[arg(long)]
        jump_command: String,
        #[arg(long, default_value_t = 20)]
        timeout_seconds: u64,
    },
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

/// JSON input from Claude Code / Gemini CLI hooks (passed via stdin)
#[derive(Deserialize, Default, Debug)]
#[serde(default)]
struct HookInput {
    session_id: Option<String>,
    transcript_path: Option<String>,
    cwd: Option<String>,
    hook_event_name: Option<String>,
    // Stop / SubagentStop
    last_assistant_message: Option<String>,
    // SubagentStop
    agent_type: Option<String>,
    agent_id: Option<String>,
    // Notification
    notification_type: Option<String>,
    message: Option<String>,
    title: Option<String>,
    // SessionStart
    source: Option<String>,
    model: Option<String>,
    // SessionEnd
    reason: Option<String>,
    // Terminal context
    terminal_program: Option<String>,
    terminal_id: Option<String>,
    terminal_window_id: Option<String>,
    terminal_tab_id: Option<String>,
    terminal_pane_id: Option<String>,
    terminal_jump_command: Option<String>,
}

/// Read and parse stdin JSON when piped (non-TTY)
fn read_stdin_hook_input() -> HookInput {
    if atty::is(atty::Stream::Stdin) {
        return HookInput::default();
    }
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_ok() && !buf.trim().is_empty() {
        serde_json::from_str(&buf).unwrap_or_default()
    } else {
        HookInput::default()
    }
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
    #[cfg(target_os = "macos")]
    {
        let _ = notify_rust::set_application(notify::BUNDLE_ID);
    }

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
            channel_id,
            terminal_program,
            terminal_id,
            terminal_window_id,
            terminal_tab_id,
            terminal_pane_id,
            terminal_jump_command,
        } => {
            let stdin = read_stdin_hook_input();
            let send_context = context_builder::build_send_context(
                &stdin,
                session_id,
                project,
                metadata.as_deref(),
                terminal_program,
                terminal_id,
                terminal_window_id,
                terminal_tab_id,
                terminal_pane_id,
                terminal_jump_command,
            );

            // Merge: CLI args take priority over stdin
            let ctx = notify::NotificationContext {
                event,
                notification_type: notification_type.or(stdin.notification_type),
                message,
                tool,
                session_id: send_context.session_id,
                project: send_context.project,
                cwd: send_context.cwd,
                tokens,
                metadata: send_context.metadata,
                silent,
                last_assistant_message: stdin.last_assistant_message,
                model: stdin.model,
                source: stdin.source,
                reason: stdin.reason,
                agent_type: stdin.agent_type,
                title: stdin.title,
                stdin_message: stdin.message,
                channel_id,
                terminal: send_context.terminal,
                terminal_jump_command: send_context.terminal_jump_command,
            };

            notify::send_notification(&get_db_path(), &ctx)
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
            println!(
                "  Kill switch: {}",
                if kill_switch {
                    "ACTIVE (notifications disabled)"
                } else {
                    "off"
                }
            );
            println!(
                "  Database: {}",
                if db_exists { "found" } else { "not found" }
            );

            if db_exists {
                if let Ok(db) = db::open_db(&get_db_path()) {
                    let channel_count: i64 = db
                        .query_row(
                            "SELECT COUNT(*) FROM channels WHERE enabled = 1",
                            [],
                            |row| row.get(0),
                        )
                        .unwrap_or(0);
                    let history_count: i64 = db
                        .query_row("SELECT COUNT(*) FROM notification_history", [], |row| {
                            row.get(0)
                        })
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
            let ctx = notify::NotificationContext {
                event: "test".to_string(),
                notification_type: None,
                message: Some("This is a test notification from CC Notify".to_string()),
                tool: "cc-notify".to_string(),
                session_id: None,
                project: None,
                cwd: None,
                tokens: None,
                metadata: serde_json::Value::Null,
                silent: false,
                last_assistant_message: None,
                model: None,
                source: None,
                reason: None,
                agent_type: None,
                title: None,
                stdin_message: None,
                channel_id: None,
                terminal: notify::TerminalContext::default(),
                terminal_jump_command: None,
            };
            notify::send_notification(&get_db_path(), &ctx)
        }
        Commands::NativeClickWorker {
            summary,
            body,
            jump_command,
            timeout_seconds,
        } => notify::run_native_click_worker(&summary, &body, &jump_command, timeout_seconds),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
