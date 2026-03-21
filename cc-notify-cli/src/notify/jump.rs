use super::types::NotificationContext;
use std::process::{Command, Stdio};

/// Shell-escape a value so it can be safely interpolated into command templates.
fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        "''".to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

/// Render a terminal jump command template with escaped placeholders.
fn render_terminal_jump_command(template: &str, ctx: &NotificationContext) -> String {
    let mut command = template.to_string();
    let replacements = [
        (
            "{terminal_program}",
            ctx.terminal.program.as_deref().unwrap_or(""),
        ),
        (
            "{terminal_id}",
            ctx.terminal.terminal_id.as_deref().unwrap_or(""),
        ),
        (
            "{terminal_window_id}",
            ctx.terminal.window_id.as_deref().unwrap_or(""),
        ),
        (
            "{terminal_tab_id}",
            ctx.terminal.tab_id.as_deref().unwrap_or(""),
        ),
        (
            "{terminal_pane_id}",
            ctx.terminal.pane_id.as_deref().unwrap_or(""),
        ),
        ("{session_id}", ctx.session_id.as_deref().unwrap_or("")),
        ("{project}", ctx.project.as_deref().unwrap_or("")),
        ("{cwd}", ctx.cwd.as_deref().unwrap_or("")),
        ("{tool}", ctx.tool.as_str()),
        ("{event}", ctx.event.as_str()),
    ];

    for (placeholder, value) in replacements {
        command = command.replace(placeholder, &shell_escape(value));
    }
    command
}

/// Best-effort default jump commands for common terminal IDs.
fn default_terminal_jump_command(ctx: &NotificationContext) -> Option<String> {
    let program = ctx
        .terminal
        .program
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();

    if let Some(pane_id) = ctx.terminal.pane_id.as_deref() {
        if program.contains("wezterm") || std::env::var_os("WEZTERM_PANE").is_some() {
            return Some(format!(
                "wezterm cli activate-pane --pane-id {}",
                shell_escape(pane_id)
            ));
        }

        if program.contains("tmux") || std::env::var_os("TMUX").is_some() {
            let pane = shell_escape(pane_id);
            return Some(format!(
                "tmux select-pane -t {pane} && tmux select-window -t {pane}"
            ));
        }
    }

    if let Some(window_id) = ctx.terminal.window_id.as_deref() {
        if program.contains("kitty") || std::env::var_os("KITTY_WINDOW_ID").is_some() {
            return Some(format!(
                "kitty @ focus-window --match id:{}",
                shell_escape(window_id)
            ));
        }
    }

    None
}

/// Resolve the command executed when a notification is clicked.
pub(crate) fn resolve_terminal_jump_command(ctx: &NotificationContext) -> Option<String> {
    if let Some(template) = ctx.terminal_jump_command.as_deref() {
        let rendered = render_terminal_jump_command(template, ctx);
        if rendered.trim().is_empty() {
            None
        } else {
            Some(rendered)
        }
    } else if ctx.terminal.has_any_identifier() {
        default_terminal_jump_command(ctx)
    } else {
        None
    }
}

/// Execute a jump command in a detached shell.
pub(crate) fn execute_jump_command(command: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command]);
        cmd
    };

    #[cfg(not(target_os = "windows"))]
    let mut cmd = {
        let mut cmd = Command::new("sh");
        cmd.args(["-lc", command]);
        cmd
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to run jump command: {e}"))?;

    Ok(())
}
