use super::message::{
    build_context_footer, build_event_header, build_message_body, event_color, feishu_header_color,
    html_escape, truncate_message,
};
use super::native_click::send_native_notification;
use super::types::NotificationContext;
use std::path::Path;

/// Send notification to a specific channel type.
pub(crate) fn send_to_channel(
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
        "voice" => play_notification_sound(config),
        _ => Err(format!("Unknown channel type: {channel_type}")),
    }
}

/// Slack — Block Kit layout.
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

/// Discord — Rich Embed.
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

/// Telegram — HTML formatting.
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

/// Webhook + Feishu.
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

/// Feishu — Rich interactive card.
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

/// Generic webhook — Complete JSON payload.
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
        "tokens": ctx.tokens,
        "cwd": ctx.cwd,
        "summary": ctx.last_assistant_message.as_deref().map(|s| truncate_message(s, 500)),
        "agent_type": ctx.agent_type,
        "source": ctx.source,
        "reason": ctx.reason,
        "terminal": ctx.terminal,
        "metadata": ctx.metadata,
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

/// Play a notification sound file.
/// Reads `config["sound_file"]` (default "default.mp3") and `config["volume"]`.
/// Relative paths are resolved under `~/.cc-notify/sounds/`.
fn play_notification_sound(config: &serde_json::Value) -> Result<(), String> {
    let sound_file = config["sound_file"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or("default.mp3");

    let volume = config["volume"].as_f64();

    // Resolve path: absolute paths used as-is, otherwise look in ~/.cc-notify/sounds/
    let file_path = if Path::new(sound_file).is_absolute() {
        std::path::PathBuf::from(sound_file)
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".cc-notify")
            .join("sounds")
            .join(sound_file)
    };

    if !file_path.exists() {
        return Err(format!("Sound file not found: {}", file_path.display()));
    }

    let file = file_path.to_string_lossy();

    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("afplay");
        if let Some(v) = volume {
            cmd.args(["-v", &v.to_string()]);
        }
        cmd.arg(file.as_ref())
            .spawn()
            .map_err(|e| format!("Failed to play sound: {e}"))?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let pa_volume = volume.map(|v| ((v * 65536.0) as u32).to_string());
        let mut cmd = std::process::Command::new("paplay");
        if let Some(ref vol) = pa_volume {
            cmd.args(["--volume", vol]);
        }
        if cmd.arg(file.as_ref()).spawn().is_ok() {
            return Ok(());
        }
        if std::process::Command::new("aplay")
            .arg(file.as_ref())
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
        std::process::Command::new("ffplay")
            .args(["-nodisp", "-autoexit", file.as_ref()])
            .spawn()
            .map_err(|e| format!("Failed to play sound: {e}"))?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("powershell")
            .args([
                "-Command",
                &format!("(New-Object Media.SoundPlayer '{}').PlaySync()", file),
            ])
            .spawn()
            .map_err(|e| format!("Failed to play sound: {e}"))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Unsupported platform for sound playback".to_string())
}
