use super::jump::{execute_jump_command, resolve_terminal_jump_command};
use super::message::build_event_header;
use super::types::{NotificationContext, BUNDLE_ID};
use std::process::{Command, Stdio};

#[cfg(all(unix, not(target_os = "macos")))]
fn send_native_notification_with_action(
    summary: &str,
    body: &str,
    jump_command: &str,
) -> Result<(), String> {
    let mut notification = notify_rust::Notification::new();
    notification
        .summary(summary)
        .body(body)
        .appname("CC Notify")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .action("open-terminal", "Open Terminal");

    let handle = notification
        .show()
        .map_err(|e| format!("Failed to show notification: {e}"))?;

    let command = jump_command.to_string();
    handle.wait_for_action(move |action| {
        if action == "default" || action == "open-terminal" {
            let _ = execute_jump_command(&command);
        }
    });

    Ok(())
}

#[cfg(unix)]
fn spawn_native_click_worker(summary: &str, body: &str, jump_command: &str) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Failed to locate cc-notify executable: {e}"))?;
    Command::new(exe)
        .arg("__native-click-worker")
        .arg("--summary")
        .arg(summary)
        .arg("--body")
        .arg(body)
        .arg("--jump-command")
        .arg(jump_command)
        .arg("--timeout-seconds")
        .arg("20")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn click worker: {e}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn run_native_click_worker(
    summary: &str,
    body: &str,
    jump_command: &str,
    timeout_seconds: u64,
) -> Result<(), String> {
    use mac_notification_sys::{Notification, NotificationResponse};
    use std::sync::mpsc;
    use std::time::Duration;

    let _ = mac_notification_sys::set_application(BUNDLE_ID);

    let (tx, rx) = mpsc::channel();
    let summary = summary.to_string();
    let body = body.to_string();

    std::thread::spawn(move || {
        let result = Notification::new()
            .title(&summary)
            .message(&body)
            .wait_for_click(true)
            .send()
            .map_err(|e| format!("Failed to show notification: {e}"));
        let _ = tx.send(result);
    });

    let response = if timeout_seconds == 0 {
        rx.recv()
            .map_err(|_| "Native click worker channel closed unexpectedly".to_string())?
    } else {
        match rx.recv_timeout(Duration::from_secs(timeout_seconds)) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => return Ok(()),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("Native click worker channel closed unexpectedly".to_string());
            }
        }
    }?;

    if matches!(
        response,
        NotificationResponse::Click | NotificationResponse::ActionButton(_)
    ) {
        execute_jump_command(jump_command)?;
    }

    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn run_native_click_worker(
    summary: &str,
    body: &str,
    jump_command: &str,
    _timeout_seconds: u64,
) -> Result<(), String> {
    send_native_notification_with_action(summary, body, jump_command)
}

#[cfg(not(unix))]
pub fn run_native_click_worker(
    _summary: &str,
    _body: &str,
    _jump_command: &str,
    _timeout_seconds: u64,
) -> Result<(), String> {
    Err("Native click worker is only supported on Unix-like systems".to_string())
}

pub(crate) fn send_native_notification(
    event: &str,
    message: &str,
    ctx: &NotificationContext,
) -> Result<(), String> {
    let header = build_event_header(event, ctx);
    let summary = format!("CC Notify: {}", header);

    if let Some(jump_command) = resolve_terminal_jump_command(ctx) {
        #[cfg(unix)]
        {
            if let Err(err) = spawn_native_click_worker(&summary, message, &jump_command) {
                if !ctx.silent {
                    eprintln!("Failed to start native click worker: {}", err);
                }
            } else {
                return Ok(());
            }
        }
    }

    notify_rust::Notification::new()
        .summary(&summary)
        .body(message)
        .appname("CC Notify")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show()
        .map_err(|e| format!("Failed to show notification: {e}"))?;
    Ok(())
}
