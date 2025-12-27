#![expect(
    clippy::module_name_repetitions,
    reason = "Error types include the module name to indicate their scope"
)]

use std::error::Error as StdError;
use std::fmt;

/// WebSocket error variants.
#[non_exhaustive]
#[derive(Debug)]
pub enum WsError {
    /// Error connecting to or communicating with the WebSocket server
    Connection(tokio_tungstenite::tungstenite::Error),
    /// Error parsing a WebSocket message
    MessageParse(serde_json::Error),
    /// Subscription request failed
    SubscriptionFailed(String),
    /// Authentication failed for authenticated channel
    AuthenticationFailed,
    /// WebSocket connection was closed
    ConnectionClosed,
    /// Operation timed out
    Timeout,
    /// Received an invalid or unexpected message
    InvalidMessage(String),
    /// Subscription stream lagged and missed messages
    Lagged {
        /// Number of messages that were missed
        count: u64,
    },
}

impl fmt::Display for WsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(e) => write!(f, "WebSocket connection error: {e}"),
            Self::MessageParse(e) => write!(f, "Failed to parse WebSocket message: {e}"),
            Self::SubscriptionFailed(reason) => write!(f, "Subscription failed: {reason}"),
            Self::AuthenticationFailed => write!(f, "WebSocket authentication failed"),
            Self::ConnectionClosed => write!(f, "WebSocket connection closed"),
            Self::Timeout => write!(f, "WebSocket operation timed out"),
            Self::InvalidMessage(msg) => write!(f, "Invalid WebSocket message: {msg}"),
            Self::Lagged { count } => write!(f, "Subscription lagged, missed {count} messages"),
        }
    }
}

impl StdError for WsError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Connection(e) => Some(e),
            Self::MessageParse(e) => Some(e),
            _ => None,
        }
    }
}

// Integration with main Error type
impl From<WsError> for crate::error::Error {
    fn from(e: WsError) -> Self {
        crate::error::Error::with_source(crate::error::Kind::WebSocket, e)
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for crate::error::Error {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        crate::error::Error::with_source(crate::error::Kind::WebSocket, WsError::Connection(e))
    }
}
