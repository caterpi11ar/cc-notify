mod channels;
mod jump;
mod message;
mod native_click;
mod types;

pub use message::project_name_from_cwd;
pub use native_click::run_native_click_worker;
pub use types::{NotificationContext, TerminalContext, BUNDLE_ID};

use crate::db;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

fn kill_switch_path(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("disabled")
}

fn rate_limit_path(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("last_notification")
}

fn resolve_event_type_id(ctx: &NotificationContext) -> String {
    match ctx.event.as_str() {
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
    }
}

fn resolve_message_text(ctx: &NotificationContext, event_type_id: &str) -> String {
    ctx.message
        .clone()
        .unwrap_or_else(|| message::build_default_message(ctx, event_type_id))
}

fn should_rate_limit_stop(rate_limit_file: &Path, event_type_id: &str) -> bool {
    if event_type_id != "stop" {
        return false;
    }

    std::fs::read_to_string(rate_limit_file)
        .ok()
        .and_then(|content| content.trim().parse::<i64>().ok())
        .is_some_and(|last_ts| chrono::Utc::now().timestamp() - last_ts < 10)
}

fn update_rate_limit_timestamp(rate_limit_file: &Path) {
    let now = chrono::Utc::now().timestamp();
    if let Some(parent) = rate_limit_file.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(rate_limit_file, now.to_string()).ok();
}

fn is_event_enabled(conn: &Connection, event_type_id: &str) -> bool {
    conn.query_row(
        "SELECT enabled FROM event_types WHERE id = ?1",
        rusqlite::params![event_type_id],
        |row| row.get(0),
    )
    .unwrap_or(true)
}

fn record_channel_result(
    conn: &Connection,
    event_type_id: &str,
    channel_id: &str,
    message_text: &str,
    metadata: &serde_json::Value,
    result: &Result<(), String>,
) {
    let (status, error_msg) = match result {
        Ok(_) => ("sent", None),
        Err(e) => ("failed", Some(e.as_str())),
    };
    db::record_history(
        conn,
        event_type_id,
        channel_id,
        status,
        message_text,
        error_msg,
        metadata,
    )
    .ok();
}

fn send_to_explicit_channel(
    conn: &Connection,
    event_type_id: &str,
    message_text: &str,
    ctx: &NotificationContext,
) -> Option<Result<(), String>> {
    let target_id = ctx.channel_id.as_deref()?;

    let channel: Option<(String, String, String)> = conn
        .query_row(
            "SELECT id, channel_type, config FROM channels WHERE id = ?1",
            rusqlite::params![target_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    let (channel_type, result) = if let Some((_id, channel_type, config_str)) = channel {
        let config: serde_json::Value = serde_json::from_str(&config_str).unwrap_or_default();
        let result =
            channels::send_to_channel(&channel_type, &config, event_type_id, message_text, ctx);
        (channel_type, result)
    } else {
        (
            "unknown".to_string(),
            Err(format!("Channel not found: {}", target_id)),
        )
    };

    record_channel_result(
        conn,
        event_type_id,
        target_id,
        message_text,
        &ctx.metadata,
        &result,
    );

    if !ctx.silent {
        match &result {
            Ok(_) => println!("Sent to {} ({})", target_id, channel_type),
            Err(e) => eprintln!("Failed to send to {}: {}", target_id, e),
        }
    }

    Some(result)
}

fn send_to_routed_channels(
    conn: &Connection,
    event_type_id: &str,
    message_text: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    let routes = db::get_routing_for_event(conn, event_type_id).unwrap_or_default();

    if routes.is_empty() {
        return native_click::send_native_notification(event_type_id, message_text, ctx);
    }

    let channels = db::get_enabled_channels(conn).unwrap_or_default();
    for (channel_id, _priority) in &routes {
        if let Some((_id, channel_type, config_str)) =
            channels.iter().find(|(id, _, _)| id == channel_id)
        {
            let config: serde_json::Value = serde_json::from_str(config_str).unwrap_or_default();
            let result =
                channels::send_to_channel(channel_type, &config, event_type_id, message_text, ctx);
            record_channel_result(
                conn,
                event_type_id,
                channel_id,
                message_text,
                &ctx.metadata,
                &result,
            );

            if !ctx.silent {
                match result {
                    Ok(_) => println!("Sent to {} ({})", channel_id, channel_type),
                    Err(e) => eprintln!("Failed to send to {}: {}", channel_id, e),
                }
            }
        }
    }
    Ok(())
}

/// Send a notification through the cc-notify pipeline.
pub fn send_notification(db_path: &Path, ctx: &NotificationContext) -> Result<(), String> {
    // Check kill switch
    let kill_switch_path = kill_switch_path(db_path);
    if kill_switch_path.exists() {
        if !ctx.silent {
            println!("Notifications disabled (kill switch active)");
        }
        return Ok(());
    }

    let event_type_id = resolve_event_type_id(ctx);
    let message_text = resolve_message_text(ctx, &event_type_id);

    // Check rate limiting
    let rate_limit_file = rate_limit_path(db_path);
    if should_rate_limit_stop(&rate_limit_file, &event_type_id) {
        if !ctx.silent {
            println!("Rate limited (stop cooldown)");
        }
        return Ok(());
    }

    // Update rate limit timestamp
    update_rate_limit_timestamp(&rate_limit_file);

    // Try to use database for routing, fall back to native notification
    if db_path.exists() {
        if let Ok(conn) = db::open_db_rw(db_path) {
            // If --channel-id is specified, send directly to that channel (bypass routing)
            if let Some(result) =
                send_to_explicit_channel(&conn, &event_type_id, &message_text, ctx)
            {
                return result.map(|_| ());
            }

            // Check if event type is enabled
            if !is_event_enabled(&conn, &event_type_id) {
                if !ctx.silent {
                    println!("Event type '{}' is disabled", event_type_id);
                }
                return Ok(());
            }

            send_to_routed_channels(&conn, &event_type_id, &message_text, ctx)?;
            return Ok(());
        }
    }

    // No database available, send native notification directly
    native_click::send_native_notification(&event_type_id, &message_text, ctx)?;
    if !ctx.silent {
        println!("Notification sent (native fallback)");
    }
    Ok(())
}
