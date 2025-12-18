use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use serde_with::{DisplayFromStr, serde_as};

use super::interest::MessageInterest;
use crate::{
    auth::Credentials,
    types::{Side, TraderSide},
};

/// Authentication payload for user channel subscriptions.
pub type AuthPayload = Credentials;

/// Top-level WebSocket message wrapper.
///
/// All messages received from the WebSocket connection are deserialized into this enum.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum WsMessage {
    /// User trade execution (authenticated channel)
    Trade(TradeMessage),
    /// User order update (authenticated channel)
    Order(OrderMessage),
    /// Price change notification
    PriceChange(PriceChange),
    /// Tick size change notification
    TickSizeChange(TickSizeChange),
    /// Last trade price update
    LastTradePrice(LastTradePrice),
    /// Full or incremental orderbook update
    Book(BookUpdate),
}

impl WsMessage {
    /// Check if the message is a user-specific message.
    #[must_use]
    pub const fn is_user(&self) -> bool {
        matches!(self, WsMessage::Trade(_) | WsMessage::Order(_))
    }

    /// Check if the message is a market data message.
    #[must_use]
    pub const fn is_market(&self) -> bool {
        !self.is_user()
    }
}

/// Orderbook update message (full snapshot or delta).
///
/// When first subscribing or when trades occur, this message contains the current
/// state of the orderbook with bids and asks arrays.
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookUpdate {
    /// Asset/token identifier
    pub asset_id: String,
    /// Market identifier
    pub market: String,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
    /// Current bid levels (price descending)
    #[serde(default)]
    pub bids: Vec<OrderBookLevel>,
    /// Current ask levels (price ascending)
    #[serde(default)]
    pub asks: Vec<OrderBookLevel>,
    /// Hash for orderbook validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// Individual price level in an orderbook.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderBookLevel {
    /// Price at this level
    pub price: Decimal,
    /// Total size available at this price
    pub size: Decimal,
}

/// Price change event triggered by new orders or cancellations.
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriceChange {
    /// Asset/token identifier
    pub asset_id: String,
    /// Market identifier
    pub market: String,
    /// New price
    pub price: Decimal,
    /// Total size affected by this price change (if provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Decimal>,
    /// Side of the price change (BUY or SELL)
    pub side: Side,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
    /// Hash for validation (if present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Best bid price after this change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_bid: Option<Decimal>,
    /// Best ask price after this change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_ask: Option<Decimal>,
}

/// Raw batch payload sent for price change events.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PriceChangeBatch {
    /// Market identifier shared across batch entries
    pub market: String,
    /// Unix timestamp in milliseconds (string or number)
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
    /// Individual price change entries
    #[serde(default)]
    pub price_changes: Vec<PriceChangeBatchEntry>,
}

/// Individual entry inside a price change batch.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PriceChangeBatchEntry {
    /// Asset/token identifier
    pub asset_id: String,
    /// New price
    pub price: Decimal,
    /// Total size affected at this price level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Decimal>,
    /// Side of the book that changed (BUY or SELL)
    pub side: Side,
    /// Hash for this entry (if present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// Best bid price after the change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_bid: Option<Decimal>,
    /// Best ask price after the change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_ask: Option<Decimal>,
}

impl PriceChangeBatch {
    fn into_price_changes(self) -> Vec<PriceChange> {
        self.price_changes
            .into_iter()
            .map(|entry| PriceChange {
                asset_id: entry.asset_id,
                market: self.market.clone(),
                price: entry.price,
                size: entry.size,
                side: entry.side,
                timestamp: self.timestamp,
                hash: entry.hash,
                best_bid: entry.best_bid,
                best_ask: entry.best_ask,
            })
            .collect()
    }
}

