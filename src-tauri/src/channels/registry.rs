use std::collections::HashMap;
use super::traits::NotificationChannel;
use super::*;

pub struct ChannelRegistry {
    channels: HashMap<String, Box<dyn NotificationChannel>>,
}

impl ChannelRegistry {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        let mut channels: HashMap<String, Box<dyn NotificationChannel>> = HashMap::new();
        channels.insert("native".to_string(), Box::new(native::NativeChannel::new(app_handle.clone())));
        channels.insert("slack".to_string(), Box::new(slack::SlackChannel));
        channels.insert("discord".to_string(), Box::new(discord::DiscordChannel));
        channels.insert("teams".to_string(), Box::new(teams::TeamsChannel));
        channels.insert("telegram".to_string(), Box::new(telegram::TelegramChannel));
        channels.insert("webhook".to_string(), Box::new(webhook::WebhookChannel));
        channels.insert("sound".to_string(), Box::new(sound::SoundChannel));
        channels.insert("voice".to_string(), Box::new(voice::VoiceChannel));
        channels.insert(
            "tray_badge".to_string(),
            Box::new(tray_badge::TrayBadgeChannel::new(app_handle)),
        );
        Self { channels }
    }

    pub fn get(&self, channel_type: &str) -> Option<&dyn NotificationChannel> {
        // Legacy mapping: feishu channels are now handled by webhook
        let resolved = match channel_type {
            "feishu" => "webhook",
            other => other,
        };
        self.channels.get(resolved).map(|c| c.as_ref())
    }
}
