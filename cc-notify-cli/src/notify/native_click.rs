use super::jump::{execute_jump_command, resolve_terminal_jump_command};
use super::message::build_event_header;
use super::types::{NotificationContext, BUNDLE_ID};
use std::io::Write;
use std::process::{Command, Stdio};

const DEFAULT_CLICK_WORKER_TIMEOUT_SECONDS: u64 = 180;

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

fn click_worker_timeout_seconds() -> u64 {
    std::env::var("CC_NOTIFY_NATIVE_CLICK_TIMEOUT_SECONDS")
        .ok()
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_CLICK_WORKER_TIMEOUT_SECONDS)
}

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

#[cfg(target_os = "macos")]
fn spawn_native_click_worker(summary: &str, body: &str, jump_command: &str) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Failed to locate cc-notify executable: {e}"))?;
    let timeout_seconds = click_worker_timeout_seconds();
    let label = format!(
        "com.ccnotify.native-click-worker.{}.{}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis()
    );
    click_debug_log(&format!(
        "spawn worker via launchctl label={} timeout={} summary={:?} jump_command={:?}",
        label, timeout_seconds, summary, jump_command
    ));

    let mut cmd = Command::new("launchctl");
    cmd.arg("submit")
        .arg("-l")
        .arg(&label)
        .arg("--");

    if click_debug_enabled() {
        cmd.arg("/usr/bin/env").arg("CC_NOTIFY_CLICK_DEBUG=1");
    }

    cmd.arg(exe)
        .arg("__native-click-worker")
        .arg("--summary")
        .arg(summary)
        .arg("--body")
        .arg(body)
        .arg("--jump-command")
        .arg(jump_command)
        .arg("--timeout-seconds")
        .arg(timeout_seconds.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("Failed to submit click worker to launchctl: {e}"))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!(
                    "launchctl submit failed with status code {:?}",
                    status.code()
                ))
            }
        })?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn spawn_native_click_worker(summary: &str, body: &str, jump_command: &str) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Failed to locate cc-notify executable: {e}"))?;
    let timeout_seconds = click_worker_timeout_seconds();
    click_debug_log(&format!(
        "spawn worker timeout={} summary={:?} jump_command={:?}",
        timeout_seconds, summary, jump_command
    ));
    Command::new(exe)
        .arg("__native-click-worker")
        .arg("--summary")
        .arg(summary)
        .arg("--body")
        .arg(body)
        .arg("--jump-command")
        .arg(jump_command)
        .arg("--timeout-seconds")
        .arg(timeout_seconds.to_string())
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
    use mac_notification_sys::{MainButton, Notification, NotificationResponse};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use std::time::Duration;

    let _ = mac_notification_sys::set_application(BUNDLE_ID);
    click_debug_log(&format!(
        "worker start timeout={} summary={:?} jump_command={:?}",
        timeout_seconds, summary, jump_command
    ));

    let finished = Arc::new(AtomicBool::new(false));
    if timeout_seconds > 0 {
        let done = Arc::clone(&finished);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(timeout_seconds));
            if !done.load(Ordering::SeqCst) {
                click_debug_log("worker timeout reached, exiting");
                std::process::exit(0);
            }
        });
    }

    let response = Notification::new()
        .title(summary)
        .message(body)
        .main_button(MainButton::SingleAction("Open Terminal"))
        .wait_for_click(true)
        .send()
        .map_err(|e| format!("Failed to show notification: {e}"))?;
    finished.store(true, Ordering::SeqCst);
    click_debug_log(&format!("worker response={:?}", response));

    if matches!(
        response,
        NotificationResponse::Click | NotificationResponse::ActionButton(_)
    ) {
        click_debug_log("worker response matched click/action, executing jump command");
        execute_jump_command(jump_command)?;
    } else {
        click_debug_log("worker response did not match click/action, skip jump command");
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
