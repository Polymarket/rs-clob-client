#![allow(
    clippy::module_name_repetitions,
    reason = "Response suffix is intentional for clarity"
)]

use std::collections::HashMap;

use bon::Builder;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{
    DefaultOnError, DefaultOnNull, NoneAsEmptyString, TimestampMilliSeconds, TimestampSeconds,
    TryFromInto, serde_as,
};
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

use crate::Result;
use crate::auth::ApiKey;
use crate::clob::types::{OrderStatusType, OrderType, Side, TickSize, TraderSide};
use crate::serde_helpers::StringFromAny;
use crate::types::{Address, B256, Decimal, U256};

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct MidpointResponse {
    pub mid: Decimal,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Builder, PartialEq)]
#[serde(transparent)]
pub struct MidpointsResponse {
    pub midpoints: HashMap<U256, Decimal>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct PriceResponse {
    pub price: Decimal,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Builder, PartialEq)]
#[serde(transparent)]
pub struct PricesResponse {
    pub prices: Option<HashMap<U256, HashMap<Side, Decimal>>>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct SpreadResponse {
    pub spread: Decimal,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct SpreadsResponse {
    pub spreads: Option<HashMap<U256, Decimal>>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct PriceHistoryResponse {
    pub history: Vec<PricePoint>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct PricePoint {
    pub t: i64,
    pub p: Decimal,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
#[builder(on(TickSize, into))]
pub struct TickSizeResponse {
    pub minimum_tick_size: TickSize,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct NegRiskResponse {
    pub neg_risk: bool,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct FeeRateResponse {
    pub base_fee: u32,
}

/// Response from the Polymarket geoblock endpoint.
///
/// This indicates whether the requesting IP address is blocked from placing orders
/// due to geographic restrictions.
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct GeoblockResponse {
    /// Whether the user is blocked from placing orders
    pub blocked: bool,
    /// The detected IP address
    pub ip: String,
    /// ISO 3166-1 alpha-2 country code
    pub country: String,
    /// Region/state code
    pub region: String,
}

#[non_exhaustive]
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct OrderBookSummaryResponse {
    /// The market condition ID.
    pub market: B256,
    pub asset_id: U256,
    #[serde_as(as = "TimestampMilliSeconds<String>")]
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
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
    #[serde_as(as = "TryFromInto<Decimal>")]
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
pub struct OrderSummary {
    pub price: Decimal,
    pub size: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq)]
pub struct LastTradePriceResponse {
    pub price: Decimal,
    pub side: Side,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct LastTradesPricesResponse {
    pub token_id: U256,
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
#[builder(on(String, into))]
pub struct MarketResponse {
    pub enable_order_book: bool,
    pub active: bool,
    pub closed: bool,
    pub archived: bool,
    pub accepting_orders: bool,
    pub accepting_order_timestamp: Option<DateTime<Utc>>,
    pub minimum_order_size: Decimal,
    pub minimum_tick_size: Decimal,
    /// The market condition ID (unique market identifier).
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub condition_id: Option<B256>,
    /// The CTF question ID.
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub question_id: Option<B256>,
    pub question: String,
    pub description: String,
    pub market_slug: String,
    pub end_date_iso: Option<DateTime<Utc>>,
    pub game_start_time: Option<DateTime<Utc>>,
    pub seconds_delay: u64,
    /// The FPMM (Fixed Product Market Maker) contract address.
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub fpmm: Option<Address>,
    pub maker_base_fee: Decimal,
    pub taker_base_fee: Decimal,
    pub notifications_enabled: bool,
    pub neg_risk: bool,
    /// The negative risk market ID (empty string if not a neg risk market).
    #[serde_as(as = "DefaultOnError<NoneAsEmptyString>")]
    #[serde(default)]
    pub neg_risk_market_id: Option<B256>,
    /// The negative risk request ID (empty string if not a neg risk market).
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub neg_risk_request_id: Option<B256>,
    pub icon: String,
    pub image: String,
    pub rewards: Rewards,
    pub is_50_50_outcome: bool,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub tokens: Vec<Token>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub tags: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct Token {
    pub token_id: U256,
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
#[builder(on(String, into))]
pub struct SimplifiedMarketResponse {
    /// The market condition ID (unique market identifier).
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub condition_id: Option<B256>,
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
pub struct ApiKeysResponse {
    #[serde(rename = "apiKeys")]
    keys: Option<Vec<ApiKey>>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
pub struct BanStatusResponse {
    pub closed_only: bool,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct PostOrderResponse {
    pub error_msg: Option<String>,
    #[serde(deserialize_with = "empty_string_as_zero")]
    pub making_amount: Decimal,
    #[serde(deserialize_with = "empty_string_as_zero")]
    pub taking_amount: Decimal,
    #[serde(rename = "orderID")]
    pub order_id: String,
    pub status: OrderStatusType,
    pub success: bool,
    /// On-chain transaction hashes for the order execution.
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(alias = "transactionsHashes")]
    pub transaction_hashes: Vec<B256>,
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
#[builder(on(String, into))]
pub struct OpenOrderResponse {
    pub id: String,
    pub status: OrderStatusType,
    pub owner: ApiKey,
    pub maker_address: Address,
    /// The market condition ID.
    pub market: B256,
    pub asset_id: U256,
    pub side: Side,
    pub original_size: Decimal,
    pub size_matched: Decimal,
    pub price: Decimal,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub associate_trades: Vec<String>,
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
pub struct CancelOrdersResponse {
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub canceled: Vec<String>,
    #[builder(default)]
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(alias = "not_canceled")]
    pub not_canceled: HashMap<String, String>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct TradeResponse {
    pub id: String,
    pub taker_order_id: String,
    /// The market condition ID.
    pub market: B256,
    pub asset_id: U256,
    pub side: Side,
    pub size: Decimal,
    pub fee_rate_bps: Decimal,
    pub price: Decimal,
    pub status: OrderStatusType,
    #[serde_as(as = "TimestampSeconds<String>")]
    pub match_time: DateTime<Utc>,
    #[serde_as(as = "TimestampSeconds<String>")]
    pub last_update: DateTime<Utc>,
    pub outcome: String,
    pub bucket_index: u32,
    pub owner: ApiKey,
    pub maker_address: Address,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub maker_orders: Vec<MakerOrder>,
    /// On-chain transaction hash.
    pub transaction_hash: B256,
    pub trader_side: TraderSide,
    #[serde(default)]
    pub error_msg: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
pub struct NotificationResponse {
    pub r#type: u32,
    pub owner: ApiKey,
    pub payload: NotificationPayload,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct NotificationPayload {
    pub asset_id: U256,
    /// The market condition ID (unique market identifier).
    pub condition_id: B256,
    #[serde(rename = "eventSlug")]
    pub event_slug: String,
    pub icon: String,
    pub image: String,
    /// The market condition ID (same as `condition_id`).
    pub market: B256,
    pub market_slug: String,
    pub matched_size: Decimal,
    pub order_id: String,
    pub original_size: Decimal,
    pub outcome: String,
    pub outcome_index: u64,
    pub owner: ApiKey,
    pub price: Decimal,
    pub question: String,
    pub remaining_size: Decimal,
    #[serde(rename = "seriesSlug")]
    pub series_slug: String,
    pub side: Side,
    pub trade_id: String,
    /// On-chain transaction hash.
    pub transaction_hash: B256,
    #[serde(alias = "type")]
    pub order_type: OrderType,
}

#[non_exhaustive]
#[allow(
    clippy::allow_attributes,
    reason = "Bon will generate code that has an allow attribute for some reason on the `allowances` field"
)]
#[allow(
    clippy::allow_attributes_without_reason,
    reason = "Bon will generate code that has an allow attribute for some reason on the `allowances` field"
)]
#[derive(Debug, Default, Clone, Deserialize, Builder, PartialEq)]
pub struct BalanceAllowanceResponse {
    pub balance: Decimal,
    #[serde(default)]
    #[builder(default)]
    pub allowances: HashMap<Address, String>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
pub struct OrderScoringResponse {
    pub scoring: bool,
}

pub type OrdersScoringResponse = HashMap<String, bool>;

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Builder, PartialEq)]
pub struct PriceSideResponse {
    pub side: Side,
    pub price: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize, Clone, Builder, PartialEq)]
pub struct RewardRate {
    pub asset_address: Address,
    pub rewards_daily_rate: Decimal,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize, Builder, PartialEq)]
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
#[builder(on(String, into))]
pub struct UserInfo {
    pub address: Address,
    pub username: String,
    pub profile_picture: String,
    pub optimized_profile_picture: String,
    pub pseudonym: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct MakerOrder {
    pub order_id: String,
    pub owner: ApiKey,
    pub maker_address: Address,
    pub matched_amount: Decimal,
    pub price: Decimal,
    pub fee_rate_bps: Decimal,
    pub asset_id: U256,
    pub outcome: String,
    pub side: Side,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct UserEarningResponse {
    pub date: NaiveDate,
    /// The market condition ID (unique market identifier).
    pub condition_id: B256,
    pub asset_address: Address,
    pub maker_address: Address,
    pub earnings: Decimal,
    pub asset_rate: Decimal,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct TotalUserEarningResponse {
    pub date: NaiveDate,
    pub asset_address: Address,
    pub maker_address: Address,
    pub earnings: Decimal,
    pub asset_rate: Decimal,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct UserRewardsEarningResponse {
    /// The market condition ID (unique market identifier).
    pub condition_id: B256,
    pub question: String,
    pub market_slug: String,
    pub event_slug: String,
    pub image: String,
    pub rewards_max_spread: Decimal,
    pub rewards_min_size: Decimal,
    pub market_competitiveness: Decimal,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub tokens: Vec<Token>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub rewards_config: Vec<RewardsConfig>,
    pub maker_address: Address,
    pub earning_percentage: Decimal,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub earnings: Vec<Earning>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
pub struct RewardsConfig {
    pub asset_address: Address,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub rate_per_day: Decimal,
    pub total_rewards: Decimal,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct MarketRewardsConfig {
    #[serde_as(as = "StringFromAny")]
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
pub struct Earning {
    pub asset_address: Address,
    pub earnings: Decimal,
    pub asset_rate: Decimal,
}

pub type RewardsPercentagesResponse = HashMap<String, Decimal>;

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct CurrentRewardResponse {
    /// The market condition ID (unique market identifier).
    pub condition_id: B256,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub rewards_config: Vec<RewardsConfig>,
    pub rewards_max_spread: Decimal,
    pub rewards_min_size: Decimal,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct MarketRewardResponse {
    /// The market condition ID (unique market identifier).
    pub condition_id: B256,
    pub question: String,
    pub market_slug: String,
    pub event_slug: String,
    pub image: String,
    pub rewards_max_spread: Decimal,
    pub rewards_min_size: Decimal,
    pub market_competitiveness: Decimal,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub tokens: Vec<Token>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub rewards_config: Vec<MarketRewardsConfig>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BuilderApiKeyResponse {
    pub key: ApiKey,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub revoked_at: Option<DateTime<Utc>>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct BuilderTradeResponse {
    pub id: String,
    pub trade_type: String,
    /// Hash of the taker order.
    pub taker_order_hash: B256,
    /// Address of the builder.
    pub builder: Address,
    /// The market condition ID.
    pub market: B256,
    pub asset_id: U256,
    pub side: Side,
    pub size: Decimal,
    pub size_usdc: Decimal,
    pub price: Decimal,
    pub status: OrderStatusType,
    pub outcome: String,
    pub outcome_index: u32,
    pub owner: ApiKey,
    /// Address of the maker.
    pub maker: Address,
    /// On-chain transaction hash.
    pub transaction_hash: B256,
    #[serde_as(as = "TimestampSeconds<String>")]
    pub match_time: DateTime<Utc>,
    pub bucket_index: u32,
    pub fee: Decimal,
    pub fee_usdc: Decimal,
    #[serde(alias = "err_msg")]
    pub err_msg: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct HeartbeatResponse {
    pub heartbeat_id: Uuid,
    pub error: Option<String>,
}

/// Generic wrapper structure that holds inner `data` with metadata designating how to query for the
/// next page.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct Page<T> {
    pub data: Vec<T>,
    /// The continuation token to supply to the API to trigger for the next [`Page<T>`].
    pub next_cursor: String,
    /// The maximum length of `data`.
    pub limit: u64,
    /// The length of `data`
    pub count: u64,
}

/// Response from creating an RFQ request.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CreateRfqRequestResponse {
    /// Unique identifier for the created request.
    pub request_id: String,
    /// Unix timestamp when the request expires.
    pub expiry: i64,
}

/// Response from creating an RFQ quote.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CreateRfqQuoteResponse {
    /// Unique identifier for the created quote.
    pub quote_id: String,
}

/// Response from accepting an RFQ quote.
///
/// Returns "OK" as text, represented as unit type for deserialization.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptRfqQuoteResponse;

/// Response from approving an RFQ order.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct ApproveRfqOrderResponse {
    /// Trade IDs for the executed order.
    pub trade_ids: Vec<String>,
}

/// An RFQ request in the system.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct RfqRequest {
    /// Unique request identifier.
    pub request_id: String,
    /// User's address.
    pub user: Address,
    /// Proxy address (may be same as user).
    pub proxy: Address,
    /// Market condition ID.
    pub market: B256,
    /// Token ID for the outcome token.
    pub token: String,
    /// Complement token ID.
    pub complement: String,
    /// Order side (BUY or SELL).
    pub side: Side,
    /// Size of tokens to receive.
    pub size_in: Decimal,
    /// Size of tokens to give.
    pub size_out: Decimal,
    /// Price for the request.
    pub price: Decimal,
    /// Unix timestamp when the request expires.
    pub expiry: i64,
}

/// An RFQ quote in the system.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct RfqQuote {
    /// Unique quote identifier.
    pub quote_id: String,
    /// Request ID this quote is for.
    pub request_id: String,
    /// Quoter's address.
    pub user: Address,
    /// Proxy address (may be same as user).
    pub proxy: Address,
    /// Market condition ID.
    pub market: B256,
    /// Token ID for the outcome token.
    pub token: String,
    /// Complement token ID.
    pub complement: String,
    /// Order side (BUY or SELL).
    pub side: Side,
    /// Size of tokens to receive.
    pub size_in: Decimal,
    /// Size of tokens to give.
    pub size_out: Decimal,
    /// Quoted price.
    pub price: Decimal,
}
