use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use serde_with::{DisplayFromStr, serde_as};

use crate::types::{OrderType, Side, TraderSide};

/// Top-level WebSocket message wrapper.
///
/// All messages received from the WebSocket connection are deserialized into this enum.
/// The message type is determined by the `event_type` field in the JSON.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum WsMessage {
    /// Full or incremental orderbook update
    Book(BookUpdate),
    /// Price change notification
    PriceChange(PriceChange),
    /// Tick size change notification
    TickSizeChange(TickSizeChange),
    /// Last trade price update
    LastTradePrice(LastTradePrice),
    /// User trade execution (authenticated channel)
    Trade(TradeMessage),
    /// User order update (authenticated channel)
    Order(OrderMessage),
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
    /// Unix timestamp in milliseconds (can be string or number)
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
    /// Current bid levels (price descending)
    #[serde(default)]
    pub bids: Vec<OrderBookLevel>,
    /// Current ask levels (price ascending)
    #[serde(default)]
    pub asks: Vec<OrderBookLevel>,
    /// Hash for orderbook validation (if present)
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
    /// Unix timestamp in milliseconds (can be string or number)
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
    /// Unix timestamp in milliseconds (can be string or number)
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
    /// Unix timestamp in milliseconds (can be string or number)
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// User trade execution message (authenticated channel only).
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradeMessage {
    /// Trade identifier
    pub id: String,
    /// Order identifier that was filled
    pub order_id: String,
    /// Market identifier
    pub market: String,
    /// Asset/token identifier
    pub asset_id: String,
    /// Side of the trade (BUY or SELL)
    pub side: Side,
    /// Size of the trade
    pub size: Decimal,
    /// Execution price
    pub price: Decimal,
    /// Fee rate in basis points
    pub fee_rate_bps: u32,
    /// Fee amount
    pub fee: Decimal,
    /// Whether user was maker or taker
    pub trader_side: TraderSide,
    /// Unix timestamp in milliseconds (can be string or number)
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
    /// Trade status (MATCHED, MINED, CONFIRMED, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// User order update message (authenticated channel only).
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderMessage {
    /// Order identifier
    pub id: String,
    /// Market identifier
    pub market: String,
    /// Asset/token identifier
    pub asset_id: String,
    /// Side of the order (BUY or SELL)
    pub side: Side,
    /// Original order size
    pub size: Decimal,
    /// Order price (None for market orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// Order type (GTC, FOK, etc.)
    pub order_type: OrderType,
    /// Current order status
    pub status: OrderStatus,
    /// Unix timestamp in milliseconds (can be string or number)
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
    /// Amount matched so far
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_amount: Option<Decimal>,
}

/// Order status for WebSocket order messages.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    /// Order is open and active
    Open,
    /// Order has been matched with a counterparty
    Matched,
    /// Order has been partially filled
    #[serde(rename = "PARTIALLY_FILLED")]
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
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SubscriptionRequest {
    /// Subscribe to public market data channel
    Market {
        /// List of asset IDs to subscribe to
        assets_ids: Vec<String>,
    },
    /// Subscribe to authenticated user channel
    User {
        /// List of market IDs to subscribe to (empty for all markets)
        markets: Vec<String>,
        /// Authentication credentials
        auth: AuthPayload,
    },
}

/// Authentication payload for user channel subscriptions.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct AuthPayload {
    /// API key (UUID)
    #[serde(rename = "apiKey")]
    pub api_key: String,
    /// API secret (base64-encoded)
    pub secret: String,
    /// API passphrase
    pub passphrase: String,
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
    /// Unix timestamp in milliseconds (can be string or number)
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// Parse a raw WebSocket message string into one or more [`WsMessage`] instances.
pub(crate) fn parse_ws_text(text: &str) -> serde_json::Result<Vec<WsMessage>> {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if trimmed.starts_with('[') {
        let values: Vec<Value> = serde_json::from_str(trimmed)?;
        let mut messages = Vec::new();
        for value in values {
            messages.extend(parse_ws_value(value)?);
        }
        Ok(messages)
    } else {
        parse_ws_value(serde_json::from_str(trimmed)?)
    }
}

fn parse_ws_value(value: Value) -> serde_json::Result<Vec<WsMessage>> {
    if is_price_change_batch(&value) {
        let batch: PriceChangeBatch = serde_json::from_value(value)?;
        Ok(batch
            .into_price_changes()
            .into_iter()
            .map(WsMessage::PriceChange)
            .collect())
    } else {
        serde_json::from_value(value).map(|msg| vec![msg])
    }
}

fn is_price_change_batch(value: &Value) -> bool {
    matches!(
        value.get("event_type").and_then(Value::as_str),
        Some("price_change")
    ) && value.get("price_changes").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_book_message() {
        let json = r#"{
            "event_type": "book",
            "asset_id": "123",
            "market": "market1",
            "timestamp": 1234567890,
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
            "timestamp": 1234567890
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

        let msgs = parse_ws_text(json).unwrap();
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
    fn serialize_subscription_request() {
        let request = SubscriptionRequest::Market {
            assets_ids: vec!["asset1".to_owned(), "asset2".to_owned()],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"market\""));
        assert!(json.contains("\"assets_ids\""));
    }
}