/// Tick size change event (triggered when price crosses thresholds).
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TickSizeChange {
    /// Asset/token identifier
    pub asset_id: String,
    /// Market identifier
    pub market: String,
    /// Previous tick size
    pub old_tick_size: Decimal,
    /// New tick size
    pub new_tick_size: Decimal,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// Last trade price update.
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LastTradePrice {
    /// Asset/token identifier
    pub asset_id: String,
    /// Market identifier
    pub market: String,
    /// Last trade price
    pub price: Decimal,
    /// Side of the last trade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// Maker order details within a trade message.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MakerOrder {
    /// Asset/token identifier of the maker order
    pub asset_id: String,
    /// Amount of maker order matched in trade
    pub matched_amount: String,
    /// Maker order ID
    pub order_id: String,
    /// Outcome (Yes/No)
    pub outcome: String,
    /// Owner (API key) of maker order
    pub owner: String,
    /// Price of maker order
    pub price: String,
}

/// User trade execution message (authenticated channel only).
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradeMessage {
    /// Trade identifier
    pub id: String,
    /// Market identifier (condition ID)
    pub market: String,
    /// Asset/token identifier
    pub asset_id: String,
    /// Side of the trade (BUY or SELL)
    pub side: Side,
    /// Size of the trade
    pub size: String,
    /// Execution price
    pub price: String,
    /// Trade status (MATCHED, MINED, CONFIRMED, etc.)
    pub status: String,
    /// Event type (always "trade")
    #[serde(default)]
    pub event_type: Option<String>,
    /// Message type (always "TRADE")
    #[serde(rename = "type", default)]
    pub msg_type: Option<String>,
    /// Timestamp of last trade modification
    #[serde(default)]
    pub last_update: Option<String>,
    /// Time trade was matched
    #[serde(default)]
    pub matchtime: Option<String>,
    /// Unix timestamp of event
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Outcome (Yes/No)
    #[serde(default)]
    pub outcome: Option<String>,
    /// API key of event owner
    #[serde(default)]
    pub owner: Option<String>,
    /// API key of trade owner
    #[serde(default)]
    pub trade_owner: Option<String>,
    /// ID of taker order
    #[serde(default)]
    pub taker_order_id: Option<String>,
    /// Array of maker order details
    #[serde(default)]
    pub maker_orders: Vec<MakerOrder>,
    /// Fee rate in basis points (string in API response)
    #[serde(default)]
    pub fee_rate_bps: Option<String>,
    /// Transaction hash
    #[serde(default)]
    pub transaction_hash: Option<String>,
    /// Whether user was maker or taker
    #[serde(default)]
    pub trader_side: Option<TraderSide>,
}

/// User order update message (authenticated channel only).
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderMessage {
    /// Order identifier
    pub id: String,
    /// Market identifier (condition ID)
    pub market: String,
    /// Asset/token identifier
    pub asset_id: String,
    /// Side of the order (BUY or SELL)
    pub side: Side,
    /// Order price
    pub price: String,
    /// Event type (always "order")
    #[serde(default)]
    pub event_type: Option<String>,
    /// Message type (PLACEMENT, UPDATE, or CANCELLATION)
    #[serde(rename = "type", default)]
    pub msg_type: Option<String>,
    /// Outcome (Yes/No)
    #[serde(default)]
    pub outcome: Option<String>,
    /// Owner (API key)
    #[serde(default)]
    pub owner: Option<String>,
    /// Order owner (API key of order originator)
    #[serde(default)]
    pub order_owner: Option<String>,
    /// Original order size
    #[serde(default)]
    pub original_size: Option<String>,
    /// Amount matched so far
    #[serde(default)]
    pub size_matched: Option<String>,
    /// Unix timestamp of event
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Associated trade IDs
    #[serde(default)]
    pub associate_trades: Option<Vec<String>>,
}

/// Order status for WebSocket order messages.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// Order is open and active
    Open,
    /// Order has been matched with a counterparty
    Matched,
    /// Order has been partially filled
    PartiallyFilled,
    /// Order has been cancelled
    Cancelled,
    /// Order has been placed (initial status)
    Placement,
    /// Order update (partial match)
    Update,
    /// Order cancellation in progress
    Cancellation,
}

