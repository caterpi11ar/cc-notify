use crate::db;
use std::path::Path;

/// Bundle identifier for CC Notify app.
/// Must match `identifier` in src-tauri/tauri.conf.json.
pub const BUNDLE_ID: &str = "com.ccnotify.desktop";

/// Rich notification context built from CLI args + stdin JSON
pub struct NotificationContext {
    pub event: String,
    pub notification_type: Option<String>,
    /// Explicit message from CLI --message flag. If set, used as-is (override).
    pub message: Option<String>,
    pub tool: String,
    pub session_id: Option<String>,
    pub project: Option<String>,
    pub cwd: Option<String>,
    pub tokens: Option<u64>,
    pub metadata: serde_json::Value,
    pub silent: bool,
    // Rich context from stdin JSON
    pub last_assistant_message: Option<String>,
    pub model: Option<String>,
    pub source: Option<String>,
    pub reason: Option<String>,
    pub agent_type: Option<String>,
    pub title: Option<String>,
    /// The `message` field from stdin JSON (kept separate from CLI --message).
    /// Used as notification body text for Notification events.
    pub stdin_message: Option<String>,
}

// ────────────────────────────────────────────────────────────
// Helper functions
// ────────────────────────────────────────────────────────────

/// Extract project name from a cwd path
pub fn project_name_from_cwd(cwd: &str) -> String {
    std::path::Path::new(cwd)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| cwd.to_string())
}

/// Truncate to max_len characters, appending "..." if truncated
fn truncate_message(msg: &str, max_len: usize) -> String {
    let chars: Vec<char> = msg.chars().collect();
    if chars.len() <= max_len {
        msg.to_string()
    } else {
        let truncated: String = chars[..max_len].iter().collect();
        format!("{}...", truncated)
    }
}

