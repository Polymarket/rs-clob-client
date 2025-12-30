use std::fmt;

use alloy::primitives::Address;
use rust_decimal::Decimal;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer as _, Serialize};
use serde_json::Value;

use super::interest::MessageInterest;
use crate::auth::Credentials;
use crate::error::Kind;

/// Top-level RTDS message wrapper.
///
/// All messages received from the RTDS WebSocket connection are deserialized into this struct.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub struct RtdsMessage {
    /// The subscription topic (e.g., `crypto_prices`, `comments`)
    pub topic: String,
    /// The message type/event (e.g., `update`, `comment_created`)
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    /// Event-specific data object
    pub payload: Value,
}

impl RtdsMessage {
    /// Get the message interest for this message based on its topic.
    #[must_use]
    pub fn interest(&self) -> MessageInterest {
        MessageInterest::from_topic(&self.topic)
    }

    /// Check if this message matches the given interest filter.
    #[must_use]
    pub fn matches_interest(&self, interest: MessageInterest) -> bool {
        let msg_interest = self.interest();
        !msg_interest.is_empty() && interest.contains(msg_interest)
    }

    /// Try to extract the payload as a crypto price update.
    #[must_use]
    pub fn as_crypto_price(&self) -> Option<CryptoPrice> {
        if self.topic == "crypto_prices" {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Try to extract the payload as a Chainlink price update.
    #[must_use]
    pub fn as_chainlink_price(&self) -> Option<ChainlinkPrice> {
        if self.topic == "crypto_prices_chainlink" {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }

    /// Try to extract the payload as a comment event.
    #[must_use]
    pub fn as_comment(&self) -> Option<Comment> {
        if self.topic == "comments" {
            serde_json::from_value(self.payload.clone()).ok()
        } else {
            None
        }
    }
}

/// Binance crypto price update payload.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CryptoPrice {
    /// Trading pair symbol (lowercase concatenated, e.g., "solusdt", "btcusdt")
    pub symbol: String,
    /// Price timestamp in Unix milliseconds
    pub timestamp: i64,
    /// Current price value
    pub value: Decimal,
}

/// Chainlink price feed update payload.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainlinkPrice {
    /// Trading pair symbol (slash-separated, e.g., "eth/usd", "btc/usd")
    pub symbol: String,
    /// Price timestamp in Unix milliseconds
    pub timestamp: i64,
    /// Current price value
    pub value: Decimal,
}

/// Comment event payload.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Comment {
    /// Unique identifier for this comment
    pub id: String,
    /// The text content of the comment
    pub body: String,
    /// ISO 8601 timestamp when the comment was created
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// ID of the parent comment if this is a reply (null for top-level comments)
    #[serde(rename = "parentCommentID", default)]
    pub parent_comment_id: Option<String>,
    /// ID of the parent entity (event, market, etc.)
    #[serde(rename = "parentEntityID")]
    pub parent_entity_id: i64,
    /// Type of parent entity (e.g., "Event", "Market")
    #[serde(rename = "parentEntityType")]
    pub parent_entity_type: String,
    /// Profile information of the user who created the comment
    pub profile: CommentProfile,
    /// Current number of reactions on this comment
    #[serde(rename = "reactionCount", default)]
    pub reaction_count: i64,
    /// Polygon address for replies
    #[serde(rename = "replyAddress", default)]
    pub reply_address: Option<String>,
    /// Current number of reports on this comment
    #[serde(rename = "reportCount", default)]
    pub report_count: i64,
    /// Polygon address of the user who created the comment
    #[serde(rename = "userAddress")]
    pub user_address: String,
}

/// Profile information for a comment author.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommentProfile {
    /// User profile address
    #[serde(rename = "baseAddress")]
    pub base_address: String,
    /// Whether the username should be displayed publicly
    #[serde(rename = "displayUsernamePublic", default)]
    pub display_username_public: bool,
    /// User's display name
    pub name: String,
    /// Proxy wallet address used for transactions
    #[serde(rename = "proxyWallet", default)]
    pub proxy_wallet: Option<String>,
    /// Generated pseudonym for the user
    #[serde(default)]
    pub pseudonym: Option<String>,
}

