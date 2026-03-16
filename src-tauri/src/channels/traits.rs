use async_trait::async_trait;
use crate::error::AppError;
use crate::models::{ChannelConfig, NotificationMessage, SendResult};

/// Trait for notification channel adapters
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Return the channel type identifier
    fn channel_type(&self) -> &'static str;

    /// Validate channel configuration
    fn validate_config(&self, config: &ChannelConfig) -> Result<(), AppError>;

    /// Send a notification through this channel
    async fn send(
        &self,
        config: &ChannelConfig,
        message: &NotificationMessage,
    ) -> Result<SendResult, AppError>;

    /// Send a test notification
    async fn test(&self, config: &ChannelConfig) -> Result<SendResult, AppError>;
}
