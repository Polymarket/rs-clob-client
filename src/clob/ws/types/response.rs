use std::fmt;

use bon::Builder;
use serde::de::{IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer as _};
use serde_json::Deserializer;
use serde_with::{DisplayFromStr, NoneAsEmptyString, serde_as};

use crate::auth::ApiKey;
use crate::clob::types::{Side, TraderSide};
use crate::clob::ws::interest::MessageInterest;
use crate::error::Kind;
use crate::types::{B256, Decimal, U256};

/// Top-level WebSocket message wrapper.
///
/// All messages received from the WebSocket connection are deserialized into this enum.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event_type")]
pub enum WsMessage {
    /// Full or incremental orderbook update
    #[serde(rename = "book")]
    Book(BookUpdate),
    /// Price change notification
    #[serde(rename = "price_change")]
    PriceChange(PriceChange),
    /// Tick size change notification
    #[serde(rename = "tick_size_change")]
    TickSizeChange(TickSizeChange),
    /// Last trade price update
    #[serde(rename = "last_trade_price")]
    LastTradePrice(LastTradePrice),
    /// Best bid/ask update (requires `custom_feature_enabled`)
    #[serde(rename = "best_bid_ask")]
    BestBidAsk(BestBidAsk),
    /// New market created (requires `custom_feature_enabled`)
    #[serde(rename = "new_market")]
    NewMarket(NewMarket),
    /// Market resolved (requires `custom_feature_enabled`)
    #[serde(rename = "market_resolved")]
    MarketResolved(MarketResolved),
    /// User trade execution (authenticated channel)
    #[serde(rename = "trade")]
    Trade(TradeMessage),
    /// User order update (authenticated channel)
    #[serde(rename = "order")]
    Order(OrderMessage),
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
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct BookUpdate {
    /// Asset/token identifier
    pub asset_id: U256,
    /// Market condition ID
    pub market: B256,
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
    pub hash: Option<String>,
}

/// Individual price level in an orderbook.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct OrderBookLevel {
    /// Price at this level
    pub price: Decimal,
    /// Total size available at this price
    pub size: Decimal,
}

