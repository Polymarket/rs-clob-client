use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use alloy::core::sol;
use alloy::primitives::{Address, U256};
use alloy::signers::Signature;
use chrono::{DateTime, NaiveDate, Utc};
use derive_builder::Builder;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive as _;
use rust_decimal_macros::dec;
use serde::de::Visitor;
use serde::ser::{Error as _, SerializeStruct as _};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value;
use serde_with::{
    DefaultOnNull, DisplayFromStr, FromInto, TimestampMilliSeconds, TimestampSeconds, serde_as,
};
use sha2::{Digest as _, Sha256};
use strum_macros::Display;
use uuid::Uuid;

use crate::Result;
use crate::auth::ApiKey;
use crate::clob::order_builder::{LOT_SIZE_SCALE, USDC_DECIMALS};
use crate::error::Error;

type OrderId = String;
type TradeId = String;

#[non_exhaustive]
#[derive(
    Clone, Copy, Debug, Display, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub enum OrderType {
    /// Good 'til Cancelled; If not fully filled, the order rests on the book until it is explicitly
    /// cancelled.
    #[serde(alias = "gtc")]
    GTC,
    /// Fill or Kill; Order is attempted to be filled, in full, immediately. If it cannot be fully
    /// filled, the entire order is cancelled.
    #[default]
    #[serde(alias = "fok")]
    FOK,
    /// Good 'til Date; If not fully filled, the order rests on the book until the specified date.
    #[serde(alias = "gtd")]
    GTD,
    /// Fill and Kill; Order is attempted to be filled, however much is possible, immediately. If
    /// the order cannot be fully filled, the remaining quantity is cancelled.
    #[serde(alias = "fak")]
    FAK,
    #[serde(other)]
    Unknown,
}

#[non_exhaustive]
#[derive(
    Clone, Copy, Debug, Display, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[repr(u8)]
pub enum Side {
    #[serde(alias = "buy")]
    Buy = 0,
    #[serde(alias = "sell")]
    Sell = 1,
    #[serde(other)]
    Unknown = 255,
}

impl TryFrom<u8> for Side {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Side::Buy),
            1 => Ok(Side::Sell),
            other => Err(Error::validation(format!(
                "Unable to create Side from {other}"
            ))),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AmountInner {
    Usdc(Decimal),
    Shares(Decimal),
}