/// Subscription request message sent to the WebSocket server.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionRequest {
    /// Subscription type ("market" or "user")
    pub r#type: String,
    /// List of market IDs
    pub markets: Vec<String>,
    /// List of asset IDs (the API uses `assets_ids`)
    #[serde(rename = "assets_ids")]
    pub asset_ids: Vec<String>,
    /// Request initial state dump
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_dump: Option<bool>,
    /// Authentication credentials
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthPayload>,
}

impl SubscriptionRequest {
    /// Create a market subscription request.
    #[must_use]
    pub fn market(asset_ids: Vec<String>) -> Self {
        Self {
            r#type: "market".to_owned(),
            markets: vec![],
            asset_ids,
            initial_dump: Some(true),
            auth: None,
        }
    }

    /// Create a user subscription request.
    #[must_use]
    pub fn user(markets: Vec<String>, auth: AuthPayload) -> Self {
        Self {
            r#type: "user".to_owned(),
            markets,
            asset_ids: vec![],
            initial_dump: Some(true),
            auth: Some(auth),
        }
    }
}

/// Calculated midpoint update (derived from orderbook).
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MidpointUpdate {
    /// Asset/token identifier
    pub asset_id: String,
    /// Market identifier
    pub market: String,
    /// Calculated midpoint price
    pub midpoint: Decimal,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// Parse a raw WebSocket message string into one or more [`WsMessage`] instances.
///
/// Messages that don't match the interest are skipped entirely, avoiding deserialization overhead.
pub(crate) fn parse_ws_text(
    text: &str,
    interest: MessageInterest,
) -> serde_json::Result<Vec<WsMessage>> {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if trimmed.starts_with('[') {
        let values: Vec<Value> = serde_json::from_str(trimmed)?;
        let mut messages = Vec::new();
        for value in values {
            messages.extend(parse_ws_value(value, interest)?);
        }
        Ok(messages)
    } else {
        parse_ws_value(serde_json::from_str(trimmed)?, interest)
    }
}

/// Parse a single [`Value`] into [`Vec<WsMessage>`].
///
/// Routes deserialization based on `event_type` field for efficiency,
/// avoiding the overhead of trying to deserialize into all possible message types.
fn parse_ws_value(value: Value, interest: MessageInterest) -> serde_json::Result<Vec<WsMessage>> {
    let event_type = value.get("event_type").and_then(Value::as_str);

    match event_type {
        Some("book") => {
            if !interest.contains(MessageInterest::BOOK) {
                return Ok(Vec::new());
            }
            serde_json::from_value(value).map(|b| vec![WsMessage::Book(b)])
        }
        Some("price_change") => {
            if !interest.contains(MessageInterest::PRICE_CHANGE) {
                return Ok(Vec::new());
            }
            // Check if it's a batch or single price change
            if value.get("price_changes").is_some() {
                let batch: PriceChangeBatch = serde_json::from_value(value)?;
                Ok(batch
                    .into_price_changes()
                    .into_iter()
                    .map(WsMessage::PriceChange)
                    .collect())
            } else {
                serde_json::from_value(value).map(|p| vec![WsMessage::PriceChange(p)])
            }
        }
        Some("tick_size_change") => {
            if !interest.contains(MessageInterest::TICK_SIZE) {
                return Ok(Vec::new());
            }
            serde_json::from_value(value).map(|t| vec![WsMessage::TickSizeChange(t)])
        }
        Some("last_trade_price") => {
            if !interest.contains(MessageInterest::LAST_TRADE_PRICE) {
                return Ok(Vec::new());
            }
            serde_json::from_value(value).map(|l| vec![WsMessage::LastTradePrice(l)])
        }
        Some("trade") => {
            if !interest.contains(MessageInterest::TRADE) {
                return Ok(Vec::new());
            }
            serde_json::from_value(value).map(|t| vec![WsMessage::Trade(t)])
        }
        Some("order") => {
            if !interest.contains(MessageInterest::ORDER) {
                return Ok(Vec::new());
            }
            serde_json::from_value(value).map(|o| vec![WsMessage::Order(o)])
        }
        _ => {
            // Untagged fallback deserialization for unknown event types
            serde_json::from_value(value).map(|msg| vec![msg])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ApiKey;

    #[test]
    fn parse_book_message() {
        let json = r#"{
            "event_type": "book",
            "asset_id": "123",
            "market": "market1",
            "timestamp": "1234567890",
            "bids": [{"price": "0.5", "size": "100"}],
            "asks": [{"price": "0.51", "size": "50"}]
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Book(book) => {
                assert_eq!(book.asset_id, "123");
                assert_eq!(book.bids.len(), 1);
                assert_eq!(book.asks.len(), 1);
            }
            _ => panic!("Expected Book message"),
        }
    }

    #[test]
    fn parse_price_change_message() {
        let json = r#"{
            "event_type": "price_change",
            "asset_id": "456",
            "market": "market2",
            "price": "0.52",
            "size": "10",
            "side": "BUY",
            "timestamp": "1234567890"
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::PriceChange(price) => {
                assert_eq!(price.asset_id, "456");
                assert_eq!(price.side, Side::Buy);
                assert_eq!(price.size.unwrap(), Decimal::from(10));
            }
            _ => panic!("Expected PriceChange message"),
        }
    }

    #[test]
    fn parse_price_change_batch_message() {
        let json = r#"{
            "event_type": "price_change",
            "market": "market3",
            "timestamp": "1234567890",
            "price_changes": [
                {
                    "asset_id": "asset_a",
                    "price": "0.10",
                    "side": "BUY",
                    "hash": "abc",
                    "best_bid": "0.11",
                    "best_ask": "0.12"
                },
                {
                    "asset_id": "asset_b",
                    "price": "0.90",
                    "size": "5",
                    "side": "SELL"
                }
            ]
        }"#;

        let msgs = parse_ws_text(json, MessageInterest::ALL).unwrap();
        assert_eq!(msgs.len(), 2);

        match &msgs[0] {
            WsMessage::PriceChange(price) => {
                assert_eq!(price.asset_id, "asset_a");
                assert_eq!(price.market, "market3");
                assert_eq!(
                    price.best_bid.unwrap(),
                    Decimal::from_str_exact("0.11").unwrap()
                );
                assert!(price.size.is_none());
            }
            _ => panic!("Expected first price change"),
        }

        match &msgs[1] {
            WsMessage::PriceChange(price) => {
                assert_eq!(price.asset_id, "asset_b");
                assert_eq!(price.size.unwrap(), Decimal::from(5));
                assert_eq!(price.side, Side::Sell);
            }
            _ => panic!("Expected second price change"),
        }
    }

    #[test]
    fn serialize_market_subscription_request() {
        let request = SubscriptionRequest::market(vec!["asset1".to_owned(), "asset2".to_owned()]);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"market\""));
        assert!(json.contains("\"assets_ids\""));
        assert!(json.contains("\"initial_dump\":true"));
    }

    #[test]
    fn serialize_user_subscription_request() {
        let request = SubscriptionRequest::user(
            vec!["market1".to_owned()],
            Credentials::new(
                ApiKey::nil(),
                "test-secret".to_owned(),
                "test-pass".to_owned(),
            ),
        );

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"user\""));
        assert!(json.contains("\"markets\""));
        assert!(json.contains("\"auth\""));
        assert!(json.contains(&format!("\"apiKey\":\"{}\"", ApiKey::nil())));
        assert!(json.contains("\"initial_dump\":true"));
    }
}
