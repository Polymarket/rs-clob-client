//! Core traits for generic WebSocket infrastructure.

use serde::de::DeserializeOwned;

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
pub trait MessageParser<M: DeserializeOwned>: Send + Sync + 'static {
    /// Parse incoming bytes into messages.
    ///
    /// May return empty vec if messages are filtered out based on interest or other criteria.
    /// Handles both single objects and arrays of messages.
    fn parse(&self, bytes: &[u8]) -> crate::Result<Vec<M>>;
}
