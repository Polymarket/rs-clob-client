#![expect(
    clippy::module_name_repetitions,
    reason = "Error types include the module name to indicate their scope"
)]

use std::error::Error as StdError;
use std::fmt;

/// RTDS WebSocket error variants.
#[non_exhaustive]
#[derive(Debug)]
pub enum RtdsError {
    /// Error connecting to or communicating with the WebSocket server
    Connection(tokio_tungstenite::tungstenite::Error),
    /// Error parsing a WebSocket message
    MessageParse(serde_json::Error),
    /// Subscription request failed
    SubscriptionFailed(String),
    /// Authentication failed for protected topic
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

impl fmt::Display for RtdsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(err) => write!(f, "RTDS WebSocket connection error: {err}"),
            Self::MessageParse(err) => write!(f, "Failed to parse RTDS message: {err}"),
            Self::SubscriptionFailed(reason) => write!(f, "RTDS subscription failed: {reason}"),
            Self::AuthenticationFailed => write!(f, "RTDS WebSocket authentication failed"),
            Self::ConnectionClosed => write!(f, "RTDS WebSocket connection closed"),
            Self::Timeout => write!(f, "RTDS WebSocket operation timed out"),
            Self::InvalidMessage(msg) => write!(f, "Invalid RTDS message: {msg}"),
            Self::Lagged { count } => {
                write!(f, "RTDS subscription lagged, missed {count} messages")
            }
        }
    }
}

impl StdError for RtdsError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Connection(err) => Some(err),
            Self::MessageParse(err) => Some(err),
            _ => None,
        }
    }
}

// Integration with main Error type
impl From<RtdsError> for crate::error::Error {
    fn from(err: RtdsError) -> Self {
        crate::error::Error::with_source(crate::error::Kind::WebSocket, err)
    }
}