/// Comment message types.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentType {
    /// New comment created
    CommentCreated,
    /// Comment was removed/deleted
    CommentRemoved,
    /// Reaction added to a comment
    ReactionCreated,
    /// Reaction removed from a comment
    ReactionRemoved,
}

/// RTDS subscription request message.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize)]
pub struct SubscriptionRequest {
    /// Action type ("subscribe" or "unsubscribe")
    pub action: SubscriptionAction,
    /// List of subscriptions
    pub subscriptions: Vec<Subscription>,
}

/// Subscription action type.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionAction {
    /// Subscribe to topics
    Subscribe,
    /// Unsubscribe from topics
    Unsubscribe,
}

/// Individual subscription configuration.
///
/// # Security
///
/// When serialized, this struct exposes sensitive credentials (`clob_auth`) in plaintext.
/// Ensure subscription requests are only sent over secure WebSocket connections (`wss://`)
/// and never logged or exposed in error messages.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Subscription {
    /// Topic name (e.g., `crypto_prices`, `comments`)
    pub topic: String,
    /// Message type filter (e.g., `update`, `comment_created`, or `*` for all)
    pub msg_type: String,
    /// Optional filters (string or JSON object)
    pub filters: Option<String>,
    /// CLOB authentication (key, secret, passphrase)
    pub clob_auth: Option<Credentials>,
    /// Gamma authentication (wallet address)
    pub gamma_auth: Option<Address>,
}

impl Subscription {
    /// Create a subscription for Binance crypto prices.
    #[must_use]
    pub fn crypto_prices(symbols: Option<Vec<String>>) -> Self {
        // Server expects filters as a JSON array, e.g. ["btcusdt","ethusdt"]
        let filters =
            symbols.map(|s| serde_json::to_string(&s).unwrap_or_else(|_| "[]".to_owned()));
        Self {
            topic: "crypto_prices".to_owned(),
            msg_type: "update".to_owned(),
            filters,
            clob_auth: None,
            gamma_auth: None,
        }
    }

    /// Create a subscription for Chainlink crypto prices.
    #[must_use]
    pub fn chainlink_prices(symbol: Option<String>) -> Self {
        let filters = symbol.map(|s| format!(r#"{{"symbol":"{s}"}}"#));
        Self {
            topic: "crypto_prices_chainlink".to_owned(),
            msg_type: "*".to_owned(),
            filters,
            clob_auth: None,
            gamma_auth: None,
        }
    }

    /// Create a subscription for comments.
    #[must_use]
    pub fn comments(msg_type: Option<CommentType>) -> Self {
        let type_str = msg_type.map_or("*".to_owned(), |t| {
            serde_json::to_string(&t)
                .ok()
                .and_then(|s| s.trim_matches('"').to_owned().into())
                .unwrap_or_else(|| "*".to_owned())
        });
        Self {
            topic: "comments".to_owned(),
            msg_type: type_str,
            filters: None,
            clob_auth: None,
            gamma_auth: None,
        }
    }

    /// Set CLOB authentication for this subscription.
    #[must_use]
    pub fn with_clob_auth(mut self, credentials: Credentials) -> Self {
        self.clob_auth = Some(credentials);
        self
    }

    /// Set Gamma authentication for this subscription.
    #[must_use]
    pub fn with_gamma_auth(mut self, address: Address) -> Self {
        self.gamma_auth = Some(address);
        self
    }

    /// Set custom filters for this subscription.
    #[must_use]
    pub fn with_filters(mut self, filters: String) -> Self {
        self.filters = Some(filters);
        self
    }
}

// Custom Serialize implementation for Subscription to handle auth fields
impl Serialize for Subscription {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("topic", &self.topic)?;
        map.serialize_entry("type", &self.msg_type)?;

        if let Some(filters) = &self.filters {
            // Parse filters as JSON value to emit raw JSON, not escaped string
            if let Ok(json_value) = serde_json::from_str::<Value>(filters) {
                map.serialize_entry("filters", &json_value)?;
            } else {
                // Fallback: emit as string if not valid JSON
                map.serialize_entry("filters", filters)?;
            }
        }

        // SECURITY: Credentials are intentionally revealed here for the WebSocket auth protocol.
        // This data is only sent over wss:// connections to the RTDS server.
        if let Some(creds) = &self.clob_auth {
            let auth = serde_json::json!({
                "key": creds.key.to_string(),
                "secret": creds.secret.reveal(),
                "passphrase": creds.passphrase.reveal(),
            });
            map.serialize_entry("clob_auth", &auth)?;
        }

        if let Some(addr) = &self.gamma_auth {
            let auth = serde_json::json!({
                "address": addr.to_string(),
            });
            map.serialize_entry("gamma_auth", &auth)?;
        }

        map.end()
    }
}

impl SubscriptionRequest {
    /// Create a subscribe request.
    #[must_use]
    pub fn subscribe(subscriptions: Vec<Subscription>) -> Self {
        Self {
            action: SubscriptionAction::Subscribe,
            subscriptions,
        }
    }

