use crate::db;
use std::path::Path;

/// Bundle identifier for CC Notify app.
/// Must match `identifier` in src-tauri/tauri.conf.json.
pub const BUNDLE_ID: &str = "com.ccnotify.desktop";

/// Send a notification through the cc-notify pipeline
pub fn send_notification(
    db_path: &Path,
    event: &str,
    notification_type: Option<&str>,
    message: Option<&str>,
    tool: &str,
    session_id: Option<&str>,
    project: Option<&str>,
    tokens: Option<u64>,
    metadata: &serde_json::Value,
    silent: bool,
) -> Result<(), String> {
    // Check kill switch
    let kill_switch_path = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("disabled");
    if kill_switch_path.exists() {
        if !silent {
            println!("Notifications disabled (kill switch active)");
        }
        return Ok(());
    }

    // Resolve the event type ID
    let event_type_id = match event {
        "stop" => "stop".to_string(),
        "notification" => {
            format!("notification.{}", notification_type.unwrap_or("idle_prompt"))
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

    // Build the notification message
    let default_message = match event_type_id.as_str() {
        "stop" => format!("{} task completed", tool),
        "notification.idle_prompt" => format!("{} is waiting for input", tool),
        "notification.permission_prompt" => format!("{} needs permission", tool),
        "session_start" => format!("{} session started", tool),
        "session_end" => format!("{} session ended", tool),
        "error" => format!("{} encountered an error", tool),
        "test" => "Test notification from CC Notify".to_string(),
        _ => format!("{}: {}", tool, event),
    };
    let message_text = message.unwrap_or(&default_message);

    // Check rate limiting
    let rate_limit_file = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("last_notification");
    if let Ok(content) = std::fs::read_to_string(&rate_limit_file) {
        if let Ok(last_ts) = content.trim().parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            if now - last_ts < 10 && event_type_id == "stop" {
                if !silent {
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
                if !silent {
                    println!("Event type '{}' is disabled", event_type_id);
                }
                return Ok(());
            }

            // Get routing for this event
            let routes = db::get_routing_for_event(&conn, &event_type_id).unwrap_or_default();

            if routes.is_empty() {
                // No routing configured, send native notification as fallback
                send_native_notification(&event_type_id, message_text)?;
            } else {
                // Send to each routed channel
                let channels = db::get_enabled_channels(&conn).unwrap_or_default();
                for (channel_id, priority) in &routes {
                    if let Some((_id, channel_type, config_str)) =
                        channels.iter().find(|(id, _, _)| id == channel_id)
                    {
                        let config: serde_json::Value =
                            serde_json::from_str(config_str).unwrap_or_default();
                        let result = send_to_channel(channel_type, &config, &event_type_id, message_text);

                        let (status, error_msg) = match &result {
                            Ok(_) => ("sent", None),
                            Err(e) => ("failed", Some(e.as_str())),
                        };

                        db::record_history(
                            &conn,
                            &event_type_id,
                            channel_id,
                            status,
                            message_text,
                            error_msg,
                            metadata,
                        )
                        .ok();

                        if !silent {
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
    send_native_notification(&event_type_id, message_text)?;
    if !silent {
        println!("Notification sent (native fallback)");
    }
    Ok(())
}

/// Send a native system notification
fn send_native_notification(event: &str, message: &str) -> Result<(), String> {
    notify_rust::Notification::new()
        .summary(&format!("CC Notify: {}", event))
        .body(message)
        .appname("CC Notify")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show()
        .map_err(|e| format!("Failed to show notification: {e}"))?;
    Ok(())
}

/// Send notification to a specific channel type
fn send_to_channel(
    channel_type: &str,
    config: &serde_json::Value,
    event: &str,
    message: &str,
) -> Result<(), String> {
    match channel_type {
        "native" => send_native_notification(event, message),
        "slack" => send_slack(config, event, message),
        "discord" => send_discord(config, event, message),
        "telegram" => send_telegram(config, event, message),
        "feishu" => {
            // Legacy: inject template and delegate to webhook
            let mut config = config.clone();
            config["template"] = serde_json::json!("feishu");
            send_webhook(&config, event, message)
        }
        "webhook" => send_webhook(config, event, message),
        "sound" => play_sound(config),
        "voice" => speak_notification(config, message),
        _ => Err(format!("Unknown channel type: {channel_type}")),
    }
}

/// Send Slack webhook notification
fn send_slack(config: &serde_json::Value, event: &str, message: &str) -> Result<(), String> {
    let webhook_url = config["webhook_url"]
        .as_str()
        .ok_or("Missing webhook_url in Slack config")?;

    let payload = serde_json::json!({
        "text": format!("*{}*\n{}", event, message),
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

/// Send Discord webhook notification
fn send_discord(config: &serde_json::Value, event: &str, message: &str) -> Result<(), String> {
    let webhook_url = config["webhook_url"]
        .as_str()
        .ok_or("Missing webhook_url in Discord config")?;

    let username = config["username"].as_str().unwrap_or("CC Notify");
    let embed_color = config["embed_color"].as_u64().unwrap_or(0x5865F2);

    let payload = serde_json::json!({
        "username": username,
        "embeds": [{
            "title": event,
            "description": message,
            "color": embed_color,
        }],
    });

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

/// Send Telegram bot notification
fn send_telegram(config: &serde_json::Value, event: &str, message: &str) -> Result<(), String> {
    let bot_token = config["bot_token"]
        .as_str()
        .ok_or("Missing bot_token in Telegram config")?;
    let chat_id = config["chat_id"]
        .as_str()
        .ok_or("Missing chat_id in Telegram config")?;
    let parse_mode = config["parse_mode"].as_str().unwrap_or("HTML");

    let text = format!("<b>{}</b>\n{}", event, message);
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

/// Send generic HTTP webhook notification (with template support)
fn send_webhook(config: &serde_json::Value, event: &str, message: &str) -> Result<(), String> {
    let template = config["template"].as_str().unwrap_or("generic");

    match template {
        "feishu" => {
            let webhook_url = config["webhook_url"]
                .as_str()
                .ok_or("Missing webhook_url in Feishu config")?;

            let payload = serde_json::json!({
                "msg_type": "interactive",
                "card": {
                    "header": {
                        "title": {
                            "tag": "plain_text",
                            "content": format!("CC Notify: {}", event)
                        },
                        "template": "blue"
                    },
                    "elements": [
                        {
                            "tag": "markdown",
                            "content": message
                        }
                    ]
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
        _ => {
            let url = config["url"]
                .as_str()
                .ok_or("Missing url in webhook config")?;
            let method = config["method"].as_str().unwrap_or("POST");

            let payload = serde_json::json!({
                "event": event,
                "message": message,
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

            let resp = req.send().map_err(|e| format!("Webhook request failed: {e}"))?;

            if !resp.status().is_success() {
                return Err(format!("Webhook returned status {}", resp.status()));
            }
            Ok(())
        }
    }
}

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
