// crates/gm_ports/src/outbound/notification_provider.rs
//
// The NotificationProvider outbound port — how the system surfaces notifications
// to the user through non-primary channels (desktop notifications, Slack, etc.).
// In v1 this is a no-op stub; the interface plugins handle user feedback directly
// through their own output mechanisms. The port exists so future notification
// adapters (system notifications, webhook calls) can be added without changing
// the domain or kernel.

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;

/// The severity level of a notification, used by notification adapters
/// to decide how prominently to surface the message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// A notification channel — where the message should be delivered.
#[derive(Debug, Clone)]
pub enum NotificationChannel {
    /// The application's primary UI (CLI stdout, web SSE, desktop toast).
    Ui,
    /// System notification daemon (libnotify on Linux, NSUserNotifications on macOS).
    System,
}

/// Out-of-band notification delivery.
/// The v1 implementation is a no-op; the UI plugins render results themselves.
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    async fn notify(
        &self,
        message:  &str,
        level:    NotificationLevel,
        channel:  NotificationChannel,
    ) -> Result<(), GitManagerError>;
}