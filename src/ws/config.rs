#![expect(
    clippy::module_name_repetitions,
    reason = "Configuration types intentionally mirror the module name for clarity"
)]

use std::time::Duration;

/// Default heartbeat interval value.
const DEFAULT_HEARTBEAT_INTERVAL_DURATION: Duration = Duration::from_secs(10);
/// Default heartbeat timeout value.
const DEFAULT_HEARTBEAT_TIMEOUT_DURATION: Duration = Duration::from_secs(30);
/// Default initial backoff duration for reconnections.
const DEFAULT_INITIAL_BACKOFF_DURATION: Duration = Duration::from_secs(1);
/// Default maximum backoff duration for reconnections.
const DEFAULT_MAX_BACKOFF_DURATION: Duration = Duration::from_secs(60);

/// Configuration for WebSocket client behavior.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Interval for sending PING messages to keep connection alive
    pub heartbeat_interval: Duration,
    /// Maximum time to wait for PONG response before considering connection dead
    pub heartbeat_timeout: Duration,
    /// Reconnection strategy configuration
    pub reconnect: ReconnectConfig,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL_DURATION,
            heartbeat_timeout: DEFAULT_HEARTBEAT_TIMEOUT_DURATION,
            reconnect: ReconnectConfig::default(),
        }
    }
}

/// Configuration for automatic reconnection behavior.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Maximum number of reconnection attempts before giving up.
    /// `None` means infinite retries (recommended for production).
    pub max_attempts: Option<u32>,
    /// Initial backoff duration for first reconnection attempt
    pub initial_backoff: Duration,
    /// Maximum backoff duration (cap for exponential growth)
    pub max_backoff: Duration,
    /// Multiplier for exponential backoff (typically 2.0)
    pub backoff_multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: None, // Infinite reconnection by default
            initial_backoff: DEFAULT_INITIAL_BACKOFF_DURATION,
            max_backoff: DEFAULT_MAX_BACKOFF_DURATION,
            backoff_multiplier: 2.0,
        }
    }
}

impl ReconnectConfig {
    /// Calculate backoff duration for a given attempt number.
    #[must_use]
    #[expect(
        clippy::float_arithmetic,
        reason = "Exponential backoff requires floating-point duration calculations"
    )]
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let attempt_i32 = i32::try_from(attempt).unwrap_or(i32::MAX);
        let base_secs = self.initial_backoff.as_secs_f64();
        let scaled = self.backoff_multiplier.powi(attempt_i32);
        Duration::from_secs_f64(base_secs * scaled).min(self.max_backoff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_calculation() {
        let config = ReconnectConfig::default();

        assert_eq!(config.calculate_backoff(0), Duration::from_secs(1));
        assert_eq!(config.calculate_backoff(1), Duration::from_secs(2));
        assert_eq!(config.calculate_backoff(2), Duration::from_secs(4));
        assert_eq!(config.calculate_backoff(3), Duration::from_secs(8));
    }

    #[test]
    fn backoff_cap() {
        let config = ReconnectConfig::default();

        // Attempt 10 would be 1024 seconds, but should be capped at 60
        assert_eq!(config.calculate_backoff(10), Duration::from_secs(60));
    }
}