/// Unified wire format for `price_change` events.
///
/// The server sends either a single price change or a batch. This struct captures both shapes.
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct PriceChange {
    /// Market condition ID
    pub market: B256,
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
    #[serde(default)]
    pub price_changes: Vec<PriceChangeBatchEntry>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct PriceChangeBatchEntry {
    /// Asset/token identifier
    pub asset_id: U256,
    /// New price
    pub price: Decimal,
    /// Total size affected by this price change (if provided)
    #[serde(default)]
    pub size: Option<Decimal>,
    /// Side of the price change (BUY or SELL)
    pub side: Side,
    /// Hash for validation (if present)
    #[serde(default)]
    pub hash: Option<String>,
    /// Best bid price after this change
    #[serde(default)]
    pub best_bid: Option<Decimal>,
    /// Best ask price after this change
    #[serde(default)]
    pub best_ask: Option<Decimal>,
}

/// Tick size change event (triggered when price crosses thresholds).
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct TickSizeChange {
    /// Asset/token identifier
    pub asset_id: U256,
    /// Market condition ID
    pub market: B256,
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
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct LastTradePrice {
    /// Asset/token identifier
    pub asset_id: U256,
    /// Market condition ID
    pub market: B256,
    /// Last trade price
    pub price: Decimal,
    /// Side of the last trade
    pub side: Option<Side>,
    /// Size of the last trade
    pub size: Option<Decimal>,
    /// Fee rate in basis points
    pub fee_rate_bps: Option<Decimal>,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// Best bid/ask update (requires `custom_feature_enabled` flag).
///
/// Emitted when the best bid and ask prices for a market change.
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct BestBidAsk {
    /// Market condition ID
    pub market: B256,
    /// Asset/token identifier
    pub asset_id: U256,
    /// Current best bid price
    pub best_bid: Decimal,
    /// Current best ask price
    pub best_ask: Decimal,
    /// Spread between best bid and ask
    pub spread: Decimal,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// New market created event (requires `custom_feature_enabled` flag).
///
/// Emitted when a new market is created.
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct NewMarket {
    /// Market ID
    pub id: String,
    /// Market question
    pub question: String,
    /// Market condition ID
    pub market: B256,
    /// Market slug
    pub slug: String,
    /// Market description
    pub description: String,
    /// List of asset IDs
    #[serde(rename = "assets_ids")]
    pub asset_ids: Vec<U256>,
    /// List of outcomes (e.g., `["Yes", "No"]`)
    pub outcomes: Vec<String>,
    /// Event message object
    #[serde(default)]
    pub event_message: Option<EventMessage>,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// Market resolved event (requires `custom_feature_enabled` flag).
///
/// Emitted when a market is resolved.
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct MarketResolved {
    /// Market ID
    pub id: String,
    /// Market question
    pub question: String,
    /// Market condition ID
    pub market: B256,
    /// Market slug
    pub slug: String,
    /// Market description
    pub description: String,
    /// List of asset IDs
    #[serde(rename = "assets_ids")]
    pub asset_ids: Vec<U256>,
    /// List of outcomes (e.g., `["Yes", "No"]`)
    pub outcomes: Vec<String>,
    /// Winning asset ID
    pub winning_asset_id: U256,
    /// Winning outcome (e.g., "Yes" or "No")
    pub winning_outcome: String,
    /// Event message object
    #[serde(default)]
    pub event_message: Option<EventMessage>,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// Event message object for market events.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct EventMessage {
    /// Event message ID
    pub id: String,
    /// Event message ticker
    pub ticker: String,
    /// Event message slug
    pub slug: String,
    /// Event message title
    pub title: String,
    /// Event message description
    pub description: String,
}

/// Maker order details within a trade message.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct MakerOrder {
    /// Asset/token identifier of the maker order
    pub asset_id: U256,
    /// Amount of maker order matched in trade
    pub matched_amount: Decimal,
    /// Maker order ID
    pub order_id: String,
    /// Outcome (Yes/No)
    pub outcome: String,
    /// Owner (API key) of maker order
    pub owner: ApiKey,
    /// Price of maker order
    pub price: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
pub enum TradeMessageType {
    #[serde(alias = "trade", alias = "TRADE")]
    Trade,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum TradeMessageStatus {
    #[serde(alias = "matched", alias = "MATCHED")]
    Matched,
    #[serde(alias = "mined", alias = "MINED")]
    Mined,
    #[serde(alias = "confirmed", alias = "CONFIRMED")]
    Confirmed,
    #[serde(untagged)]
    Unknown(String),
}

/// User trade execution message (authenticated channel only).
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct TradeMessage {
    /// Trade identifier
    pub id: String,
    /// Market condition ID
    pub market: B256,
    /// Asset/token identifier
    pub asset_id: U256,
    /// Side of the trade
    pub side: Side,
    /// Size of the trade
    pub size: Decimal,
    /// Execution price
    pub price: Decimal,
    /// Trade status
    pub status: TradeMessageStatus,
    /// Message type
    #[serde(rename = "type", default)]
    pub msg_type: Option<TradeMessageType>,
    /// Timestamp of last trade modification
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub last_update: Option<i64>,
    /// Time trade was matched
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub matchtime: Option<i64>,
    /// Unix timestamp of event
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub timestamp: Option<i64>,
    /// Outcome (Yes/No)
    #[serde(default)]
    pub outcome: Option<String>,
    /// API key of event owner
    #[serde(default)]
    pub owner: Option<ApiKey>,
    /// API key of trade owner
    #[serde(default)]
    pub trade_owner: Option<ApiKey>,
    /// ID of taker order
    #[serde(default)]
    pub taker_order_id: Option<String>,
    /// Array of maker order details
    #[serde(default)]
    pub maker_orders: Vec<MakerOrder>,
    /// Fee rate in basis points (string in API response)
    #[serde(default)]
    pub fee_rate_bps: Option<Decimal>,
    /// On-chain transaction hash
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub transaction_hash: Option<B256>,
    /// Whether user was maker or taker
    #[serde(default)]
    pub trader_side: Option<TraderSide>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum OrderMessageType {
    #[serde(alias = "placement", alias = "PLACEMENT")]
    Placement,
    #[serde(alias = "update", alias = "UPDATE")]
    Update,
    #[serde(alias = "cancellation", alias = "CANCELLATION")]
    Cancellation,
    #[serde(untagged)]
    Unknown(String),
}

/// User order update message (authenticated channel only).
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct OrderMessage {
    /// Order identifier
    pub id: String,
    /// Market condition ID
    pub market: B256,
    /// Asset/token identifier
    pub asset_id: U256,
    /// Side of the order (BUY or SELL)
    pub side: Side,
    /// Order price
    pub price: Decimal,
    /// Message type
    #[serde(rename = "type", default)]
    pub msg_type: Option<OrderMessageType>,
    /// Outcome (Yes/No)
    #[serde(default)]
    pub outcome: Option<String>,
    /// Owner (API key)
    #[serde(default)]
    pub owner: Option<ApiKey>,
    /// Order owner (API key of order originator)
    #[serde(default)]
    pub order_owner: Option<ApiKey>,
    /// Original order size
    #[serde(default)]
    pub original_size: Option<Decimal>,
    /// Amount matched so far
    #[serde(default)]
    pub size_matched: Option<Decimal>,
    /// Unix timestamp of event
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub timestamp: Option<i64>,
    /// Associated trade IDs
    #[serde(default)]
    pub associate_trades: Option<Vec<String>>,
}

/// Order status for WebSocket order messages.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
    /// Unknown order status from the API (captures the raw value for debugging).
    #[serde(untagged)]
    Unknown(String),
}

/// Calculated midpoint update (derived from orderbook).
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
pub struct MidpointUpdate {
    /// Asset/token identifier
    pub asset_id: U256,
    /// Market condition ID
    pub market: B256,
    /// Calculated midpoint price
    pub midpoint: Decimal,
    /// Unix timestamp in milliseconds
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: i64,
}

/// Result of peeking at the message structure without full deserialization.
enum MessageShape {
    /// Single object with the given `event_type` (if present).
    Single(Option<String>),
    /// Array of messages requiring full deserialization.
    Array,
}

/// Peeks at the JSON structure to determine if it's a single object or array,
/// and extracts the `event_type` for single objects without full deserialization.
fn peek_message_shape(bytes: &[u8]) -> Result<MessageShape, serde_json::Error> {
    struct ShapePeeker;

    impl<'de> Visitor<'de> for ShapePeeker {
        type Value = MessageShape;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a JSON object or array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // Consume the entire sequence to avoid "trailing characters" error
            while seq.next_element::<IgnoredAny>()?.is_some() {}
            Ok(MessageShape::Array)
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut event_type: Option<String> = None;
            while let Some(key) = map.next_key::<&str>()? {
                if key == "event_type" {
                    event_type = Some(map.next_value::<String>()?);
                } else {
                    map.next_value::<IgnoredAny>()?;
                }
            }
            Ok(MessageShape::Single(event_type))
        }
    }

    let mut de = Deserializer::from_slice(bytes);
    de.deserialize_any(ShapePeeker)
}

/// Check if a message matches the interest filter.
fn matches_interest(msg: &WsMessage, interest: MessageInterest) -> bool {
    match msg {
        WsMessage::Book(_) => interest.contains(MessageInterest::BOOK),
        WsMessage::PriceChange(_) => interest.contains(MessageInterest::PRICE_CHANGE),
        WsMessage::TickSizeChange(_) => interest.contains(MessageInterest::TICK_SIZE),
        WsMessage::LastTradePrice(_) => interest.contains(MessageInterest::LAST_TRADE_PRICE),
        WsMessage::BestBidAsk(_) => interest.contains(MessageInterest::BEST_BID_ASK),
        WsMessage::NewMarket(_) => interest.contains(MessageInterest::NEW_MARKET),
        WsMessage::MarketResolved(_) => interest.contains(MessageInterest::MARKET_RESOLVED),
        WsMessage::Trade(_) => interest.contains(MessageInterest::TRADE),
        WsMessage::Order(_) => interest.contains(MessageInterest::ORDER),
    }
}

/// Deserialize messages from the byte slice, filtering by interest.
///
/// For single objects, the `event_type` is extracted first to skip uninteresting messages
/// without full deserialization. For arrays, all messages are deserialized and filtered.
pub fn parse_if_interested(
    bytes: &[u8],
    interest: &MessageInterest,
) -> crate::Result<Vec<WsMessage>> {
    let shape = peek_message_shape(bytes)
        .map_err(|e| crate::error::Error::with_source(Kind::Internal, Box::new(e)))?;

    match shape {
        MessageShape::Single(None) => Ok(vec![]),
        MessageShape::Single(Some(event_type)) => {
            if !interest.is_interested_in_event(&event_type) {
                return Ok(vec![]);
            }
            let msg: WsMessage = serde_json::from_slice(bytes)?;
            Ok(vec![msg])
        }
        MessageShape::Array => {
            let messages: Vec<WsMessage> = serde_json::from_slice(bytes)?;
            Ok(messages
                .into_iter()
                .filter(|msg| matches_interest(msg, *interest))
                .collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use rust_decimal_macros::dec;

    use super::*;
    use crate::types::b256;

    // Test market condition ID
    const TEST_MARKET: B256 =
        b256!("0000000000000000000000000000000000000000000000000000000000000001");

    #[test]
    fn parse_book_message() {
        let json = r#"{
            "event_type": "book",
            "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
            "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "timestamp": "1234567890",
            "bids": [{"price": "0.5", "size": "100"}],
            "asks": [{"price": "0.51", "size": "50"}]
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Book(book) => {
                assert_eq!(book.asset_id, U256::from_str("106585164761922456203746651621390029417453862034640469075081961934906147433548").unwrap());
                assert_eq!(book.market, TEST_MARKET);
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
            "market": "0x0000000000000000000000000000000000000000000000000000000000000002",
            "timestamp": "1234567890",
            "price_changes": [{
                "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                "price": "0.52",
                "size": "10",
                "side": "BUY"
            }]
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::PriceChange(price) => {
                let changes = &price.price_changes[0];

                assert_eq!(changes.asset_id, U256::from_str("106585164761922456203746651621390029417453862034640469075081961934906147433548").unwrap());
                assert_eq!(changes.side, Side::Buy);
                assert_eq!(changes.size.unwrap(), Decimal::TEN);
            }
            _ => panic!("Expected PriceChange message"),
        }
    }

    #[test]
    fn parse_price_change_interest_message() {
        let json = r#"{
            "event_type": "price_change",
            "market": "0x0000000000000000000000000000000000000000000000000000000000000003",
            "timestamp": "1234567890",
            "price_changes": [
                {
                    "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                    "price": "0.10",
                    "side": "BUY",
                    "hash": "abc",
                    "best_bid": "0.11",
                    "best_ask": "0.12"
                },
                {
                    "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                    "price": "0.90",
                    "size": "5",
                    "side": "SELL"
                }
            ]
        }"#;

        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::ALL).unwrap();
        assert_eq!(msgs.len(), 1);

        match &msgs[0] {
            WsMessage::PriceChange(price) => {
                let expected =
                    b256!("0000000000000000000000000000000000000000000000000000000000000003");
                assert_eq!(price.market, expected);

                let changes = &price.price_changes;
                assert_eq!(changes.len(), 2);

                assert_eq!(changes[0].asset_id, U256::from_str("106585164761922456203746651621390029417453862034640469075081961934906147433548").unwrap());
                assert_eq!(changes[0].best_bid, Some(dec!(0.11)));
                assert_eq!(changes[0].price, dec!(0.10));
                assert!(changes[0].size.is_none());

                assert_eq!(changes[1].asset_id, U256::from_str("106585164761922456203746651621390029417453862034640469075081961934906147433548").unwrap());
                assert_eq!(changes[1].best_bid, None);
                assert_eq!(changes[1].size, Some(dec!(5)));
                assert_eq!(changes[1].price, dec!(0.90));
            }
            _ => panic!("Expected first price change"),
        }
    }

    #[test]
    fn parse_batch_messages() {
        let json = r#"[
            {
                "event_type": "book",
                "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "timestamp": "1234567890",
                "bids": [{"price": "0.5", "size": "100"}],
                "asks": []
            },
            {
                "event_type": "price_change",
                "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "timestamp": "1234567891",
                "price_changes": [{
                    "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                    "price": "0.51",
                    "side": "BUY"
                }]
            },
            {
                "event_type": "last_trade_price",
                "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "price": "0.6",
                "timestamp": "1234567892"
            }
        ]"#;

        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::ALL).unwrap();
        assert_eq!(msgs.len(), 3);

        assert!(
            matches!(&msgs[0], WsMessage::Book(b) if b.asset_id == U256::from_str("106585164761922456203746651621390029417453862034640469075081961934906147433548").unwrap())
        );
        assert!(matches!(&msgs[1], WsMessage::PriceChange(p) if p.market == TEST_MARKET));
        assert!(
            matches!(&msgs[2], WsMessage::LastTradePrice(l) if l.asset_id == U256::from_str("106585164761922456203746651621390029417453862034640469075081961934906147433548").unwrap())
        );
    }

    #[test]
    fn parse_batch_filters_by_interest() {
        let json = r#"[
            {
                "event_type": "book",
                "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "timestamp": "1234567890",
                "bids": [],
                "asks": []
            },
            {
                "event_type": "trade",
                "id": "trade1",
                "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                "side": "BUY",
                "size": "10",
                "price": "0.5",
                "status": "MATCHED"
            }
        ]"#;

        // Only interested in BOOK, not TRADE
        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::BOOK).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(&msgs[0], WsMessage::Book(_)));

        // Only interested in TRADE, not BOOK
        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::TRADE).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(&msgs[0], WsMessage::Trade(_)));

        // Interested in both
        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::ALL).unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn parse_best_bid_ask_message() {
        let json = r#"{
            "event_type": "best_bid_ask",
            "market": "0x0005c0d312de0be897668695bae9f32b624b4a1ae8b140c49f08447fcc74f442",
            "asset_id": "85354956062430465315924116860125388538595433819574542752031640332592237464430",
            "best_bid": "0.73",
            "best_ask": "0.77",
            "spread": "0.04",
            "timestamp": "1766789469958"
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::BestBidAsk(bba) => {
                assert_eq!(bba.best_bid, dec!(0.73));
                assert_eq!(bba.best_ask, dec!(0.77));
                assert_eq!(bba.spread, dec!(0.04));
            }
            _ => panic!("Expected BestBidAsk message"),
        }
    }

    #[test]
    fn parse_new_market_message() {
        let json = r#"{
            "id": "1031769",
            "question": "Will NVIDIA (NVDA) close above $240 end of January?",
            "market": "0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1",
            "slug": "nvda-above-240-on-january-30-2026",
            "description": "This market will resolve to Yes or No.",
            "assets_ids": [
                "76043073756653678226373981964075571318267289248134717369284518995922789326425",
                "31690934263385727664202099278545688007799199447969475608906331829650099442770"
            ],
            "outcomes": ["Yes", "No"],
            "event_message": {
                "id": "125819",
                "ticker": "nvda-above-in-january-2026",
                "slug": "nvda-above-in-january-2026",
                "title": "Will NVIDIA (NVDA) close above ___ end of January?",
                "description": "Market description"
            },
            "timestamp": "1766790415550",
            "event_type": "new_market"
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::NewMarket(nm) => {
                assert_eq!(nm.id, "1031769");
                assert_eq!(
                    nm.question,
                    "Will NVIDIA (NVDA) close above $240 end of January?"
                );
                assert_eq!(nm.asset_ids.len(), 2);
                assert_eq!(nm.outcomes, vec!["Yes", "No"]);
                assert!(nm.event_message.is_some());
                let event = nm.event_message.unwrap();
                assert_eq!(event.id, "125819");
                assert_eq!(event.ticker, "nvda-above-in-january-2026");
            }
            _ => panic!("Expected NewMarket message"),
        }
    }

    #[test]
    fn parse_market_resolved_message() {
        let json = r#"{
            "id": "1031769",
            "question": "Will NVIDIA (NVDA) close above $240 end of January?",
            "market": "0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1",
            "slug": "nvda-above-240-on-january-30-2026",
            "description": "This market will resolve to Yes or No.",
            "assets_ids": [
                "76043073756653678226373981964075571318267289248134717369284518995922789326425",
                "31690934263385727664202099278545688007799199447969475608906331829650099442770"
            ],
            "outcomes": ["Yes", "No"],
            "winning_asset_id": "76043073756653678226373981964075571318267289248134717369284518995922789326425",
            "winning_outcome": "Yes",
            "event_message": {
                "id": "125819",
                "ticker": "nvda-above-in-january-2026",
                "slug": "nvda-above-in-january-2026",
                "title": "Will NVIDIA (NVDA) close above ___ end of January?",
                "description": "Market description"
            },
            "timestamp": "1766790415550",
            "event_type": "market_resolved"
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::MarketResolved(mr) => {
                assert_eq!(mr.id, "1031769");
                assert_eq!(mr.winning_outcome, "Yes");
                assert_eq!(
                    mr.winning_asset_id,
                    U256::from_str("76043073756653678226373981964075571318267289248134717369284518995922789326425").unwrap()
                );
                assert_eq!(mr.asset_ids.len(), 2);
            }
            _ => panic!("Expected MarketResolved message"),
        }
    }

    #[test]
    fn parse_last_trade_price_with_new_fields() {
        let json = r#"{
            "asset_id": "114122071509644379678018727908709560226618148003371446110114509806601493071694",
            "event_type": "last_trade_price",
            "fee_rate_bps": "0",
            "market": "0x6a67b9d828d53862160e470329ffea5246f338ecfffdf2cab45211ec578b0347",
            "price": "0.456",
            "side": "BUY",
            "size": "219.217767",
            "timestamp": "1750428146322"
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::LastTradePrice(ltp) => {
                assert_eq!(ltp.price, dec!(0.456));
                assert_eq!(ltp.size, Some(dec!(219.217767)));
                assert_eq!(ltp.fee_rate_bps, Some(Decimal::ZERO));
                assert_eq!(ltp.side, Some(Side::Buy));
            }
            _ => panic!("Expected LastTradePrice message"),
        }
    }

    #[test]
    fn parse_custom_feature_messages_filter_by_interest() {
        let json = r#"[
            {
                "event_type": "best_bid_ask",
                "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                "best_bid": "0.5",
                "best_ask": "0.6",
                "spread": "0.1",
                "timestamp": "1234567890"
            },
            {
                "event_type": "book",
                "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
                "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "timestamp": "1234567890",
                "bids": [],
                "asks": []
            }
        ]"#;

        // Only interested in BEST_BID_ASK
        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::BEST_BID_ASK).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(&msgs[0], WsMessage::BestBidAsk(_)));

        // Only interested in BOOK
        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::BOOK).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(&msgs[0], WsMessage::Book(_)));

        // MARKET includes both
        let msgs = parse_if_interested(json.as_bytes(), &MessageInterest::MARKET).unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn parse_new_market_without_event_message() {
        let json = r#"{
            "id": "1031769",
            "question": "Will NVIDIA (NVDA) close above $240 end of January?",
            "market": "0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1",
            "slug": "nvda-above-240-on-january-30-2026",
            "description": "This market will resolve to Yes or No.",
            "assets_ids": ["106585164761922456203746651621390029417453862034640469075081961934906147433548", "106585164761922456203746651621390029417453862034640469075081961934906147433548"],
            "outcomes": ["Yes", "No"],
            "timestamp": "1766790415550",
            "event_type": "new_market"
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::NewMarket(nm) => {
                assert_eq!(nm.id, "1031769");
                assert!(nm.event_message.is_none());
            }
            _ => panic!("Expected NewMarket message"),
        }
    }

    #[test]
    fn parse_market_resolved_without_event_message() {
        let json = r#"{
            "id": "1031769",
            "question": "Will NVIDIA (NVDA) close above $240 end of January?",
            "market": "0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1",
            "slug": "nvda-above-240-on-january-30-2026",
            "description": "This market will resolve to Yes or No.",
            "assets_ids": ["106585164761922456203746651621390029417453862034640469075081961934906147433548", "106585164761922456203746651621390029417453862034640469075081961934906147433548"],
            "outcomes": ["Yes", "No"],
            "winning_asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
            "winning_outcome": "Yes",
            "timestamp": "1766790415550",
            "event_type": "market_resolved"
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::MarketResolved(mr) => {
                assert_eq!(mr.id, "1031769");
                assert!(mr.event_message.is_none());
                assert_eq!(mr.winning_outcome, "Yes");
            }
            _ => panic!("Expected MarketResolved message"),
        }
    }

    #[test]
    fn parse_last_trade_price_without_optional_fields() {
        let json = r#"{
            "asset_id": "106585164761922456203746651621390029417453862034640469075081961934906147433548",
            "event_type": "last_trade_price",
            "market": "0x0000000000000000000000000000000000000000000000000000000000000123",
            "price": "0.5",
            "timestamp": "1750428146322"
        }"#;

        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::LastTradePrice(ltp) => {
                assert_eq!(ltp.price, dec!(0.5));
                assert!(ltp.size.is_none());
                assert!(ltp.fee_rate_bps.is_none());
                assert!(ltp.side.is_none());
            }
            _ => panic!("Expected LastTradePrice message"),
        }
    }

    #[test]
    fn matches_interest_custom_feature_messages() {
        let bba = WsMessage::BestBidAsk(BestBidAsk {
            market: TEST_MARKET,
            asset_id: U256::from_str(
                "106585164761922456203746651621390029417453862034640469075081961934906147433548",
            )
            .unwrap(),
            best_bid: dec!(0.5),
            best_ask: dec!(0.6),
            spread: dec!(0.1),
            timestamp: 0,
        });
        assert!(matches_interest(&bba, MessageInterest::BEST_BID_ASK));
        assert!(!matches_interest(&bba, MessageInterest::BOOK));
        assert!(matches_interest(&bba, MessageInterest::MARKET));

        let nm = WsMessage::NewMarket(NewMarket {
            id: "1".to_owned(),
            question: "q".to_owned(),
            market: TEST_MARKET,
            slug: "s".to_owned(),
            description: "d".to_owned(),
            asset_ids: vec![],
            outcomes: vec![],
            event_message: None,
            timestamp: 0,
        });
        assert!(matches_interest(&nm, MessageInterest::NEW_MARKET));
        assert!(matches_interest(&nm, MessageInterest::MARKET));

        let mr = WsMessage::MarketResolved(MarketResolved {
            id: "1".to_owned(),
            question: "q".to_owned(),
            market: TEST_MARKET,
            slug: "s".to_owned(),
            description: "d".to_owned(),
            asset_ids: vec![],
            outcomes: vec![],
            winning_asset_id: U256::from_str(
                "106585164761922456203746651621390029417453862034640469075081961934906147433548",
            )
            .unwrap(),
            winning_outcome: "Yes".to_owned(),
            event_message: None,
            timestamp: 0,
        });
        assert!(matches_interest(&mr, MessageInterest::MARKET_RESOLVED));
        assert!(matches_interest(&mr, MessageInterest::MARKET));
    }
}