/// Return a Discord/embed color for an event type
fn event_color(event: &str) -> u64 {
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

/// Human-readable event display name
fn event_display_name(event_type_id: &str) -> &str {
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

/// Feishu card header color template
fn feishu_header_color(event_type_id: &str) -> &str {
    match event_type_id {
        "stop" | "subagent_stop" => "green",
        "session_start" => "blue",
        "session_end" | "test" => "grey",
        "error" => "red",
        e if e.starts_with("notification.") => "orange",
        _ => "blue",
    }
}

/// Build event header: "Task Completed - my-project"
fn build_event_header(event_type_id: &str, ctx: &NotificationContext) -> String {
    let display = ctx
        .title
        .as_deref()
        .unwrap_or(event_display_name(event_type_id));
    match &ctx.project {
        Some(p) if !p.is_empty() => format!("{} - {}", display, p),
        _ => display.to_string(),
    }
}

/// Build just the body portion (no header) from context fields
fn build_message_body(ctx: &NotificationContext, event_type_id: &str) -> String {
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

        "error" => format!("{} encountered an error", ctx.tool),
        "test" => "Test notification from CC Notify".to_string(),
        _ => format!("{}: {}", ctx.tool, ctx.event),
    }
}

/// Build the full default message: "[tool] Header\nBody"
fn build_default_message(ctx: &NotificationContext, event_type_id: &str) -> String {
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

/// Build context footer: "tool | project | model | timestamp"
fn build_context_footer(ctx: &NotificationContext) -> String {
    let mut parts = vec![ctx.tool.clone()];
    if let Some(project) = &ctx.project {
        if !project.is_empty() {
            parts.push(project.clone());
        }
    }
    if let Some(model) = &ctx.model {
        parts.push(model.clone());
    }
    parts.push(
        chrono::Local::now()
            .format("%Y-%m-%d %H:%M")
            .to_string(),
    );
    parts.join(" | ")
}

/// Simple HTML escape for Telegram messages
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ────────────────────────────────────────────────────────────
// Main notification pipeline
// ────────────────────────────────────────────────────────────

/// Send a notification through the cc-notify pipeline
pub fn send_notification(db_path: &Path, ctx: &NotificationContext) -> Result<(), String> {
    // Check kill switch
    let kill_switch_path = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("disabled");
    if kill_switch_path.exists() {
        if !ctx.silent {
            println!("Notifications disabled (kill switch active)");
        }
        return Ok(());
    }

    // Resolve the event type ID
    let event_type_id = match ctx.event.as_str() {
        "stop" => "stop".to_string(),
        "notification" => {
            format!(
                "notification.{}",
                ctx.notification_type.as_deref().unwrap_or("idle_prompt")
            )
        }
        "session-start" => "session_start".to_string(),
        "session-end" => "session_end".to_string(),
        "subagent-stop" => "subagent_stop".to_string(),
        "token-threshold" => "token_threshold".to_string(),
        "cost-threshold" => "cost_threshold".to_string(),
        "long-running" => "long_running".to_string(),
        "error" => "error".to_string(),
        "test" => "test".to_string(),
        "custom" => "custom".to_string(),
        other => other.to_string(),
    };

    // Build the notification message:
    // CLI --message overrides everything; otherwise build rich default
    let message_text = ctx
        .message
        .clone()
        .unwrap_or_else(|| build_default_message(ctx, &event_type_id));

    // Check rate limiting
    let rate_limit_file = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("last_notification");
    if let Ok(content) = std::fs::read_to_string(&rate_limit_file) {
        if let Ok(last_ts) = content.trim().parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            if now - last_ts < 10 && event_type_id == "stop" {
                if !ctx.silent {
                    println!("Rate limited (stop cooldown)");
                }
                return Ok(());
            }
        }
    }

    // Update rate limit timestamp
    let now = chrono::Utc::now().timestamp();
    if let Some(parent) = rate_limit_file.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&rate_limit_file, now.to_string()).ok();

    // Try to use database for routing, fall back to native notification
    if db_path.exists() {
        if let Ok(conn) = db::open_db_rw(db_path) {
            // Check if event type is enabled
            let enabled: bool = conn
                .query_row(
                    "SELECT enabled FROM event_types WHERE id = ?1",
                    rusqlite::params![&event_type_id],
                    |row| row.get(0),
                )
                .unwrap_or(true);

            if !enabled {
                if !ctx.silent {
                    println!("Event type '{}' is disabled", event_type_id);
                }
                return Ok(());
            }

            // Get routing for this event
            let routes = db::get_routing_for_event(&conn, &event_type_id).unwrap_or_default();

            if routes.is_empty() {
                // No routing configured, send native notification as fallback
                send_native_notification(&event_type_id, &message_text, ctx)?;
            } else {
                // Send to each routed channel
                let channels = db::get_enabled_channels(&conn).unwrap_or_default();
                for (channel_id, _priority) in &routes {
                    if let Some((_id, channel_type, config_str)) =
                        channels.iter().find(|(id, _, _)| id == channel_id)
                    {
                        let config: serde_json::Value =
                            serde_json::from_str(config_str).unwrap_or_default();
                        let result = send_to_channel(
                            channel_type,
                            &config,
                            &event_type_id,
                            &message_text,
                            ctx,
                        );

                        let (status, error_msg) = match &result {
                            Ok(_) => ("sent", None),
                            Err(e) => ("failed", Some(e.as_str())),
                        };

                        db::record_history(
                            &conn,
                            &event_type_id,
                            channel_id,
                            status,
                            &message_text,
                            error_msg,
                            &ctx.metadata,
                        )
                        .ok();

                        if !ctx.silent {
                            match result {
                                Ok(_) => println!("Sent to {} ({})", channel_id, channel_type),
                                Err(e) => eprintln!("Failed to send to {}: {}", channel_id, e),
                            }
                        }
                    }
                }
            }

            return Ok(());
        }
    }

    // No database available, send native notification directly
    send_native_notification(&event_type_id, &message_text, ctx)?;
    if !ctx.silent {
        println!("Notification sent (native fallback)");
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────
// Channel dispatch
// ────────────────────────────────────────────────────────────

/// Send notification to a specific channel type
fn send_to_channel(
    channel_type: &str,
    config: &serde_json::Value,
    event: &str,
    message: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    match channel_type {
        "native" => send_native_notification(event, message, ctx),
        "slack" => send_slack(config, event, message, ctx),
        "discord" => send_discord(config, event, message, ctx),
        "telegram" => send_telegram(config, event, message, ctx),
        "feishu" => {
            // Legacy: inject template and delegate to webhook
            let mut config = config.clone();
            config["template"] = serde_json::json!("feishu");
            send_webhook(&config, event, message, ctx)
        }
        "webhook" => send_webhook(config, event, message, ctx),
        "sound" => play_sound(config),
        "voice" => speak_notification(config, message),
        _ => Err(format!("Unknown channel type: {channel_type}")),
    }
}

// ────────────────────────────────────────────────────────────
// Native notification
// ────────────────────────────────────────────────────────────

fn send_native_notification(
    event: &str,
    message: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    let header = build_event_header(event, ctx);
    notify_rust::Notification::new()
        .summary(&format!("CC Notify: {}", header))
        .body(message)
        .appname("CC Notify")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show()
        .map_err(|e| format!("Failed to show notification: {e}"))?;
    Ok(())
}

// ────────────────────────────────────────────────────────────
// Slack — Block Kit layout
// ────────────────────────────────────────────────────────────

fn send_slack(
    config: &serde_json::Value,
    event: &str,
    message: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    let webhook_url = config["webhook_url"]
        .as_str()
        .ok_or("Missing webhook_url in Slack config")?;
    let mention = config["mention"].as_str().unwrap_or("");

    let header = build_event_header(event, ctx);
    let body = ctx
        .message
        .clone()
        .unwrap_or_else(|| build_message_body(ctx, event));
    let footer = build_context_footer(ctx);

    let header_text = if mention.is_empty() {
        header.clone()
    } else {
        format!("{} {}", mention, header)
    };

    let mut blocks = vec![
        // Header block
        serde_json::json!({
            "type": "header",
            "text": {
                "type": "plain_text",
                "text": truncate_message(&header_text, 150),
                "emoji": true
            }
        }),
    ];

    // Body section (skip if empty)
    if !body.is_empty() {
        blocks.push(serde_json::json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": body
            }
        }));
    }

    // Context footer
    blocks.push(serde_json::json!({
        "type": "context",
        "elements": [{
            "type": "mrkdwn",
            "text": footer
        }]
    }));

    let payload = serde_json::json!({
        "text": message,
        "blocks": blocks
    });

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .map_err(|e| format!("Slack request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Slack returned status {}", resp.status()));
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────
// Discord — Rich Embed
// ────────────────────────────────────────────────────────────

