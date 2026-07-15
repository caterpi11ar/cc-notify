use super::types::NotificationContext;

/// Extract project name from a cwd path.
pub fn project_name_from_cwd(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| cwd.to_string())
}

/// Truncate to max_len characters, appending "..." if truncated.
pub(crate) fn truncate_message(msg: &str, max_len: usize) -> String {
    let chars: Vec<char> = msg.chars().collect();
    if chars.len() <= max_len {
        msg.to_string()
    } else {
        let truncated: String = chars[..max_len].iter().collect();
        format!("{}...", truncated)
    }
}

/// Return a Discord/embed color for an event type.
pub(crate) fn event_color(event: &str) -> u64 {
    match event {
        "stop" | "subagent_stop" => 0x00C853,
        "session_start" => 0x2196F3,
        "session_end" => 0x9E9E9E,
        "error" => 0xFF1744,
        "test" => 0x5865F2,
        e if e.starts_with("notification.") => 0xFF9800,
        _ => 0x5865F2,
    }
}

/// Human-readable event display name.
pub(crate) fn event_display_name(event_type_id: &str) -> &str {
    match event_type_id {
        "stop" => "Task Completed",
        "subagent_stop" => "Subagent Completed",
        "session_start" => "Session Started",
        "session_end" => "Session Ended",
        "notification.idle_prompt" => "Waiting for Input",
        "notification.permission_prompt" => "Permission Request",
        "notification.auth_success" => "Auth Success",
        "notification.elicitation_dialog" => "MCP Input Required",
        "error" => "Error",
        "test" => "Test",
        _ => "Notification",
    }
}

/// Feishu card header color template.
pub(crate) fn feishu_header_color(event_type_id: &str) -> &str {
    match event_type_id {
        "stop" | "subagent_stop" => "green",
        "session_start" => "blue",
        "session_end" | "test" => "grey",
        "error" => "red",
        e if e.starts_with("notification.") => "orange",
        _ => "blue",
    }
}

/// Build event header: "Task Completed - my-project".
pub(crate) fn build_event_header(event_type_id: &str, ctx: &NotificationContext) -> String {
    let display = ctx
        .title
        .as_deref()
        .unwrap_or(event_display_name(event_type_id));
    match &ctx.project {
        Some(p) if !p.is_empty() => format!("{} - {}", display, p),
        _ => display.to_string(),
    }
}

/// Build just the body portion (no header) from context fields.
pub(crate) fn build_message_body(ctx: &NotificationContext, event_type_id: &str) -> String {
    match event_type_id {
        "stop" => ctx
            .last_assistant_message
            .as_deref()
            .map(|s| format!("Summary: {}", truncate_message(s, 200)))
            .unwrap_or_default(),

        "subagent_stop" => {
            let agent = ctx.agent_type.as_deref().unwrap_or("unknown");
            ctx.last_assistant_message
                .as_deref()
                .map(|s| format!("Agent: {} | Summary: {}", agent, truncate_message(s, 200)))
                .unwrap_or_else(|| format!("Subagent ({}) completed", agent))
        }

        e if e.starts_with("notification.") => ctx
            .stdin_message
            .as_deref()
            .map(|s| truncate_message(s, 200))
            .unwrap_or_else(|| match e {
                "notification.idle_prompt" => format!("{} is waiting for input", ctx.tool),
                "notification.permission_prompt" => {
                    format!("{} needs permission", ctx.tool)
                }
                "notification.auth_success" => "Authentication successful".to_string(),
                "notification.elicitation_dialog" => "MCP input required".to_string(),
                _ => format!("{} notification", ctx.tool),
            }),

        "session_start" => {
            let mut parts = Vec::new();
            if let Some(model) = &ctx.model {
                parts.push(format!("Model: {}", model));
            }
            if let Some(source) = &ctx.source {
                parts.push(format!("Source: {}", source));
            }
            if parts.is_empty() {
                String::new()
            } else {
                parts.join(" | ")
            }
        }

        "session_end" => ctx
            .reason
            .as_deref()
            .map(|r| format!("Reason: {}", r))
            .unwrap_or_default(),

        "token_threshold" => ctx
            .tokens
            .map(|tokens| format!("Token threshold reached: {}", tokens))
            .unwrap_or_else(|| "Token threshold reached".to_string()),

        "error" => format!("{} encountered an error", ctx.tool),
        "test" => "Test notification from CC Notify".to_string(),
        _ => format!("{}: {}", ctx.tool, ctx.event),
    }
}

/// Build the full default message: "[tool] Header\nBody".
pub(crate) fn build_default_message(ctx: &NotificationContext, event_type_id: &str) -> String {
    let display = ctx
        .title
        .as_deref()
        .unwrap_or(event_display_name(event_type_id));
    let project_suffix = ctx
        .project
        .as_deref()
        .filter(|p| !p.is_empty())
        .map(|p| format!(" - {}", p))
        .unwrap_or_default();
    let body = build_message_body(ctx, event_type_id);
    if body.is_empty() {
        format!("[{}] {}{}", ctx.tool, display, project_suffix)
    } else {
        format!("[{}] {}{}\n{}", ctx.tool, display, project_suffix, body)
    }
}

/// Build context footer: "tool | project | model | timestamp".
pub(crate) fn build_context_footer(ctx: &NotificationContext) -> String {
    let mut parts = vec![ctx.tool.clone()];
    if let Some(project) = &ctx.project {
        if !project.is_empty() {
            parts.push(project.clone());
        }
    }
    if let Some(model) = &ctx.model {
        parts.push(model.clone());
    }
    parts.push(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
    parts.join(" | ")
}

/// Simple HTML escape for Telegram messages.
pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