    /// Create an unsubscribe request.
    #[must_use]
    pub fn unsubscribe(subscriptions: Vec<Subscription>) -> Self {
        Self {
            action: SubscriptionAction::Unsubscribe,
            subscriptions,
        }
    }
}

/// Result of peeking at the message structure without full deserialization.
enum MessageShape {
    /// Single object with the given topic (if present).
    Single(Option<String>),
    /// Array of messages requiring full deserialization.
    Array,
}

/// Peeks at the JSON structure to determine if it's a single object or array,
/// and extracts the topic for single objects without full deserialization.
fn peek_message_shape(bytes: &[u8]) -> Result<MessageShape, serde_json::Error> {
    struct ShapePeeker;

    impl<'de> Visitor<'de> for ShapePeeker {
        type Value = MessageShape;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a JSON object or array")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut topic: Option<String> = None;
            while let Some(key) = map.next_key::<&str>()? {
                if key == "topic" {
                    topic = Some(map.next_value::<String>()?);
                } else {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
            Ok(MessageShape::Single(topic))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            // Consume the entire sequence to avoid "trailing characters" error
            while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {}
            Ok(MessageShape::Array)
        }
    }

    let mut de = serde_json::Deserializer::from_slice(bytes);
    de.deserialize_any(ShapePeeker)
}

/// Deserialize messages from the byte slice, filtering by interest.
///
/// For single objects, the topic is extracted first to skip uninteresting messages
/// without full deserialization. For arrays, all messages are deserialized and filtered.
pub fn parse_if_interested(
    bytes: &[u8],
    interest: &MessageInterest,
) -> crate::Result<Vec<RtdsMessage>> {
    let shape = peek_message_shape(bytes)
        .map_err(|e| crate::error::Error::with_source(Kind::Internal, e))?;

    match shape {
        MessageShape::Single(None) => Ok(vec![]),
        MessageShape::Single(Some(topic)) => {
            if !interest.is_interested_in_topic(&topic) {
                return Ok(vec![]);
            }
            let msg: RtdsMessage = serde_json::from_slice(bytes)?;
            Ok(vec![msg])
        }
        MessageShape::Array => {
            let messages: Vec<RtdsMessage> = serde_json::from_slice(bytes)?;
            Ok(messages
                .into_iter()
                .filter(|msg| msg.matches_interest(*interest))
                .collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn parse_crypto_price_message() {
        let json = r#"{
            "topic": "crypto_prices",
            "type": "update",
            "timestamp": 1753314064237,
            "payload": {
                "symbol": "solusdt",
                "timestamp": 1753314064213,
                "value": 189.55
            }
        }"#;

        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::ALL).unwrap();
        assert_eq!(msgs.len(), 1);

        let msg = &msgs[0];
        assert_eq!(msg.topic, "crypto_prices");
        assert_eq!(msg.msg_type, "update");

        let price = msg.as_crypto_price().unwrap();
        assert_eq!(price.symbol, "solusdt");
        assert_eq!(price.value, dec!(189.55));
    }

    #[test]
    fn parse_chainlink_price_message() {
        let json = r#"{
            "topic": "crypto_prices_chainlink",
            "type": "update",
            "timestamp": 1753314064237,
            "payload": {
                "symbol": "eth/usd",
                "timestamp": 1753314064213,
                "value": 3456.78
            }
        }"#;

        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::ALL).unwrap();
        assert_eq!(msgs.len(), 1);

        let msg = &msgs[0];
        assert_eq!(msg.topic, "crypto_prices_chainlink");

        let price = msg.as_chainlink_price().unwrap();
        assert_eq!(price.symbol, "eth/usd");
        assert_eq!(price.value, dec!(3456.78));
    }

    #[test]
    fn parse_comment_message() {
        let json = r#"{
            "topic": "comments",
            "type": "comment_created",
            "timestamp": 1753454975808,
            "payload": {
                "body": "Test comment",
                "createdAt": "2025-07-25T14:49:35.801298Z",
                "id": "1763355",
                "parentCommentID": "1763325",
                "parentEntityID": 18396,
                "parentEntityType": "Event",
                "profile": {
                    "baseAddress": "0xce533188d53a16ed580fd5121dedf166d3482677",
                    "displayUsernamePublic": true,
                    "name": "salted.caramel",
                    "proxyWallet": "0x4ca749dcfa93c87e5ee23e2d21ff4422c7a4c1ee",
                    "pseudonym": "Adored-Disparity"
                },
                "reactionCount": 0,
                "replyAddress": "0x0bda5d16f76cd1d3485bcc7a44bc6fa7db004cdd",
                "reportCount": 0,
                "userAddress": "0xce533188d53a16ed580fd5121dedf166d3482677"
            }
        }"#;

        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::ALL).unwrap();
        assert_eq!(msgs.len(), 1);