impl AmountInner {
    pub fn as_inner(&self) -> Decimal {
        match self {
            AmountInner::Usdc(d) | AmountInner::Shares(d) => *d,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Amount(pub(crate) AmountInner);

impl Amount {
    pub fn usdc(value: Decimal) -> Result<Amount> {
        let normalized = value.normalize();
        if normalized.scale() > USDC_DECIMALS {
            return Err(Error::validation(format!(
                "Unable to build Amount with {} decimal points, must be <= {USDC_DECIMALS}",
                normalized.scale()
            )));
        }

        Ok(Amount(AmountInner::Usdc(normalized)))
    }

    pub fn shares(value: Decimal) -> Result<Amount> {
        let normalized = value.normalize();
        if normalized.scale() > LOT_SIZE_SCALE {
            return Err(Error::validation(format!(
                "Unable to build Amount with {} decimal points, must be <= {LOT_SIZE_SCALE}",
                normalized.scale()
            )));
        }

        Ok(Amount(AmountInner::Shares(normalized)))
    }

    #[must_use]
    pub fn as_inner(&self) -> Decimal {
        self.0.as_inner()
    }

    #[must_use]
    pub fn is_usdc(&self) -> bool {
        matches!(self.0, AmountInner::Usdc(_))
    }

    #[must_use]
    pub fn is_shares(&self) -> bool {
        matches!(self.0, AmountInner::Shares(_))
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Display, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum SignatureType {
    #[default]
    Eoa = 0,
    Proxy = 1,
    GnosisSafe = 2,
}

#[non_exhaustive]
#[derive(Clone, Copy, Display, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum OrderStatusType {
    #[serde(alias = "live")]
    Live,

    #[serde(alias = "matched")]
    Matched,

    #[serde(alias = "canceled")]
    Canceled,

    #[serde(alias = "delayed")]
    Delayed,

    #[serde(alias = "unmatched")]
    Unmatched,

    #[serde(other)]
    Unknown,
}

#[non_exhaustive]
#[derive(
    Clone, Copy, Debug, Default, Display, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum AssetType {
    #[default]
    Collateral,
    Conditional,
    #[serde(other)]
    Unknown,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TraderSide {
    Taker,
    Maker,
    #[serde(other)]
    Unknown,
}

/// Represents the maximum number of decimal places for an order's price field
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum TickSize {
    Tenth,
    Hundredth,
    Thousandth,
    TenThousandth,
}

impl fmt::Display for TickSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TickSize::Tenth => "Tenth",
            TickSize::Hundredth => "Hundredth",
            TickSize::Thousandth => "Thousandth",
            TickSize::TenThousandth => "TenThousandth",
        };

        write!(f, "{name}({})", self.as_decimal())
    }
}

impl TickSize {
    #[must_use]
    pub fn as_decimal(&self) -> Decimal {
        match self {
            TickSize::Tenth => dec!(0.1),
            TickSize::Hundredth => dec!(0.01),
            TickSize::Thousandth => dec!(0.001),
            TickSize::TenThousandth => dec!(0.0001),
        }
    }
}

impl From<TickSize> for Decimal {
    fn from(tick_size: TickSize) -> Self {
        tick_size.as_decimal()
    }
}

impl From<Decimal> for TickSize {
    fn from(value: Decimal) -> Self {
        match value {
            v if v == dec!(0.1) => TickSize::Tenth,
            v if v == dec!(0.01) => TickSize::Hundredth,
            v if v == dec!(0.001) => TickSize::Thousandth,
            v if v == dec!(0.0001) => TickSize::TenThousandth,
            other => panic!("Unable to convert {other:?} to TickSize"),
        }
    }
}

impl PartialEq for TickSize {
    fn eq(&self, other: &Self) -> bool {
        self.as_decimal() == other.as_decimal()
    }
}

impl<'de> Deserialize<'de> for TickSize {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dec = <Decimal as Deserialize>::deserialize(deserializer)?;
        Ok(TickSize::from(dec))
    }
}

sol! {
    /// Alloy solidity type representing an order in the context of the Polymarket exchange
    ///
    /// <!-- The CLOB expects all `uint256` types, [`U256`], excluding `salt`, to be presented as a
    /// string so we must serialize as Display, which for U256 is lower hex-encoded string.
    /// -->
    #[non_exhaustive]
    #[serde_as]
    #[derive(Serialize, Debug, Default, PartialEq)]
    struct Order {
        #[serde(serialize_with = "ser_salt")]
        uint256 salt;
        address maker;
        address signer;
        address taker;
        #[serde_as(as = "DisplayFromStr")]
        uint256 tokenId;
        #[serde_as(as = "DisplayFromStr")]
        uint256 makerAmount;
        #[serde_as(as = "DisplayFromStr")]
        uint256 takerAmount;
        #[serde_as(as = "DisplayFromStr")]
        uint256 expiration;
        #[serde_as(as = "DisplayFromStr")]
        uint256 nonce;
        #[serde_as(as = "DisplayFromStr")]
        uint256 feeRateBps;
        uint8   side;
        uint8   signatureType;
    }
}

// CLOB expects salt as a JSON number. U256 as an integer will not fit as a JSON number. Since
// we generated the salt as a u64 originally (see `salt_generator`), we can be very confident that
// we can invert the conversion to U256 and return a u64 when serializing.
fn ser_salt<S: Serializer>(value: &U256, serializer: S) -> std::result::Result<S::Ok, S::Error> {
    let v: u64 = value
        .try_into()
        .map_err(|e| S::Error::custom(format!("salt does not fit into u64: {e}")))?;
    serializer.serialize_u64(v)
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Serialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct SignableOrder {
    pub order: Order,
    pub order_type: OrderType,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct MidpointRequest {
    pub token_id: String,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct MidpointResponse {
    pub mid: Decimal,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[serde(transparent)]
pub struct MidpointsResponse {
    pub midpoints: HashMap<String, Decimal>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct PriceRequest {
    pub token_id: String,
    pub side: Side,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct PriceResponse {
    pub price: Decimal,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(strip_option))]
#[serde(transparent)]
pub struct PricesResponse {
    pub prices: Option<HashMap<String, HashMap<Side, Decimal>>>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct SpreadRequest {
    pub token_id: String,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct SpreadResponse {
    pub spread: Decimal,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(strip_option))]
pub struct SpreadsResponse {
    pub spreads: Option<HashMap<String, Decimal>>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct TickSizeResponse {
    pub minimum_tick_size: TickSize,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct NegRiskResponse {
    pub neg_risk: bool,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct FeeRateResponse {
    pub base_fee: u32,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct OrderBookSummaryRequest {
    pub token_id: String,
}

#[non_exhaustive]
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct OrderBookSummaryResponse {
    pub market: String,
    pub asset_id: String,
    #[serde_as(as = "TimestampMilliSeconds<String>")]
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    #[builder(default)]
    #[builder(setter(into, strip_option))]
    pub hash: Option<String>,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub bids: Vec<OrderSummary>,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub asks: Vec<OrderSummary>,
    pub min_order_size: Decimal,
    pub neg_risk: bool,
    #[serde_as(as = "FromInto<Decimal>")]
    pub tick_size: TickSize,
}

impl OrderBookSummaryResponse {
    pub fn hash(&self) -> Result<String> {
        let json = serde_json::to_string(&self)?;

        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let result = hasher.finalize();

        Ok(format!("{result:x}"))
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, Hash, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct OrderSummary {
    pub price: Decimal,
    pub size: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct LastTradePriceRequest {
    pub token_id: String,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct LastTradePriceResponse {
    pub price: Decimal,
    pub side: Side,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct LastTradesPricesResponse {
    pub token_id: String,
    pub price: Decimal,
    pub side: Side,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "The current API has these fields, so we have to capture this"
)]
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Deserialize, Clone, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(strip_option))]
pub struct MarketResponse {
    pub enable_order_book: bool,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub accepting_orders: bool,
    #[builder(setter(into))]
    #[builder(default)]
    pub accepting_order_timestamp: Option<DateTime<Utc>>,
    pub minimum_order_size: Decimal,
    pub minimum_tick_size: Decimal,
    #[builder(setter(into))]
    pub condition_id: String,
    #[builder(setter(into))]
    pub question_id: String,
    #[builder(setter(into))]
    pub question: String,
    #[builder(setter(into))]
    pub description: String,
    #[builder(setter(into))]
    pub market_slug: String,
    #[builder(setter(into))]
    #[builder(default)]
    pub end_date_iso: Option<DateTime<Utc>>,
    #[builder(setter(into))]
    #[builder(default)]
    pub game_start_time: Option<DateTime<Utc>>,
    pub seconds_delay: u64,
    #[builder(setter(into))]
    pub fpmm: String,
    pub maker_base_fee: Decimal,
    pub taker_base_fee: Decimal,
    pub notifications_enabled: bool,
    pub neg_risk: bool,
    #[builder(setter(into))]
    pub neg_risk_market_id: String,
    #[builder(setter(into))]
    pub neg_risk_request_id: String,
    #[builder(setter(into))]
    pub icon: String,
    #[builder(setter(into))]
    pub image: String,
    pub rewards: Rewards,
    pub is_50_50_outcome: bool,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub tokens: Vec<Token>,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub tags: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
pub struct Token {
    pub token_id: String,
    pub outcome: String,
    pub price: Decimal,
    #[serde(default)]
    pub winner: bool,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "The current API has these fields"
)]
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Deserialize, Clone, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
pub struct SimplifiedMarketResponse {
    pub condition_id: String,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub tokens: Vec<Token>,
    pub rewards: Rewards,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub accepting_orders: bool,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(strip_option))]
#[builder(default)]
pub struct ApiKeysResponse {
    #[serde(rename = "apiKeys")]
    keys: Option<Vec<Uuid>>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct BanStatusResponse {
    pub closed_only: bool,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct PostOrderResponse {
    #[builder(default)]
    #[builder(setter(into, strip_option))]
    pub error_msg: Option<String>,
    #[serde(deserialize_with = "empty_string_as_zero")]
    pub making_amount: Decimal,
    #[serde(deserialize_with = "empty_string_as_zero")]
    pub taking_amount: Decimal,
    #[serde(rename = "orderID")]
    #[builder(setter(into))]
    pub order_id: String,
    pub status: OrderStatusType,
    pub success: bool,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub transaction_hashes: Vec<String>,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub trade_ids: Vec<String>,
}

pub fn empty_string_as_zero<'de, D>(deserializer: D) -> std::result::Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    if s.trim().is_empty() {
        Ok(Decimal::ZERO)
    } else {
        s.parse::<Decimal>().map_err(serde::de::Error::custom)
    }
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct OpenOrderResponse {
    #[builder(setter(into))]
    pub id: OrderId,
    pub status: OrderStatusType,
    pub owner: ApiKey,
    pub maker_address: Address,
    #[builder(setter(into))]
    pub market: String,
    #[builder(setter(into))]
    pub asset_id: String,
    pub side: Side,
    pub original_size: Decimal,
    pub size_matched: Decimal,
    pub price: Decimal,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub associate_trades: Vec<String>,
    #[builder(setter(into))]
    pub outcome: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde_as(as = "TimestampSeconds<String>")]
    pub expiration: DateTime<Utc>,
    pub order_type: OrderType,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct CancelOrdersResponse {
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub canceled: Vec<OrderId>,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub not_canceled: HashMap<OrderId, String>,
}

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
#[builder(default)]
pub struct CancelMarketOrderRequest {
    pub market: Option<String>,
    pub asset_id: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
#[builder(default)]
pub struct TradesRequest {
    pub id: Option<String>,
    pub maker_address: Option<Address>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub before: Option<i64>,
    pub after: Option<i64>,
}

impl TradesRequest {
    pub(crate) fn as_params(&self, next_cursor: Option<&String>) -> String {
        let id = self.id.as_ref().map(|o| format!("id={o}"));
        let maker_address = self
            .maker_address
            .as_ref()
            .map(|m| format!("maker_address={m}"));
        let market = self.market.as_ref().map(|a| format!("market={a}"));
        let asset_id = self.asset_id.as_ref().map(|a| format!("asset_id={a}"));
        let before = self.before.as_ref().map(|a| format!("before={a}"));
        let after = self.after.as_ref().map(|a| format!("after={a}"));

        let params = [id, maker_address, market, asset_id, before, after]
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join("&");

        format_params_with_cursor(params.as_str(), next_cursor)
    }
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct TradeResponse {
    #[builder(setter(into))]
    pub id: TradeId,
    #[builder(setter(into))]
    pub taker_order_id: String,
    #[builder(setter(into))]
    pub market: String,
    #[builder(setter(into))]
    pub asset_id: String,
    pub side: Side,
    pub size: Decimal,
    pub fee_rate_bps: Decimal,
    pub price: Decimal,
    pub status: OrderStatusType,
    #[serde_as(as = "TimestampSeconds<String>")]
    pub match_time: DateTime<Utc>,
    #[serde_as(as = "TimestampSeconds<String>")]
    pub last_update: DateTime<Utc>,
    #[builder(setter(into))]
    pub outcome: String,
    pub bucket_index: u32,
    pub owner: ApiKey,
    pub maker_address: Address,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub maker_orders: Vec<MakerOrder>,
    #[builder(setter(into))]
    pub transaction_hash: String,
    pub trader_side: TraderSide,
    #[serde(default)]
    #[builder(default)]
    #[builder(setter(into, strip_option))]
    pub error_msg: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
#[builder(default)]
pub struct OrdersRequest {
    pub order_id: Option<OrderId>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
}

impl OrdersRequest {
    pub(crate) fn as_params(&self, next_cursor: Option<&String>) -> String {
        let order_id = self.order_id.as_ref().map(|o| format!("order_id={o}"));
        let market = self.market.as_ref().map(|m| format!("market={m}"));
        let asset_id = self.asset_id.as_ref().map(|a| format!("asset_id={a}"));

        let params = [order_id, market, asset_id]
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join("&");

        format_params_with_cursor(params.as_str(), next_cursor)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct NotificationResponse {
    pub r#type: u32,
    pub owner: ApiKey,
    pub payload: NotificationPayload,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct NotificationPayload {
    #[builder(setter(into))]
    pub asset_id: String,
    #[builder(setter(into))]
    pub condition_id: String,
    #[serde(rename = "eventSlug")]
    #[builder(setter(into))]
    pub event_slug: String,
    #[builder(setter(into))]
    pub icon: String,
    #[builder(setter(into))]
    pub image: String,
    #[builder(setter(into))]
    pub market: String,
    #[builder(setter(into))]
    pub market_slug: String,
    #[builder(setter(into))]
    pub matched_size: Decimal,
    #[builder(setter(into))]
    pub order_id: OrderId,
    #[builder(setter(into))]
    pub original_size: Decimal,
    #[builder(setter(into))]
    pub outcome: String,
    pub outcome_index: u64,
    pub owner: ApiKey,
    #[builder(setter(into))]
    pub price: Decimal,
    #[builder(setter(into))]
    pub question: String,
    #[builder(setter(into))]
    pub remaining_size: Decimal,
    #[serde(rename = "seriesSlug")]
    #[builder(setter(into))]
    pub series_slug: String,
    pub side: Side,
    #[builder(setter(into))]
    pub trade_id: String,
    #[builder(setter(into))]
    pub transaction_hash: String,
    #[serde(alias = "type")]
    pub order_type: OrderType,
}

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
#[builder(default)]
pub struct DeleteNotificationsRequest {
    pub notification_ids: Option<Vec<String>>,
}

impl DeleteNotificationsRequest {
    pub(crate) fn as_params(&self) -> String {
        self.notification_ids.as_ref().map_or(String::new(), |ids| {
            if ids.is_empty() {
                String::new()
            } else {
                format!("?ids={}", ids.join(","))
            }
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
pub struct BalanceAllowanceRequest {
    pub asset_type: AssetType,
    #[builder(default)]
    pub token_id: Option<String>,
    #[builder(default)]
    pub signature_type: Option<SignatureType>,
}

impl BalanceAllowanceRequest {
    pub(crate) fn as_params(&self, default_signature_type: SignatureType) -> String {
        let token_id = self.token_id.as_ref().map(|m| format!("token_id={m}"));
        let signature_type = self.signature_type.unwrap_or(default_signature_type);

        let signature_type = format!("signature_type={}", signature_type as u8);

        let params = [
            Some(format!("asset_type={}", self.asset_type)),
            token_id,
            Some(signature_type),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join("&");

        if params.is_empty() {
            String::new()
        } else {
            format!("?{params}")
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct BalanceAllowanceResponse {
    pub balance: Decimal,
    #[serde(default)]
    #[builder(default)]
    pub allowances: HashMap<Address, String>,
}

pub type UpdateBalanceAllowanceRequest = BalanceAllowanceRequest;

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct OrderScoringResponse {
    pub scoring: bool,
}

pub type OrdersScoringResponse = HashMap<String, bool>;

#[non_exhaustive]
#[derive(Debug, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct SignedOrder {
    pub order: Order,
    pub signature: Signature,
    pub order_type: OrderType,
    pub owner: ApiKey,
}

// CLOB expects a struct that has the `signature` "folded" into the `order` key
impl Serialize for SignedOrder {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut st = serializer.serialize_struct("SignedOrder", 3)?;

        let mut order = serde_json::to_value(&self.order).map_err(serde::ser::Error::custom)?;

        // inject signature into order object
        if let Value::Object(ref mut map) = order {
            map.insert(
                "signature".to_owned(),
                Value::String(self.signature.to_string()),
            );
        }

        // Side has to be serialized as "BUY" or "SELL" when hitting the CLOB, but the actual
        // signature for a SignedOrder has to be done on the integer representation.
        if let Some(value) = order.get_mut("side")
            && let Some(side_numeric) = value.as_u64()
            && let Some(side_numeric) = side_numeric.to_u8()
            && let Ok(side) = Side::try_from(side_numeric)
        {
            *value = Value::String(side.to_string());
        }

        st.serialize_field("order", &order)?;
        st.serialize_field("orderType", &self.order_type)?;
        st.serialize_field("owner", &self.owner)?;

        st.end()
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct PriceSideResponse {
    pub side: Side,
    pub price: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct RewardRate {
    pub asset_address: Address,
    pub rewards_daily_rate: Decimal,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
pub struct Rewards {
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub rates: Vec<RewardRate>,
    pub min_size: Decimal,
    pub max_spread: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct MarketInfo {
    pub condition_id: String,
    pub asset_id: String,
    pub question: String,
    pub icon: String,
    pub slug: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
pub struct UserInfo {
    pub address: Address,
    pub username: String,
    pub profile_picture: String,
    pub optimized_profile_picture: String,
    pub pseudonym: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct MakerOrder {
    #[builder(setter(into))]
    pub order_id: OrderId,
    pub owner: ApiKey,
    pub maker_address: Address,
    pub matched_amount: Decimal,
    pub price: Decimal,
    pub fee_rate_bps: Decimal,
    #[builder(setter(into))]
    pub asset_id: String,
    #[builder(setter(into))]
    pub outcome: String,
    pub side: Side,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct UserEarningResponse {
    pub date: NaiveDate,
    #[builder(setter(into))]
    pub condition_id: String,
    pub asset_address: Address,
    pub maker_address: Address,
    pub earnings: Decimal,
    pub asset_rate: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct TotalUserEarningResponse {
    pub date: NaiveDate,
    pub asset_address: Address,
    pub maker_address: Address,
    pub earnings: Decimal,
    pub asset_rate: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
pub struct UserRewardsEarningRequest {
    pub date: NaiveDate,
    #[builder(default)]
    pub order_by: String,
    #[builder(default)]
    pub position: String,
    #[builder(default)]
    pub no_competition: bool,
}

impl UserRewardsEarningRequest {
    pub(crate) fn as_params(&self, next_cursor: Option<&String>) -> String {
        let order_by = format!("order_by={}", self.order_by);
        let position = format!("position={}", self.position);
        let no_competition = format!("no_competition={}", self.no_competition);

        let params = format!("date={}&{order_by}&{position}&{no_competition}", self.date);

        format_params_with_cursor(params.as_str(), next_cursor)
    }
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct UserRewardsEarningResponse {
    #[builder(setter(into))]
    pub condition_id: String,
    #[builder(setter(into))]
    pub question: String,
    #[builder(setter(into))]
    pub market_slug: String,
    #[builder(setter(into))]
    pub event_slug: String,
    #[builder(setter(into))]
    pub image: String,
    pub rewards_max_spread: Decimal,
    pub rewards_min_size: Decimal,
    pub market_competitiveness: Decimal,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub tokens: Vec<Token>,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub rewards_config: Vec<RewardsConfig>,
    pub maker_address: Address,
    pub earning_percentage: Decimal,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub earnings: Vec<Earning>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct RewardsConfig {
    pub asset_address: Address,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub rate_per_day: Decimal,
    pub total_rewards: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct MarketRewardsConfig {
    #[builder(setter(into))]
    // We sometimes get numbers or strings back
    #[serde(deserialize_with = "string_from_number_or_string")]
    pub id: String,
    pub asset_address: Address,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub rate_per_day: Decimal,
    pub total_rewards: Decimal,
    pub total_days: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct Earning {
    pub asset_address: Address,
    pub earnings: Decimal,
    pub asset_rate: Decimal,
}

pub type RewardsPercentagesResponse = HashMap<String, Decimal>;

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct CurrentRewardResponse {
    #[builder(setter(into))]
    pub condition_id: String,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub rewards_config: Vec<RewardsConfig>,
    pub rewards_max_spread: Decimal,
    pub rewards_min_size: Decimal,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct MarketRewardResponse {
    #[builder(setter(into))]
    pub condition_id: String,
    #[builder(setter(into))]
    pub question: String,
    #[builder(setter(into))]
    pub market_slug: String,
    #[builder(setter(into))]
    pub event_slug: String,
    #[builder(setter(into))]
    pub image: String,
    pub rewards_max_spread: Decimal,
    pub rewards_min_size: Decimal,
    pub market_competitiveness: Decimal,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub tokens: Vec<Token>,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub rewards_config: Vec<MarketRewardsConfig>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct BuilderApiKeyResponse {
    pub key: ApiKey,
    #[builder(default)]
    #[builder(setter(strip_option))]
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[builder(default)]
    #[builder(setter(strip_option))]
    #[serde(default)]
    pub revoked_at: Option<DateTime<Utc>>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(pattern = "owned", build_fn(error = "Error"))]
pub struct BuilderTradeResponse {
    #[builder(setter(into))]
    pub id: String,
    #[builder(setter(into))]
    pub trade_type: String,
    #[builder(setter(into))]
    pub taker_order_hash: String,
    #[builder(setter(into))]
    pub builder: String,
    #[builder(setter(into))]
    pub market: String,
    #[builder(setter(into))]
    pub asset_id: String,
    pub side: Side,
    pub size: Decimal,
    pub size_usdc: Decimal,
    pub price: Decimal,
    pub status: OrderStatusType,
    #[builder(setter(into))]
    pub outcome: String,
    pub outcome_index: u32,
    pub owner: ApiKey,
    #[builder(setter(into))]
    pub maker: String,
    #[builder(setter(into))]
    pub transaction_hash: String,
    #[serde_as(as = "TimestampSeconds<String>")]
    pub match_time: DateTime<Utc>,
    pub bucket_index: u32,
    pub fee: Decimal,
    pub fee_usdc: Decimal,
    #[builder(setter(into, strip_option))]
    #[builder(default)]
    #[serde(alias = "err_msg")]
    pub err_msg: Option<String>,
    #[builder(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[builder(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Generic wrapper structure that holds inner `data` with metadata designating how to query for the
/// next page.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, Builder, PartialEq)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(strip_option))]
pub struct Page<T> {
    pub data: Vec<T>,
    /// The continuation token to supply to the API to trigger for the next [`Page<T>`].
    #[builder(setter(into))]
    pub next_cursor: String,
    /// The maximum length of `data`.
    pub limit: u64,
    /// The length of `data`
    pub count: u64,
}

fn format_params_with_cursor(params: &str, next_cursor: Option<&String>) -> String {
    match (params, next_cursor) {
        ("", Some(cursor)) => format!("?next_cursor={cursor}"),
        ("", None) => String::new(),
        (params, Some(cursor)) => format!("?{params}&next_cursor={cursor}"),
        (params, None) => format!("?{params}"),
    }
}

fn string_from_number_or_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNumberVisitor;

    impl Visitor<'_> for StringOrNumberVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or integer")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v.to_owned())
        }

        fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }

        fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v.to_string())
        }

        fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v.to_string())
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Validation;

    #[test]
    fn tick_size_decimals_should_succeed() {
        assert_eq!(TickSize::Tenth.as_decimal().scale(), 1);
        assert_eq!(TickSize::Hundredth.as_decimal().scale(), 2);
        assert_eq!(TickSize::Thousandth.as_decimal().scale(), 3);
        assert_eq!(TickSize::TenThousandth.as_decimal().scale(), 4);
    }

    #[test]
    fn trades_request_as_params_should_succeed() -> Result<()> {
        let request = TradesRequestBuilder::default()
            .market("10000")
            .asset_id("100")
            .id("aa-bb")
            .maker_address(Address::ZERO)
            .build()?;

        assert_eq!(
            request.as_params(None),
            "?id=aa-bb&maker_address=0x0000000000000000000000000000000000000000&market=10000&asset_id=100"
        );
        assert_eq!(
            request.as_params(Some(&"1".to_owned())),
            "?id=aa-bb&maker_address=0x0000000000000000000000000000000000000000&market=10000&asset_id=100&next_cursor=1"
        );

        Ok(())
    }

    #[test]
    fn orders_request_as_params_should_succeed() -> Result<()> {
        let request = OrdersRequestBuilder::default()
            .market("10000")
            .asset_id("100")
            .order_id("aa-bb")
            .build()?;

        assert_eq!(
            request.as_params(None),
            "?order_id=aa-bb&market=10000&asset_id=100"
        );
        assert_eq!(
            request.as_params(Some(&"1".to_owned())),
            "?order_id=aa-bb&market=10000&asset_id=100&next_cursor=1"
        );

        Ok(())
    }

    #[test]
    fn delete_notifications_request_as_params_should_succeed() -> Result<()> {
        let empty_request = DeleteNotificationsRequestBuilder::default().build()?;
        let request = DeleteNotificationsRequestBuilder::default()
            .notification_ids(vec!["1".to_owned(), "2".to_owned()])
            .build()?;

        assert_eq!(empty_request.as_params(), "");
        assert_eq!(request.as_params(), "?ids=1,2");

        Ok(())
    }

    #[test]
    fn balance_allowance_request_as_params_should_succeed() -> Result<()> {
        let request = BalanceAllowanceRequestBuilder::default()
            .asset_type(AssetType::Collateral)
            .token_id("1".to_owned())
            .build()?;

        assert_eq!(
            request.as_params(SignatureType::Eoa),
            "?asset_type=COLLATERAL&token_id=1&signature_type=0"
        );

        Ok(())
    }

    #[test]
    fn user_rewards_earning_request_as_params_should_succeed() -> Result<()> {
        let request = UserRewardsEarningRequestBuilder::default()
            .date(NaiveDate::MIN)
            .build()?;

        assert_eq!(
            request.as_params(Some(&"1".to_owned())),
            "?date=-262143-01-01&order_by=&position=&no_competition=false&next_cursor=1"
        );

        Ok(())
    }

    #[test]
    fn tick_size_should_display() {
        assert_eq!(format!("{}", TickSize::Tenth), "Tenth(0.1)");
        assert_eq!(format!("{}", TickSize::Hundredth), "Hundredth(0.01)");
        assert_eq!(format!("{}", TickSize::Thousandth), "Thousandth(0.001)");
        assert_eq!(
            format!("{}", TickSize::TenThousandth),
            "TenThousandth(0.0001)"
        );
    }

    #[test]
    fn tick_from_decimal_should_succeed() {
        assert_eq!(TickSize::from(dec!(0.0001)), TickSize::TenThousandth);
        assert_eq!(TickSize::from(dec!(0.001)), TickSize::Thousandth);
        assert_eq!(TickSize::from(dec!(0.01)), TickSize::Hundredth);
        assert_eq!(TickSize::from(dec!(0.1)), TickSize::Tenth);
    }

    #[test]
    #[should_panic = "Unable to convert 1 to TickSize"]
    fn non_standard_decimal_to_tick_size_should_fail() {
        _ = TickSize::from(Decimal::ONE);
    }

    #[test]
    fn amount_should_succeed() -> Result<()> {
        let usdc = Amount::usdc(Decimal::ONE_HUNDRED)?;
        assert!(usdc.is_usdc());
        assert_eq!(usdc.as_inner(), Decimal::ONE_HUNDRED);

        let shares = Amount::shares(Decimal::ONE_HUNDRED)?;
        assert!(shares.is_shares());
        assert_eq!(shares.as_inner(), Decimal::ONE_HUNDRED);

        Ok(())
    }

    #[test]
    fn improper_shares_lot_size_should_fail() {
        let Err(err) = Amount::shares(dec!(0.23400)) else {
            panic!()
        };

        let message = err.downcast_ref::<Validation>().unwrap();
        assert_eq!(
            message.reason,
            format!("Unable to build Amount with 3 decimal points, must be <= {LOT_SIZE_SCALE}")
        );
    }

    #[test]
    fn improper_usdc_decimal_size_should_fail() {
        let Err(err) = Amount::usdc(dec!(0.2340011)) else {
            panic!()
        };

        let message = err.downcast_ref::<Validation>().unwrap();
        assert_eq!(
            message.reason,
            format!("Unable to build Amount with 7 decimal points, must be <= {USDC_DECIMALS}")
        );
    }

    #[test]
    fn side_to_string_should_succeed() {
        assert_eq!(Side::Buy.to_string(), "BUY");
        assert_eq!(Side::Sell.to_string(), "SELL");
    }
}
