use super::types::NotificationContext;
use std::io::Write;
use std::process::{Command, Stdio};

/// Shell-escape a value so it can be safely interpolated into command templates.
fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        "''".to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}

fn click_debug_enabled() -> bool {
    std::env::var_os("CC_NOTIFY_CLICK_DEBUG").is_some()
}

fn click_debug_log(message: &str) {
    if !click_debug_enabled() {
        return;
    }
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".cc-notify").join("click-debug.log");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(
                file,
                "[{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                message
            );
        }
    }
}

/// Parse macOS TERM/ITERM session locator formats like `w0t1p0` or `w0t1p0:...`.
fn parse_wtp_locator(raw: &str) -> Option<(u32, u32, Option<u32>)> {
    let token = raw.split(':').next()?.trim().to_ascii_lowercase();
    let rest = token.strip_prefix('w')?;
    let (window_str, rest) = rest.split_once('t')?;
    let (tab_str, pane_str) = if let Some((tab, pane)) = rest.split_once('p') {
        (tab, Some(pane))
    } else {
        (rest, None)
    };

    if window_str.is_empty() || tab_str.is_empty() || pane_str.is_some_and(|s| s.is_empty()) {
        return None;
    }

    let window = window_str.parse::<u32>().ok()?;
    let tab = tab_str.parse::<u32>().ok()?;
    let pane = pane_str.and_then(|s| s.parse::<u32>().ok());

    Some((window, tab, pane))
}

#[cfg(target_os = "macos")]
fn macos_app_name(program: &str) -> Option<&'static str> {
    let normalized = program.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if normalized.contains("iterm") {
        return Some("iTerm");
    }
    if normalized.contains("apple_terminal")
        || normalized == "terminal"
        || normalized == "terminal.app"
    {
        return Some("Terminal");
    }
    if normalized.contains("wezterm") {
        return Some("WezTerm");
    }
    if normalized.contains("kitty") {
        return Some("kitty");
    }
    if normalized.contains("warp") {
        return Some("Warp");
    }
    if normalized.contains("ghostty") {
        return Some("Ghostty");
    }
    if normalized.contains("alacritty") {
        return Some("Alacritty");
    }
    if normalized.contains("vscode")
        || normalized.contains("visual studio code")
        || normalized == "code"
    {
        return Some("Visual Studio Code");
    }
    if normalized.contains("cursor") {
        return Some("Cursor");
    }
    None
}

#[cfg(target_os = "macos")]
fn fallback_activate_terminal_program(program: &str) -> Option<String> {
    if let Some(app) = macos_app_name(program) {
        return Some(format!("open -a {}", shell_escape(app)));
    }

    let trimmed = program.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(format!("open -a {}", shell_escape(trimmed)))
    }
}