fn send_discord(
    config: &serde_json::Value,
    event: &str,
    message: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    let webhook_url = config["webhook_url"]
        .as_str()
        .ok_or("Missing webhook_url in Discord config")?;
    let username = config["username"].as_str().unwrap_or("CC Notify");

    let header = build_event_header(event, ctx);
    let body = ctx
        .message
        .clone()
        .unwrap_or_else(|| build_message_body(ctx, event));
    let color = event_color(event);

    let mut fields = Vec::new();
    if let Some(project) = &ctx.project {
        if !project.is_empty() {
            fields.push(serde_json::json!({"name": "Project", "value": project, "inline": true}));
        }
    }
    fields.push(serde_json::json!({"name": "Tool", "value": &ctx.tool, "inline": true}));
    if let Some(model) = &ctx.model {
        fields.push(serde_json::json!({"name": "Model", "value": model, "inline": true}));
    }

    let embed = serde_json::json!({
        "title": header,
        "description": if body.is_empty() { message.to_string() } else { body },
        "color": color,
        "fields": fields,
        "footer": {"text": "CC Notify"},
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let mut payload = serde_json::json!({
        "username": username,
        "embeds": [embed]
    });

    if let Some(url) = config["avatar_url"].as_str() {
        payload["avatar_url"] = serde_json::json!(url);
    }

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .map_err(|e| format!("Discord request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Discord returned status {}", resp.status()));
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────
// Telegram — HTML formatting
// ────────────────────────────────────────────────────────────

fn send_telegram(
    config: &serde_json::Value,
    event: &str,
    _message: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    let bot_token = config["bot_token"]
        .as_str()
        .ok_or("Missing bot_token in Telegram config")?;
    let chat_id = config["chat_id"]
        .as_str()
        .ok_or("Missing chat_id in Telegram config")?;
    let parse_mode = config["parse_mode"].as_str().unwrap_or("HTML");

    let header = build_event_header(event, ctx);
    let body = ctx
        .message
        .clone()
        .unwrap_or_else(|| build_message_body(ctx, event));
    let footer = build_context_footer(ctx);

    let text = if body.is_empty() {
        format!(
            "<b>{}</b>\n\n<i>{}</i>",
            html_escape(&header),
            html_escape(&footer),
        )
    } else {
        format!(
            "<b>{}</b>\n\n{}\n\n<i>{}</i>",
            html_escape(&header),
            html_escape(&body),
            html_escape(&footer),
        )
    };

    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": parse_mode,
    });

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| format!("Telegram request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Telegram returned status {}", resp.status()));
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────
// Webhook + Feishu
// ────────────────────────────────────────────────────────────

fn send_webhook(
    config: &serde_json::Value,
    event: &str,
    message: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    let template = config["template"].as_str().unwrap_or("generic");

    match template {
        "feishu" => send_feishu(config, event, message, ctx),
        _ => send_generic_webhook(config, event, message, ctx),
    }
}

/// Feishu — Rich interactive card
fn send_feishu(
    config: &serde_json::Value,
    event: &str,
    _message: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    let webhook_url = config["webhook_url"]
        .as_str()
        .ok_or("Missing webhook_url in Feishu config")?;

    let header = build_event_header(event, ctx);
    let body = ctx
        .message
        .clone()
        .unwrap_or_else(|| build_message_body(ctx, event));
    let color = feishu_header_color(event);
    let footer = build_context_footer(ctx);

    let mut elements = Vec::new();
    if !body.is_empty() {
        elements.push(serde_json::json!({"tag": "markdown", "content": body}));
    }
    elements.push(serde_json::json!({"tag": "hr"}));
    elements.push(serde_json::json!({
        "tag": "note",
        "elements": [{"tag": "plain_text", "content": footer}]
    }));

    let payload = serde_json::json!({
        "msg_type": "interactive",
        "card": {
            "header": {
                "title": {"tag": "plain_text", "content": header},
                "template": color
            },
            "elements": elements
        }
    });

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .map_err(|e| format!("Feishu request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Feishu returned status {}", resp.status()));
    }
    Ok(())
}

