use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;

pub struct SoundChannel;

#[async_trait]
impl NotificationChannel for SoundChannel {
    fn channel_type(&self) -> &'static str {
        "sound"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), AppError> {
        // If a custom sound file is provided, check it exists
        if let Some(sound_file) = config.params.get("sound_file").and_then(|v| v.as_str()) {
            if !sound_file.is_empty() && !std::path::Path::new(sound_file).exists() {
                return Err(AppError::InvalidInput(format!(
                    "Sound file not found: {sound_file}"
                )));
            }
        }
        Ok(())
    }

    async fn send(
        &self,
        config: &ChannelConfig,
        _message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        let sound_file = config
            .params
            .get("sound_file")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        play_sound(sound_file)?;

        Ok(SendResult {
            success: true,
            channel_type: "sound".to_string(),
            message: None,
        })
    }

    async fn test(&self, config: &ChannelConfig) -> Result<SendResult, AppError> {
        self.validate_config(config)?;
        let test_msg = NotificationMessage {
            event: "test".to_string(),
            event_type: None,
            message: Some("Test sound from CC Notify".to_string()),
            tool: Some("cc-notify".to_string()),
            session_id: None,
            project: None,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now().timestamp(),
        };
        self.send(config, &test_msg).await
    }
}

fn play_sound(sound_file: &str) -> Result<(), AppError> {
    if cfg!(target_os = "macos") {
        play_sound_macos(sound_file)
    } else if cfg!(target_os = "linux") {
        play_sound_linux(sound_file)
    } else if cfg!(target_os = "windows") {
        play_sound_windows(sound_file)
    } else {
        Err(AppError::Channel(
            "Sound playback is not supported on this platform".to_string(),
        ))
    }
}

fn play_sound_macos(sound_file: &str) -> Result<(), AppError> {
    let file = if sound_file.is_empty() {
        // Default macOS system sound
        "/System/Library/Sounds/Glass.aiff"
    } else {
        sound_file
    };

    std::process::Command::new("afplay")
        .arg(file)
        .spawn()
        .map_err(|e| AppError::Channel(format!("Failed to play sound with afplay: {e}")))?;

    Ok(())
}

fn play_sound_linux(sound_file: &str) -> Result<(), AppError> {
    let file = if sound_file.is_empty() {
        // Try common system sound locations
        let default_paths = [
            "/usr/share/sounds/freedesktop/stereo/complete.oga",
            "/usr/share/sounds/freedesktop/stereo/message.oga",
            "/usr/share/sounds/gnome/default/alerts/drip.ogg",
        ];
        let found = default_paths.iter().find(|p| std::path::Path::new(p).exists());
        match found {
            Some(path) => *path,
            None => {
                return Err(AppError::Channel(
                    "No default sound file found on this system".to_string(),
                ));
            }
        }
    } else {
        sound_file
    };

    // Try paplay first, then aplay, then ffplay
    let result = std::process::Command::new("paplay")
        .arg(file)
        .spawn();

    if result.is_ok() {
        return Ok(());
    }

    let result = std::process::Command::new("aplay")
        .arg(file)
        .spawn();

    if result.is_ok() {
        return Ok(());
    }

    let result = std::process::Command::new("ffplay")
        .args(["-nodisp", "-autoexit", "-loglevel", "quiet"])
        .arg(file)
        .spawn();

    if result.is_ok() {
        return Ok(());
    }

    Err(AppError::Channel(
        "No audio player found (tried paplay, aplay, ffplay)".to_string(),
    ))
}

fn play_sound_windows(sound_file: &str) -> Result<(), AppError> {
    let script = if sound_file.is_empty() {
        // Play default Windows notification sound
        r#"[System.Media.SystemSounds]::Asterisk.Play()"#.to_string()
    } else {
        format!(
            r#"(New-Object Media.SoundPlayer '{}').PlaySync()"#,
            sound_file.replace('\'', "''")
        )
    };

    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .spawn()
        .map_err(|e| AppError::Channel(format!("Failed to play sound with powershell: {e}")))?;

    Ok(())
}