#[cfg(target_os = "macos")]
fn macos_terminal_jump_command(ctx: &NotificationContext, program: &str) -> Option<String> {
    let normalized = program.trim().to_ascii_lowercase();
    if normalized.contains("vscode")
        || normalized.contains("visual studio code")
        || normalized == "code"
    {
        let code_cli = "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code";
        let code_cli_escaped = shell_escape(code_cli);
        if let Some(cwd) = ctx.cwd.as_deref().filter(|v| !v.trim().is_empty()) {
            let cwd_escaped = shell_escape(cwd);
            return Some(format!(
                "if [ -x {code_cli_escaped} ]; then {code_cli_escaped} --reuse-window {cwd_escaped}; else open -a 'Visual Studio Code' --args --reuse-window {cwd_escaped}; fi"
            ));
        }
        return Some(format!(
            "if [ -x {code_cli_escaped} ]; then {code_cli_escaped} --reuse-window; else open -a 'Visual Studio Code'; fi"
        ));
    }

    let session_locator = ctx
        .terminal
        .terminal_id
        .as_deref()
        .or(ctx.terminal.tab_id.as_deref());
    let parsed = session_locator.and_then(parse_wtp_locator);

    if normalized.contains("iterm") {
        if let Some((window, tab, _)) = parsed {
            let window = window + 1;
            let tab = tab + 1;
            return Some(format!(
                "osascript -e 'tell application \"iTerm\" to activate' -e 'tell application \"iTerm\" to set current window to window {window}' -e 'tell application \"iTerm\" to tell current window to set current tab to tab {tab}' || osascript -e 'tell application \"iTerm2\" to activate' -e 'tell application \"iTerm2\" to set current window to window {window}' -e 'tell application \"iTerm2\" to tell current window to set current tab to tab {tab}' || open -a 'iTerm'"
            ));
        }
        return fallback_activate_terminal_program(program);
    }

    if normalized.contains("apple_terminal")
        || normalized == "terminal"
        || normalized == "terminal.app"
    {
        if let Some((window, tab, _)) = parsed {
            let window = window + 1;
            let tab = tab + 1;
            return Some(format!(
                "osascript -e 'tell application \"Terminal\" to activate' -e 'tell application \"Terminal\" to set frontmost to true' -e 'tell application \"Terminal\" to set selected of tab {tab} of window {window} to true' || osascript -e 'tell application \"Terminal\" to activate'"
            ));
        }
        return Some("osascript -e 'tell application \"Terminal\" to activate'".to_string());
    }

    fallback_activate_terminal_program(program)
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

    if let Some(pane_id) = ctx
        .terminal
        .pane_id
        .as_deref()
        .or(ctx.terminal.terminal_id.as_deref())
    {
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

    if let Some(window_id) = ctx
        .terminal
        .window_id
        .as_deref()
        .or(ctx.terminal.terminal_id.as_deref())
    {
        if program.contains("kitty") || std::env::var_os("KITTY_WINDOW_ID").is_some() {
            return Some(format!(
                "kitty @ focus-window --match id:{}",
                shell_escape(window_id)
            ));
        }
    }

    #[cfg(target_os = "macos")]
    if let Some(command) = macos_terminal_jump_command(ctx, &program) {
        return Some(command);
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
    } else {
        default_terminal_jump_command(ctx)
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
        let mut cmd = Command::new("/bin/sh");
        cmd.args(["-lc", command]);
        cmd
    };

    click_debug_log(&format!("execute jump command: {:?}", command));

    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| {
            click_debug_log(&format!("jump command run failed: {}", e));
            format!("Failed to run jump command: {e}")
        })?;

    click_debug_log(&format!("jump command exit status={:?}", status.code()));
    if !status.success() {
        return Err(format!("Jump command exited with status {:?}", status.code()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::resolve_terminal_jump_command;
    use crate::notify::{NotificationContext, TerminalContext};

    fn make_ctx(
        terminal: TerminalContext,
        terminal_jump_command: Option<&str>,
    ) -> NotificationContext {
        NotificationContext {
            event: "stop".to_string(),
            notification_type: None,
            message: None,
            tool: "codex".to_string(),
            session_id: Some("s-1".to_string()),
            project: Some("demo".to_string()),
            cwd: Some("/tmp/demo".to_string()),
            tokens: None,
            metadata: serde_json::Value::Null,
            silent: true,
            last_assistant_message: None,
            model: None,
            source: None,
            reason: None,
            agent_type: None,
            title: None,
            stdin_message: None,
            channel_id: None,
            terminal,
            terminal_jump_command: terminal_jump_command.map(str::to_string),
        }
    }

    #[test]
    fn explicit_template_uses_shell_escaped_values() {
        let ctx = make_ctx(
            TerminalContext {
                program: Some("WezTerm".to_string()),
                terminal_id: Some("pane'42".to_string()),
                window_id: None,
                tab_id: None,
                pane_id: None,
            },
            Some("echo {terminal_id} {project}"),
        );

        let cmd = resolve_terminal_jump_command(&ctx).expect("command");
        assert_eq!(cmd, "echo 'pane'\"'\"'42' 'demo'");
    }

    #[test]
    fn wezterm_uses_terminal_id_when_pane_id_missing() {
        let ctx = make_ctx(
            TerminalContext {
                program: Some("WezTerm".to_string()),
                terminal_id: Some("123".to_string()),
                window_id: None,
                tab_id: None,
                pane_id: None,
            },
            None,
        );

        let cmd = resolve_terminal_jump_command(&ctx).expect("command");
        assert_eq!(cmd, "wezterm cli activate-pane --pane-id '123'");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn iterm_session_id_generates_window_tab_focus_command() {
        let ctx = make_ctx(
            TerminalContext {
                program: Some("iTerm.app".to_string()),
                terminal_id: Some("w1t2p0:abc".to_string()),
                window_id: None,
                tab_id: None,
                pane_id: None,
            },
            None,
        );

        let cmd = resolve_terminal_jump_command(&ctx).expect("command");
        assert!(cmd.contains("iTerm"));
        assert!(cmd.contains("window 2"));
        assert!(cmd.contains("tab 3"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn terminal_session_id_generates_window_tab_focus_command() {
        let ctx = make_ctx(
            TerminalContext {
                program: Some("Apple_Terminal".to_string()),
                terminal_id: Some("w0t1p0:abc".to_string()),
                window_id: None,
                tab_id: None,
                pane_id: None,
            },
            None,
        );

        let cmd = resolve_terminal_jump_command(&ctx).expect("command");
        assert!(cmd.contains("application \"Terminal\""));
        assert!(cmd.contains("tab 2 of window 1"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_program_only_still_has_fallback_command() {
        let ctx = make_ctx(
            TerminalContext {
                program: Some("iTerm.app".to_string()),
                terminal_id: None,
                window_id: None,
                tab_id: None,
                pane_id: None,
            },
            None,
        );

        let cmd = resolve_terminal_jump_command(&ctx).expect("command");
        assert_eq!(cmd, "open -a 'iTerm'");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn vscode_program_reuses_window_for_cwd() {
        let mut ctx = make_ctx(
            TerminalContext {
                program: Some("vscode".to_string()),
                terminal_id: None,
                window_id: None,
                tab_id: None,
                pane_id: None,
            },
            None,
        );
        ctx.cwd = Some("/tmp/demo".to_string());

        let cmd = resolve_terminal_jump_command(&ctx).expect("command");
        assert!(cmd.contains("Visual Studio Code"));
        assert!(cmd.contains("--reuse-window '/tmp/demo'"));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_program_only_does_not_generate_command() {
        let ctx = make_ctx(
            TerminalContext {
                program: Some("iTerm.app".to_string()),
                terminal_id: None,
                window_id: None,
                tab_id: None,
                pane_id: None,
            },
            None,
        );

        assert!(resolve_terminal_jump_command(&ctx).is_none());
    }
}