/// Generic webhook — Complete JSON payload
fn send_generic_webhook(
    config: &serde_json::Value,
    event: &str,
    message: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    let url = config["url"]
        .as_str()
        .ok_or("Missing url in webhook config")?;
    let method = config["method"].as_str().unwrap_or("POST");

    let payload = serde_json::json!({
        "event": event,
        "message": message,
        "project": ctx.project,
        "tool": ctx.tool,
        "model": ctx.model,
        "session_id": ctx.session_id,
        "cwd": ctx.cwd,
        "summary": ctx.last_assistant_message.as_deref().map(|s| truncate_message(s, 500)),
        "agent_type": ctx.agent_type,
        "source": ctx.source,
        "reason": ctx.reason,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let client = reqwest::blocking::Client::new();
    let req = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        _ => client.post(url).json(&payload),
    };

    // Add custom headers if configured
    let mut req = req;
    if let Some(headers) = config["headers"].as_object() {
        for (key, value) in headers {
            if let Some(v) = value.as_str() {
                req = req.header(key, v);
            }
        }
    }

    let resp = req
        .send()
        .map_err(|e| format!("Webhook request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Webhook returned status {}", resp.status()));
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────
// Sound + Voice (unchanged)
// ────────────────────────────────────────────────────────────

/// Play a sound notification
fn play_sound(config: &serde_json::Value) -> Result<(), String> {
    let sound_file = config["sound_file"].as_str();

    #[cfg(target_os = "macos")]
    {
        let file = sound_file.unwrap_or("/System/Library/Sounds/Glass.aiff");
        std::process::Command::new("afplay")
            .arg(file)
            .spawn()
            .map_err(|e| format!("Failed to play sound: {e}"))?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let file = sound_file.unwrap_or("/usr/share/sounds/freedesktop/stereo/complete.oga");
        // Try paplay -> aplay -> ffplay
        if std::process::Command::new("paplay")
            .arg(file)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        if std::process::Command::new("aplay")
            .arg(file)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        std::process::Command::new("ffplay")
            .args(["-nodisp", "-autoexit", file])
            .spawn()
            .map_err(|e| format!("Failed to play sound: {e}"))?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let file = sound_file.unwrap_or("C:\\Windows\\Media\\chimes.wav");
        std::process::Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "(New-Object Media.SoundPlayer '{}').PlaySync()",
                    file
                ),
            ])
            .spawn()
            .map_err(|e| format!("Failed to play sound: {e}"))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Unsupported platform for sound playback".to_string())
}

/// Speak a notification using text-to-speech (macOS only)
fn speak_notification(config: &serde_json::Value, message: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let voice = config["voice"].as_str().unwrap_or("Samantha");
        std::process::Command::new("say")
            .args(["-v", voice, message])
            .spawn()
            .map_err(|e| format!("Failed to speak notification: {e}"))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Voice notifications are only supported on macOS".to_string())
}
