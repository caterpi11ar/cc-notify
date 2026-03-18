use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};
use super::traits::NotificationChannel;
use std::path::{Path, PathBuf};

pub struct VoiceChannel;

#[async_trait]
impl NotificationChannel for VoiceChannel {
    fn channel_type(&self) -> &'static str {
        "voice"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), AppError> {
        if !cfg!(target_os = "macos") {
            return Err(AppError::InvalidInput(
                "Voice channel is only available on macOS".to_string(),
            ));
        }

        let mode = config
            .params
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("tts");
        if mode != "tts" && mode != "voice_pack" {
            return Err(AppError::InvalidInput(format!(
                "Invalid voice mode: {mode} (expected 'tts' or 'voice_pack')"
            )));
        }

        if mode == "voice_pack" {
            let dir = config
                .params
                .get("voice_pack_dir")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if dir.is_empty() {
                return Err(AppError::InvalidInput(
                    "voice_pack_dir is required when mode=voice_pack".to_string(),
                ));
            }

            let dir_path = Path::new(dir);
            if !dir_path.is_dir() {
                return Err(AppError::InvalidInput(format!(
                    "Voice pack directory not found: {dir}"
                )));
            }

            std::fs::read_dir(dir_path).map_err(|e| {
                AppError::InvalidInput(format!(
                    "Voice pack directory is not readable ({dir}): {e}"
                ))
            })?;
        }

        Ok(())
    }

    async fn send(
        &self,
        config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError> {
        if !cfg!(target_os = "macos") {
            return Err(AppError::Channel(
                "Voice channel is only available on macOS".to_string(),
            ));
        }

        self.validate_config(config)?;

        let mode = config
            .params
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("tts");

        let voice = config
            .params
            .get("voice")
            .and_then(|v| v.as_str())
            .unwrap_or("Samantha");

        let rate = config
            .params
            .get("rate")
            .and_then(json_rate_to_u64)
            .unwrap_or(200);

        let text = message
            .message
            .as_deref()
            .unwrap_or(&message.event);

        if mode == "voice_pack" {
            let voice_pack_dir = config
                .params
                .get("voice_pack_dir")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if let Some(audio_file) = find_voice_pack_file(voice_pack_dir, &message.event) {
                play_audio_file(&audio_file)?;
            } else if let Some(default_file) = find_voice_pack_file(voice_pack_dir, "default") {
                play_audio_file(&default_file)?;
            }
        } else {
            speak_text(voice, rate, text)?;
        }

        Ok(SendResult {
            success: true,
            channel_type: "voice".to_string(),
            message: None,
        })
    }

    async fn test(&self, config: &ChannelConfig) -> Result<SendResult, AppError> {
        self.validate_config(config)?;
        let test_msg = NotificationMessage {
            event: "test".to_string(),
            event_type: None,
            message: Some("Test voice notification from CC Notify".to_string()),
            tool: Some("cc-notify".to_string()),
            session_id: None,
            project: None,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now().timestamp(),
            title: None,
            model: None,
            cwd: None,
            last_assistant_message: None,
            source: None,
            reason: None,
            agent_type: None,
        };
        self.send(config, &test_msg).await
    }
}

fn find_voice_pack_file(dir: &str, event_id: &str) -> Option<PathBuf> {
    let exts = ["wav", "aiff", "m4a", "mp3"];
    for ext in exts {
        let path = Path::new(dir).join(format!("{event_id}.{ext}"));
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn play_audio_file(path: &Path) -> Result<(), AppError> {
    std::process::Command::new("afplay")
        .arg(path)
        .spawn()
        .map_err(|e| AppError::Channel(format!("Failed to play voice pack file: {e}")))?;
    Ok(())
}

fn speak_text(voice: &str, rate: u64, text: &str) -> Result<(), AppError> {
    let speech_text = format!("CC Notify: {text}");
    std::process::Command::new("say")
        .args(["-v", voice, "-r", &rate.to_string()])
        .arg(&speech_text)
        .spawn()
        .map_err(|e| AppError::Channel(format!("Failed to run say command: {e}")))?;
    Ok(())
}

fn json_rate_to_u64(value: &serde_json::Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_f64().map(|v| v.max(0.0) as u64))
        .or_else(|| value.as_str().and_then(|v| v.parse::<u64>().ok()))
}

#[cfg(test)]
mod tests {
    use super::find_voice_pack_file;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn finds_event_file_before_default() {
        let dir = tempdir().expect("create temp dir");
        let base = dir.path();

        fs::write(base.join("stop.wav"), b"mock-audio-stop").expect("write stop.wav");
        fs::write(base.join("default.aiff"), b"mock-audio-default").expect("write default.aiff");

        let event_hit = find_voice_pack_file(base.to_str().unwrap_or_default(), "stop");
        let default_hit = find_voice_pack_file(base.to_str().unwrap_or_default(), "default");

        assert!(event_hit.is_some());
        assert!(default_hit.is_some());
        assert!(event_hit
            .unwrap()
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|n| n == "stop.wav"));
        assert!(default_hit
            .unwrap()
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|n| n == "default.aiff"));
    }

    #[test]
    fn returns_none_when_no_mock_audio_files() {
        let dir = tempdir().expect("create temp dir");
        let hit = find_voice_pack_file(dir.path().to_str().unwrap_or_default(), "notification.idle_prompt");
        assert!(hit.is_none());
    }
}