        let msg = &msgs[0];
        assert_eq!(msg.topic, "comments");
        assert_eq!(msg.msg_type, "comment_created");

        let comment = msg.as_comment().unwrap();
        assert_eq!(comment.id, "1763355");
        assert_eq!(comment.body, "Test comment");
        assert_eq!(comment.profile.name, "salted.caramel");
    }

    #[test]
    fn parse_filters_by_interest() {
        let json = r#"{
            "topic": "crypto_prices",
            "type": "update",
            "timestamp": 1753314064237,
            "payload": {
                "symbol": "btcusdt",
                "timestamp": 1753314064213,
                "value": 67234.50
            }
        }"#;

        // Only interested in comments, not crypto prices
        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::COMMENTS).unwrap();
        assert_eq!(msgs.len(), 0);

        // Interested in crypto prices
        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::CRYPTO_PRICES).unwrap();
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn serialize_subscription_request() {
        let sub =
            Subscription::crypto_prices(Some(vec!["btcusdt".to_owned(), "ethusdt".to_owned()]));
        let request = SubscriptionRequest::subscribe(vec![sub]);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"subscribe\""));
        assert!(json.contains("\"topic\":\"crypto_prices\""));
        // Filters should be a JSON array, not a comma-separated string
        assert!(json.contains("\"filters\":[\"btcusdt\",\"ethusdt\"]"));
    }

    #[test]
    fn serialize_chainlink_subscription() {
        let sub = Subscription::chainlink_prices(Some("eth/usd".to_owned()));
        let request = SubscriptionRequest::subscribe(vec![sub]);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"topic\":\"crypto_prices_chainlink\""));
        assert!(json.contains("\"type\":\"*\""));
    }

    #[test]
    fn serialize_comments_subscription() {
        let sub = Subscription::comments(Some(CommentType::CommentCreated));
        let request = SubscriptionRequest::subscribe(vec![sub]);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"topic\":\"comments\""));
        assert!(json.contains("\"type\":\"comment_created\""));
    }
}
