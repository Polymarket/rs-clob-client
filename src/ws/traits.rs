//! Core traits for generic WebSocket infrastructure.

use serde::Serialize;

/// Trait for WebSocket messages that can be broadcast and filtered.
///
/// This trait represents the message type that flows through the connection manager's
/// broadcast channel. Each implementation (CLOB/WS, RTDS) will have its own message enum
/// that implements this trait.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone)]
/// enum MyMessage {
///     Public(PublicData),
///     User(UserData),
/// }
///
/// impl WsMessage for MyMessage {}
/// ```
pub trait WsMessage: Clone + Send + Sync + std::fmt::Debug + 'static {}

/// Trait for subscription requests that can be sent over WebSocket.
///
/// This abstracts the different subscription formats:
/// - CLOB/WS: `{"type": "market", "asset_ids": [...], "operation": "subscribe"}`
/// - RTDS: `{"action": "subscribe", "subscriptions": [...]}`
///
/// # Example
///
/// ```ignore
/// #[derive(Serialize, Clone)]
/// pub struct MySubscriptionRequest {
///     action: String,
///     topics: Vec<String>,
/// }
///
/// impl SubscriptionRequest for MySubscriptionRequest {}
/// ```
pub trait SubscriptionRequest: Serialize + Clone + Send + Sync + 'static {
    /// Serialize to JSON string for transmission.
    ///
    /// Returns an error if serialization fails. Default implementation uses
    /// `serde_json::to_string`.
    fn to_json(&self) -> crate::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn from_topics(topics: Vec<String>) -> Self;

    fn topics(&self) -> impl Iterator<Item = &str>;
}

pub trait UnsubscriptionRequest: Serialize + Clone + Send + Sync + 'static {
    /// Serialize to JSON string for transmission.
    ///
    /// Returns an error if serialization fails. Default implementation uses
    /// `serde_json::to_string`.
    fn to_json(&self) -> crate::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn topics(&self) -> impl Iterator<Item = &str>;

    fn set_topics(&mut self, topics: Vec<String>);
}

/// Message parser trait for converting raw bytes to messages.
///
/// This abstracts the different parsing strategies:
/// - CLOB/WS: Interest-based filtering, peeking at `event_type` before full deserialization
/// - RTDS: Simple parse, no filtering
///
/// # Example
///
/// ```ignore
/// pub struct SimpleParser;
///
/// impl MessageParser<MyMessage> for SimpleParser {
///     fn parse(&self, bytes: &[u8]) -> crate::Result<Vec<MyMessage>> {
///         let msg: MyMessage = serde_json::from_slice(bytes)?;
///         Ok(vec![msg])
///     }
/// }
/// ```
pub trait MessageParser<M: WsMessage>: Send + Sync + 'static {
    /// Parse incoming bytes into messages.
    ///
    /// May return empty vec if messages are filtered out based on interest or other criteria.
    /// Handles both single objects and arrays of messages.
    fn parse(&self, bytes: &[u8]) -> crate::Result<Vec<M>>;
}
