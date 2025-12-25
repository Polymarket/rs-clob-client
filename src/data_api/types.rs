//! Types for the Polymarket Data API.
//!
//! This module contains all types used by the Data API client, organized into:
//!
//! - **Common types**: Fundamental types like [`Hash64`],
//!   as well as enums for sorting, filtering, and pagination.
//!
//! - **Request types**: Builder-pattern structs for each API endpoint
//!   (e.g., [`PositionsRequest`], [`TradesRequest`]).
//!
//! - **Response types**: Structs representing API responses
//!   (e.g., [`Position`], [`Trade`], [`Activity`]).
//!
//! # Request Building
//!
//! All request types use the builder pattern via the [`bon`](https://docs.rs/bon) crate:
//!
//! ```
//! use polymarket_client_sdk::data_api::types::{PositionsRequest, PositionSortBy, SortDirection};
//!
//! let request = PositionsRequest::builder()
//!     .user("0x56687bf447db6ffa42ffe2204a05edaa20f55839")
//!     .sort_by(PositionSortBy::CashPnl)
//!     .sort_direction(SortDirection::Desc)
//!     .build();
//! ```
//!
//! # Type Safety
//!
//! The types in this module provide compile-time validation where possible:
//!
//! - Bounded integer types (e.g., [`PositionsLimit`]) enforce API limits

use std::error::Error as StdError;
use std::fmt;

use bon::Builder;
use serde::{Deserialize, Serialize};

use super::ser::{comma_separated, comma_separated_vec, is_empty_vec, vec_is_empty};

// =============================================================================
// Common Types
// =============================================================================

/// Type alias for Ethereum addresses (0x-prefixed hex strings).
pub type Address = String;

/// Type alias for 64-character hex hashes (condition IDs, market identifiers).
pub type Hash64 = String;

/// Type alias for market title filter strings.
pub type Title = String;

/// The side of a trade (buy or sell).
///
/// Used to indicate whether a trade was a purchase or sale of outcome tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum Side {
    /// Buying outcome tokens (going long on an outcome).
    Buy,
    /// Selling outcome tokens (going short or closing a long position).
    Sell,
}

/// The type of on-chain activity for a user.
///
/// Activities represent various operations that users can perform on the Polymarket protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum ActivityType {
    /// A trade (buy or sell) of outcome tokens.
    Trade,
    /// Splitting collateral into outcome token sets.
    Split,
    /// Merging outcome token sets back into collateral.
    Merge,
    /// Redeeming winning outcome tokens for collateral after market resolution.
    Redeem,
    /// Receiving a reward (e.g., liquidity mining rewards).
    Reward,
    /// Converting between token types.
    Conversion,
}

/// Sort criteria for position queries.
///
/// Determines how positions are ordered in the response. Default is [`Tokens`](Self::Tokens).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[non_exhaustive]
pub enum PositionSortBy {
    /// Sort by current value of the position.
    #[serde(rename = "CURRENT")]
    #[strum(serialize = "CURRENT")]
    Current,
    /// Sort by initial value (cost basis) of the position.
    #[serde(rename = "INITIAL")]
    #[strum(serialize = "INITIAL")]
    Initial,
    /// Sort by number of tokens held (default).
    #[serde(rename = "TOKENS")]
    #[strum(serialize = "TOKENS")]
    Tokens,
    /// Sort by cash profit and loss.
    #[serde(rename = "CASHPNL")]
    #[strum(serialize = "CASHPNL")]
    CashPnl,
    /// Sort by percentage profit and loss.
    #[serde(rename = "PERCENTPNL")]
    #[strum(serialize = "PERCENTPNL")]
    PercentPnl,
    /// Sort alphabetically by market title.
    #[serde(rename = "TITLE")]
    #[strum(serialize = "TITLE")]
    Title,
    /// Sort by markets that are resolving soon.
    #[serde(rename = "RESOLVING")]
    #[strum(serialize = "RESOLVING")]
    Resolving,
    /// Sort by current market price.
    #[serde(rename = "PRICE")]
    #[strum(serialize = "PRICE")]
    Price,
    /// Sort by average entry price.
    #[serde(rename = "AVGPRICE")]
    #[strum(serialize = "AVGPRICE")]
    AvgPrice,
}

/// Sort criteria for closed position queries.
///
/// Determines how closed positions are ordered in the response. Default is [`RealizedPnl`](Self::RealizedPnl).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[non_exhaustive]
pub enum ClosedPositionSortBy {
    /// Sort by realized profit and loss (default).
    #[serde(rename = "REALIZEDPNL")]
    #[strum(serialize = "REALIZEDPNL")]
    RealizedPnl,
    /// Sort alphabetically by market title.
    #[serde(rename = "TITLE")]
    #[strum(serialize = "TITLE")]
    Title,
    /// Sort by final market price.
    #[serde(rename = "PRICE")]
    #[strum(serialize = "PRICE")]
    Price,
    /// Sort by average entry price.
    #[serde(rename = "AVGPRICE")]
    #[strum(serialize = "AVGPRICE")]
    AvgPrice,
    /// Sort by timestamp when the position was closed.
    #[serde(rename = "TIMESTAMP")]
    #[strum(serialize = "TIMESTAMP")]
    Timestamp,
}

/// Sort criteria for activity queries.
///
/// Determines how activity records are ordered in the response. Default is [`Timestamp`](Self::Timestamp).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum ActivitySortBy {
    /// Sort by activity timestamp (default).
    Timestamp,
    /// Sort by number of tokens involved in the activity.
    Tokens,
    /// Sort by cash (USDC) value of the activity.
    Cash,
}

/// Sort direction for query results.
///
/// Default is [`Desc`](Self::Desc) (descending) for most endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum SortDirection {
    /// Ascending order (smallest/earliest first).
    Asc,
    /// Descending order (largest/latest first, default).
    Desc,
}

/// Filter type for trade queries.
///
/// Used with `filterAmount` to filter trades by minimum value.
/// Both `filterType` and `filterAmount` must be provided together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum FilterType {
    /// Filter by USDC cash value.
    Cash,
    /// Filter by number of tokens.
    Tokens,
}

/// Time period for aggregating leaderboard and volume data.
///
/// Default is [`Day`](Self::Day) for most endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum TimePeriod {
    /// Last 24 hours (default).
    Day,
    /// Last 7 days.
    Week,
    /// Last 30 days.
    Month,
    /// All time.
    All,
}

/// Market category for filtering trader leaderboard results.
///
/// Default is [`Overall`](Self::Overall) which includes all categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum LeaderboardCategory {
    /// All categories combined (default).
    Overall,
    /// Politics and elections markets.
    Politics,
    /// Sports betting markets.
    Sports,
    /// Cryptocurrency markets.
    Crypto,
    /// Pop culture and entertainment markets.
    Culture,
    /// Social media mentions markets.
    Mentions,
    /// Weather prediction markets.
    Weather,
    /// Economic indicator markets.
    Economics,
    /// Technology markets.
    Tech,
    /// Financial markets.
    Finance,
}

/// Ordering criteria for trader leaderboard results.
///
/// Default is [`Pnl`](Self::Pnl) (profit and loss).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum LeaderboardOrderBy {
    /// Order by profit and loss (default).
    Pnl,
    /// Order by trading volume.
    Vol,
}

/// A filter for querying by markets or events.
///
/// The API allows filtering by either condition IDs (markets) or event IDs,
/// but not both simultaneously. This enum enforces that mutual exclusivity.
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::MarketFilter;
///
/// // Filter by specific markets (condition IDs)
/// let by_markets = MarketFilter::markets(["0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917".to_string()]);
///
/// // Or filter by events (which may contain multiple markets)
/// let by_events = MarketFilter::event_ids([123]);
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MarketFilter {
    /// Filter by condition IDs (market identifiers).
    Markets(Vec<String>),
    /// Filter by event IDs (groups of related markets).
    EventIds(Vec<u64>),
}

impl MarketFilter {
    /// Creates a filter for specific markets by their condition IDs.
    #[must_use]
    pub fn markets<I: IntoIterator<Item = String>>(ids: I) -> Self {
        Self::Markets(ids.into_iter().collect())
    }

    /// Creates a filter for all markets within the specified events.
    #[must_use]
    pub fn event_ids<I: IntoIterator<Item = u64>>(ids: I) -> Self {
        Self::EventIds(ids.into_iter().collect())
    }
}

impl Serialize for MarketFilter {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Self::Markets(ids) if !ids.is_empty() => {
                let s = ids.iter().map(String::as_str).collect::<Vec<_>>().join(",");
                map.serialize_entry("market", &s)?;
            }
            Self::EventIds(ids) if !ids.is_empty() => {
                let s = ids
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                map.serialize_entry("eventId", &s)?;
            }
            _ => {}
        }
        map.end()
    }
}

/// Error type for bounded integer values that are out of range.
#[derive(Debug)]
pub struct BoundedIntError {
    value: u32,
    min: u32,
    max: u32,
    type_name: &'static str,
}

impl fmt::Display for BoundedIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} must be between {} and {} (got {})",
            self.type_name, self.min, self.max, self.value
        )
    }
}

impl StdError for BoundedIntError {}

macro_rules! bounded_u32 {
    ($name:ident, min = $min:expr, max = $max:expr, default = $default:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "u32", into = "u32")]
        pub struct $name(u32);

        impl $name {
            pub const MIN: u32 = $min;
            pub const MAX: u32 = $max;
            pub const DEFAULT: u32 = $default;

            pub fn new(value: u32) -> Result<Self, BoundedIntError> {
                if (Self::MIN..=Self::MAX).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(BoundedIntError {
                        value,
                        min: Self::MIN,
                        max: Self::MAX,
                        type_name: stringify!($name),
                    })
                }
            }

            #[must_use]
            pub fn value(self) -> u32 {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(Self::DEFAULT)
            }
        }

        impl TryFrom<u32> for $name {
            type Error = BoundedIntError;
            fn try_from(value: u32) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for u32 {
            fn from(b: $name) -> Self {
                b.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

bounded_u32!(PositionsLimit, min = 0, max = 500, default = 100);
bounded_u32!(PositionsOffset, min = 0, max = 10000, default = 0);
bounded_u32!(TradesLimit, min = 0, max = 10000, default = 100);
bounded_u32!(TradesOffset, min = 0, max = 10000, default = 0);
bounded_u32!(ActivityLimit, min = 0, max = 500, default = 100);
bounded_u32!(ActivityOffset, min = 0, max = 10000, default = 0);
bounded_u32!(HoldersLimit, min = 0, max = 20, default = 20);
bounded_u32!(HoldersMinBalance, min = 0, max = 999_999, default = 1);
bounded_u32!(ClosedPositionsLimit, min = 0, max = 50, default = 10);
bounded_u32!(ClosedPositionsOffset, min = 0, max = 100_000, default = 0);
bounded_u32!(BuilderLeaderboardLimit, min = 0, max = 50, default = 25);
bounded_u32!(BuilderLeaderboardOffset, min = 0, max = 1000, default = 0);
bounded_u32!(TraderLeaderboardLimit, min = 1, max = 50, default = 25);
bounded_u32!(TraderLeaderboardOffset, min = 0, max = 1000, default = 0);

/// A filter for minimum trade size.
///
/// Used to filter trades by a minimum value, either in USDC (cash) or tokens.
/// Both `filter_type` and `filter_amount` must be provided together to the API.
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::TradeFilter;
///
/// // Filter trades with at least $100 USDC value
/// let filter = TradeFilter::cash(100.0).unwrap();
///
/// // Filter trades with at least 50 tokens
/// let filter = TradeFilter::tokens(50.0).unwrap();
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TradeFilter {
    /// The type of filter (cash or tokens).
    pub filter_type: FilterType,
    /// The minimum amount to filter by (must be >= 0).
    pub filter_amount: f64,
}

impl TradeFilter {
    /// Creates a new trade filter with the specified type and amount.
    ///
    /// # Errors
    ///
    /// Returns [`TradeFilterError`] if the amount is negative.
    pub fn new(filter_type: FilterType, filter_amount: f64) -> Result<Self, TradeFilterError> {
        if filter_amount < 0.0 {
            return Err(TradeFilterError::NegativeAmount(filter_amount));
        }
        Ok(Self {
            filter_type,
            filter_amount,
        })
    }

    /// Creates a cash (USDC) value filter.
    ///
    /// # Errors
    ///
    /// Returns [`TradeFilterError`] if the amount is negative.
    pub fn cash(amount: f64) -> Result<Self, TradeFilterError> {
        Self::new(FilterType::Cash, amount)
    }

    /// Creates a token quantity filter.
    ///
    /// # Errors
    ///
    /// Returns [`TradeFilterError`] if the amount is negative.
    pub fn tokens(amount: f64) -> Result<Self, TradeFilterError> {
        Self::new(FilterType::Tokens, amount)
    }
}

impl Serialize for TradeFilter {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("filterType", &self.filter_type)?;
        map.serialize_entry("filterAmount", &self.filter_amount)?;
        map.end()
    }
}

/// Error type for invalid trade filter values.
#[derive(Debug)]
#[non_exhaustive]
pub enum TradeFilterError {
    /// The filter amount was negative.
    NegativeAmount(f64),
}

impl fmt::Display for TradeFilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeAmount(amount) => {
                write!(f, "filter amount must be >= 0 (got {amount})")
            }
        }
    }
}

impl StdError for TradeFilterError {}

// =============================================================================
// Request Types
// =============================================================================

/// Trait for converting request types to URL query strings.
///
/// This trait is automatically implemented for all types that implement [`Serialize`].
/// It uses [`serde_urlencoded`] to serialize the struct fields into a query string.
pub trait QueryParams: Serialize {
    /// Converts the request to a URL query string.
    ///
    /// Returns an empty string if no parameters are set, otherwise returns
    /// a string starting with `?` followed by URL-encoded key-value pairs.
    fn query_string(&self) -> String {
        let params = serde_urlencoded::to_string(self).unwrap_or_default();
        if params.is_empty() {
            params
        } else {
            format!("?{params}")
        }
    }
}

impl<T: Serialize> QueryParams for T {}

/// Request parameters for the `/positions` endpoint.
///
/// Fetches current (open) positions for a user. Positions represent holdings
/// of outcome tokens in prediction markets.
///
/// # Required Parameters
///
/// - `user`: The Ethereum address of the user whose positions to retrieve.
///
/// # Optional Parameters
///
/// - `filter`: Filter by specific markets (condition IDs) or events.
///   Cannot specify both markets and events.
/// - `size_threshold`: Minimum position size to include (default: 1).
/// - `redeemable`: If true, only return positions that can be redeemed.
/// - `mergeable`: If true, only return positions that can be merged.
/// - `limit`: Maximum positions to return (0-500, default: 100).
/// - `offset`: Pagination offset (0-10000, default: 0).
/// - `sort_by`: Sort criteria (default: TOKENS).
/// - `sort_direction`: Sort order (default: DESC).
/// - `title`: Filter by market title substring.
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::{PositionsRequest, PositionSortBy, SortDirection};
///
/// let request = PositionsRequest::builder()
///     .user("0x56687bf447db6ffa42ffe2204a05edaa20f55839")
///     .sort_by(PositionSortBy::CashPnl)
///     .sort_direction(SortDirection::Desc)
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct PositionsRequest {
    /// User address (required).
    #[builder(into)]
    pub user: Address,
    /// Filter by markets or events. Mutually exclusive options.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub filter: Option<MarketFilter>,
    /// Minimum position size to include (default: 1).
    #[serde(rename = "sizeThreshold", skip_serializing_if = "Option::is_none")]
    pub size_threshold: Option<f64>,
    /// Only return positions that can be redeemed (default: false).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redeemable: Option<bool>,
    /// Only return positions that can be merged (default: false).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mergeable: Option<bool>,
    /// Maximum number of positions to return (0-500, default: 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PositionsLimit>,
    /// Pagination offset (0-10000, default: 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PositionsOffset>,
    /// Sort criteria (default: TOKENS).
    #[serde(rename = "sortBy", skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<PositionSortBy>,
    /// Sort direction (default: DESC).
    #[serde(rename = "sortDirection", skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<SortDirection>,
    /// Filter by market title substring (max 100 chars).
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
}

/// Request parameters for the `/trades` endpoint.
///
/// Fetches trade history for a user or markets. Trades represent executed
/// orders where outcome tokens were bought or sold.
///
/// # Optional Parameters
///
/// - `user`: Filter by user address.
/// - `filter`: Filter by specific markets (condition IDs) or events.
/// - `limit`: Maximum trades to return (0-10000, default: 100).
/// - `offset`: Pagination offset (0-10000, default: 0).
/// - `taker_only`: If true, only return taker trades (default: true).
/// - `trade_filter`: Filter by minimum trade size (cash or tokens).
/// - `side`: Filter by trade side (BUY or SELL).
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::{TradesRequest, Side, TradeFilter};
///
/// let request = TradesRequest::builder()
///     .user("0x56687bf447db6ffa42ffe2204a05edaa20f55839")
///     .side(Side::Buy)
///     .trade_filter(TradeFilter::cash(100.0).unwrap())
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct TradesRequest {
    /// Filter by user address.
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Address>,
    /// Filter by markets or events. Mutually exclusive options.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub filter: Option<MarketFilter>,
    /// Maximum number of trades to return (0-10000, default: 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<TradesLimit>,
    /// Pagination offset (0-10000, default: 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<TradesOffset>,
    /// Only return taker trades (default: true).
    #[serde(rename = "takerOnly", skip_serializing_if = "Option::is_none")]
    pub taker_only: Option<bool>,
    /// Filter by minimum trade size. Must provide both type and amount.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub trade_filter: Option<TradeFilter>,
    /// Filter by trade side (BUY or SELL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

/// Request parameters for the `/activity` endpoint.
///
/// Fetches on-chain activity for a user, including trades, splits, merges,
/// redemptions, rewards, and conversions.
///
/// # Required Parameters
///
/// - `user`: The Ethereum address of the user whose activity to retrieve.
///
/// # Optional Parameters
///
/// - `filter`: Filter by specific markets (condition IDs) or events.
/// - `activity_types`: Filter by activity types (TRADE, SPLIT, MERGE, etc.).
/// - `limit`: Maximum activities to return (0-500, default: 100).
/// - `offset`: Pagination offset (0-10000, default: 0).
/// - `start`: Start timestamp filter (Unix timestamp).
/// - `end`: End timestamp filter (Unix timestamp).
/// - `sort_by`: Sort criteria (default: TIMESTAMP).
/// - `sort_direction`: Sort order (default: DESC).
/// - `side`: Filter by trade side (only applies to TRADE activities).
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::{ActivityRequest, ActivityType};
///
/// let request = ActivityRequest::builder()
///     .user("0x56687bf447db6ffa42ffe2204a05edaa20f55839")
///     .activity_types(vec![ActivityType::Trade, ActivityType::Redeem])
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct ActivityRequest {
    /// User address (required).
    #[builder(into)]
    pub user: Address,
    /// Filter by markets or events. Mutually exclusive options.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub filter: Option<MarketFilter>,
    /// Filter by activity types.
    #[serde(
        rename = "type",
        serialize_with = "comma_separated",
        skip_serializing_if = "is_empty_vec"
    )]
    pub activity_types: Option<Vec<ActivityType>>,
    /// Maximum number of activities to return (0-500, default: 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<ActivityLimit>,
    /// Pagination offset (0-10000, default: 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<ActivityOffset>,
    /// Start timestamp filter (Unix timestamp, minimum: 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    /// End timestamp filter (Unix timestamp, minimum: 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
    /// Sort criteria (default: TIMESTAMP).
    #[serde(rename = "sortBy", skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ActivitySortBy>,
    /// Sort direction (default: DESC).
    #[serde(rename = "sortDirection", skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<SortDirection>,
    /// Filter by trade side (only applies to TRADE activities).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

/// Request parameters for the `/holders` endpoint.
///
/// Fetches top token holders for specified markets. Returns holders grouped
/// by token (outcome) for each market.
///
/// # Required Parameters
///
/// - `markets`: List of condition IDs (market identifiers) to query.
///
/// # Optional Parameters
///
/// - `limit`: Maximum holders to return per token (0-20, default: 20).
/// - `min_balance`: Minimum balance to include (0-999999, default: 1).
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::HoldersRequest;
///
/// let request = HoldersRequest::builder()
///     .markets(vec!["0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917".to_string()])
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct HoldersRequest {
    /// Condition IDs of markets to query (required).
    #[serde(
        rename = "market",
        serialize_with = "comma_separated_vec",
        skip_serializing_if = "vec_is_empty"
    )]
    pub markets: Vec<Hash64>,
    /// Maximum holders to return per token (0-20, default: 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<HoldersLimit>,
    /// Minimum balance to include (0-999999, default: 1).
    #[serde(rename = "minBalance", skip_serializing_if = "Option::is_none")]
    pub min_balance: Option<HoldersMinBalance>,
}

/// Request parameters for the `/traded` endpoint.
///
/// Fetches the total count of unique markets a user has traded.
///
/// # Required Parameters
///
/// - `user`: The Ethereum address of the user to query.
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct TradedRequest {
    /// User address (required).
    #[builder(into)]
    pub user: Address,
}

/// Request parameters for the `/value` endpoint.
///
/// Fetches the total value of a user's positions, optionally filtered by markets.
///
/// # Required Parameters
///
/// - `user`: The Ethereum address of the user to query.
///
/// # Optional Parameters
///
/// - `markets`: Filter by specific condition IDs.
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct ValueRequest {
    /// User address (required).
    #[builder(into)]
    pub user: Address,
    /// Optional list of condition IDs to filter by.
    #[serde(
        rename = "market",
        serialize_with = "comma_separated",
        skip_serializing_if = "is_empty_vec"
    )]
    pub markets: Option<Vec<Hash64>>,
}

/// Request parameters for the `/oi` (open interest) endpoint.
///
/// Fetches open interest for markets. Open interest represents the total
/// value of outstanding positions in a market.
///
/// # Optional Parameters
///
/// - `markets`: Filter by specific condition IDs. If not provided, returns
///   open interest for all markets.
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct OpenInterestRequest {
    /// Optional list of condition IDs to filter by.
    #[serde(
        rename = "market",
        serialize_with = "comma_separated",
        skip_serializing_if = "is_empty_vec"
    )]
    pub markets: Option<Vec<Hash64>>,
}

/// Request parameters for the `/live-volume` endpoint.
///
/// Fetches live trading volume for an event, including total volume
/// and per-market breakdown.
///
/// # Required Parameters
///
/// - `id`: The event ID to query.
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct LiveVolumeRequest {
    /// Event ID (required).
    pub id: u64,
}

/// Request parameters for the `/closed-positions` endpoint.
///
/// Fetches closed (historical) positions for a user. These are positions
/// that have been fully sold or redeemed.
///
/// # Required Parameters
///
/// - `user`: The Ethereum address of the user to query.
///
/// # Optional Parameters
///
/// - `filter`: Filter by specific markets (condition IDs) or events.
/// - `title`: Filter by market title substring.
/// - `limit`: Maximum positions to return (0-50, default: 10).
/// - `offset`: Pagination offset (0-100000, default: 0).
/// - `sort_by`: Sort criteria (default: REALIZEDPNL).
/// - `sort_direction`: Sort order (default: DESC).
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::{ClosedPositionsRequest, ClosedPositionSortBy};
///
/// let request = ClosedPositionsRequest::builder()
///     .user("0x56687bf447db6ffa42ffe2204a05edaa20f55839")
///     .sort_by(ClosedPositionSortBy::Timestamp)
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct ClosedPositionsRequest {
    /// User address (required).
    #[builder(into)]
    pub user: Address,
    /// Filter by markets or events. Mutually exclusive options.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub filter: Option<MarketFilter>,
    /// Filter by market title substring (max 100 chars).
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Maximum number of positions to return (0-50, default: 10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<ClosedPositionsLimit>,
    /// Pagination offset (0-100000, default: 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<ClosedPositionsOffset>,
    /// Sort criteria (default: REALIZEDPNL).
    #[serde(rename = "sortBy", skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ClosedPositionSortBy>,
    /// Sort direction (default: DESC).
    #[serde(rename = "sortDirection", skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<SortDirection>,
}

/// Request parameters for the `/v1/builders/leaderboard` endpoint.
///
/// Fetches aggregated builder leaderboard rankings. Builders are third-party
/// applications that integrate with Polymarket. Returns one entry per builder
/// with aggregated totals for the specified time period.
///
/// # Optional Parameters
///
/// - `time_period`: Time period to aggregate over (default: DAY).
/// - `limit`: Maximum builders to return (0-50, default: 25).
/// - `offset`: Pagination offset (0-1000, default: 0).
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::{BuilderLeaderboardRequest, TimePeriod};
///
/// let request = BuilderLeaderboardRequest::builder()
///     .time_period(TimePeriod::Week)
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct BuilderLeaderboardRequest {
    /// Time period to aggregate results over (default: DAY).
    #[serde(rename = "timePeriod", skip_serializing_if = "Option::is_none")]
    pub time_period: Option<TimePeriod>,
    /// Maximum number of builders to return (0-50, default: 25).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<BuilderLeaderboardLimit>,
    /// Pagination offset (0-1000, default: 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<BuilderLeaderboardOffset>,
}

/// Request parameters for the `/v1/builders/volume` endpoint.
///
/// Fetches daily time-series volume data for builders. Returns multiple
/// entries per builder (one per day), each including a timestamp. No pagination.
///
/// # Optional Parameters
///
/// - `time_period`: Time period to fetch daily records for (default: DAY).
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::{BuilderVolumeRequest, TimePeriod};
///
/// let request = BuilderVolumeRequest::builder()
///     .time_period(TimePeriod::Month)
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct BuilderVolumeRequest {
    /// Time period to fetch daily records for (default: DAY).
    #[serde(rename = "timePeriod", skip_serializing_if = "Option::is_none")]
    pub time_period: Option<TimePeriod>,
}

/// Request parameters for the `/v1/leaderboard` endpoint.
///
/// Fetches trader leaderboard rankings filtered by category, time period,
/// and ordering. Returns ranked traders with their volume and `PnL` stats.
///
/// # Optional Parameters
///
/// - `category`: Market category filter (default: OVERALL).
/// - `time_period`: Time period for results (default: DAY).
/// - `order_by`: Ordering criteria - PNL or VOL (default: PNL).
/// - `limit`: Maximum traders to return (1-50, default: 25).
/// - `offset`: Pagination offset (0-1000, default: 0).
/// - `user`: Filter to a single user by address.
/// - `user_name`: Filter to a single user by username.
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data_api::types::{TraderLeaderboardRequest, LeaderboardCategory, TimePeriod, LeaderboardOrderBy};
///
/// let request = TraderLeaderboardRequest::builder()
///     .category(LeaderboardCategory::Politics)
///     .time_period(TimePeriod::Week)
///     .order_by(LeaderboardOrderBy::Vol)
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct TraderLeaderboardRequest {
    /// Market category filter (default: OVERALL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<LeaderboardCategory>,
    /// Time period for leaderboard results (default: DAY).
    #[serde(rename = "timePeriod", skip_serializing_if = "Option::is_none")]
    pub time_period: Option<TimePeriod>,
    /// Ordering criteria (default: PNL).
    #[serde(rename = "orderBy", skip_serializing_if = "Option::is_none")]
    pub order_by: Option<LeaderboardOrderBy>,
    /// Maximum number of traders to return (1-50, default: 25).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<TraderLeaderboardLimit>,
    /// Pagination offset (0-1000, default: 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<TraderLeaderboardOffset>,
    /// Filter to a single user by address.
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Address>,
    /// Filter to a single user by username.
    #[builder(into)]
    #[serde(rename = "userName", skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}

// =============================================================================
// Response Types
// =============================================================================

/// Response from the health check endpoint (`/`).
///
/// Returns "OK" when the API is healthy and operational.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct HealthResponse {
    /// Health status message (typically "OK").
    pub data: String,
}

/// Error response returned by the API on failure.
///
/// Contains an error message describing what went wrong.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct ErrorResponse {
    /// Human-readable error message.
    pub error: String,
}

/// A user's current (open) position in a prediction market.
///
/// Returned by the `/positions` endpoint. Represents holdings of outcome tokens
/// with associated profit/loss calculations.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Position {
    /// The user's proxy wallet address.
    pub proxy_wallet: Address,
    /// The outcome token asset identifier.
    pub asset: String,
    /// The market condition ID (unique market identifier).
    pub condition_id: Hash64,
    /// Number of outcome tokens held.
    pub size: f64,
    /// Average entry price for the position.
    pub avg_price: f64,
    /// Initial value (cost basis) of the position.
    pub initial_value: f64,
    /// Current market value of the position.
    pub current_value: f64,
    /// Unrealized cash profit/loss.
    pub cash_pnl: f64,
    /// Unrealized percentage profit/loss.
    pub percent_pnl: f64,
    /// Total amount bought (cumulative).
    pub total_bought: f64,
    /// Realized profit/loss from closed portions.
    pub realized_pnl: f64,
    /// Realized percentage profit/loss.
    pub percent_realized_pnl: f64,
    /// Current market price of the outcome.
    pub cur_price: f64,
    /// Whether the position can be redeemed (market resolved).
    pub redeemable: bool,
    /// Whether the position can be merged with opposite outcome.
    pub mergeable: bool,
    /// Market title/question.
    pub title: String,
    /// Market URL slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Parent event URL slug.
    pub event_slug: String,
    /// Outcome name (e.g., "Yes", "No", candidate name).
    pub outcome: String,
    /// Outcome index within the market (0 or 1 for binary markets).
    pub outcome_index: i32,
    /// Name of the opposite outcome.
    pub opposite_outcome: String,
    /// Asset identifier of the opposite outcome.
    pub opposite_asset: String,
    /// Market end/resolution date.
    pub end_date: String,
    /// Whether this is a negative risk market.
    pub negative_risk: bool,
}

/// A user's closed (historical) position in a prediction market.
///
/// Returned by the `/closed-positions` endpoint. Represents positions that
/// have been fully sold or redeemed, with final profit/loss figures.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ClosedPosition {
    /// The user's proxy wallet address.
    pub proxy_wallet: Address,
    /// The outcome token asset identifier.
    pub asset: String,
    /// The market condition ID (unique market identifier).
    pub condition_id: Hash64,
    /// Average entry price for the position.
    pub avg_price: f64,
    /// Total amount bought (cumulative).
    pub total_bought: f64,
    /// Realized profit/loss from the closed position.
    pub realized_pnl: f64,
    /// Final market price when position was closed.
    pub cur_price: f64,
    /// Unix timestamp when the position was closed.
    pub timestamp: i64,
    /// Market title/question.
    pub title: String,
    /// Market URL slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Parent event URL slug.
    pub event_slug: String,
    /// Outcome name (e.g., "Yes", "No", candidate name).
    pub outcome: String,
    /// Outcome index within the market (0 or 1 for binary markets).
    pub outcome_index: i32,
    /// Name of the opposite outcome.
    pub opposite_outcome: String,
    /// Asset identifier of the opposite outcome.
    pub opposite_asset: String,
    /// Market end/resolution date.
    pub end_date: String,
}

/// A trade (buy or sell) of outcome tokens.
///
/// Returned by the `/trades` endpoint. Represents an executed order where
/// outcome tokens were bought or sold.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Trade {
    /// The trader's proxy wallet address.
    pub proxy_wallet: Address,
    /// Trade side (BUY or SELL).
    pub side: Side,
    /// The outcome token asset identifier.
    pub asset: String,
    /// The market condition ID (unique market identifier).
    pub condition_id: Hash64,
    /// Number of tokens traded.
    pub size: f64,
    /// Execution price per token.
    pub price: f64,
    /// Unix timestamp when the trade occurred.
    pub timestamp: i64,
    /// Market title/question.
    pub title: String,
    /// Market URL slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Parent event URL slug.
    pub event_slug: String,
    /// Outcome name (e.g., "Yes", "No", candidate name).
    pub outcome: String,
    /// Outcome index within the market (0 or 1 for binary markets).
    pub outcome_index: i32,
    /// Trader's display name (if public).
    pub name: Option<String>,
    /// Trader's pseudonym (if set).
    pub pseudonym: Option<String>,
    /// Trader's bio (if public).
    pub bio: Option<String>,
    /// Trader's profile image URL.
    pub profile_image: Option<String>,
    /// Trader's optimized profile image URL.
    pub profile_image_optimized: Option<String>,
    /// On-chain transaction hash.
    pub transaction_hash: String,
}

/// An on-chain activity record for a user.
///
/// Returned by the `/activity` endpoint. Represents various on-chain operations
/// including trades, splits, merges, redemptions, rewards, and conversions.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Activity {
    /// The user's proxy wallet address.
    pub proxy_wallet: Address,
    /// Unix timestamp when the activity occurred.
    pub timestamp: i64,
    /// The market condition ID (unique market identifier).
    pub condition_id: Hash64,
    /// Type of activity (TRADE, SPLIT, MERGE, REDEEM, REWARD, CONVERSION).
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    /// Number of tokens involved in the activity.
    pub size: f64,
    /// USDC value of the activity.
    pub usdc_size: f64,
    /// On-chain transaction hash.
    pub transaction_hash: String,
    /// Price per token (for trades).
    pub price: Option<f64>,
    /// Outcome token asset identifier (for trades).
    pub asset: Option<String>,
    /// Trade side (for trades only).
    pub side: Option<Side>,
    /// Outcome index (for trades).
    pub outcome_index: Option<i32>,
    /// Market title/question.
    pub title: Option<String>,
    /// Market URL slug.
    pub slug: Option<String>,
    /// Market icon URL.
    pub icon: Option<String>,
    /// Parent event URL slug.
    pub event_slug: Option<String>,
    /// Outcome name.
    pub outcome: Option<String>,
    /// User's display name (if public).
    pub name: Option<String>,
    /// User's pseudonym (if set).
    pub pseudonym: Option<String>,
    /// User's bio (if public).
    pub bio: Option<String>,
    /// User's profile image URL.
    pub profile_image: Option<String>,
    /// User's optimized profile image URL.
    pub profile_image_optimized: Option<String>,
}

/// A holder of outcome tokens in a market.
///
/// Represents a user who holds a position in a specific outcome.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Holder {
    /// The holder's proxy wallet address.
    pub proxy_wallet: Address,
    /// Holder's bio (if public).
    pub bio: Option<String>,
    /// The outcome token asset identifier.
    pub asset: String,
    /// Holder's pseudonym (if set).
    pub pseudonym: Option<String>,
    /// Amount of tokens held.
    pub amount: f64,
    /// Whether the holder's username is publicly visible.
    pub display_username_public: Option<bool>,
    /// Outcome index within the market (0 or 1 for binary markets).
    pub outcome_index: i32,
    /// Holder's display name (if public).
    pub name: Option<String>,
    /// Holder's profile image URL.
    pub profile_image: Option<String>,
    /// Holder's optimized profile image URL.
    pub profile_image_optimized: Option<String>,
}

/// Container for holders grouped by token.
///
/// Returned by the `/holders` endpoint. Groups holders by outcome token.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct MetaHolder {
    /// The outcome token identifier.
    pub token: String,
    /// List of holders for this token.
    pub holders: Vec<Holder>,
}

/// Count of unique markets a user has traded.
///
/// Returned by the `/traded` endpoint.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Traded {
    /// The user's address.
    pub user: Address,
    /// Number of unique markets traded.
    pub traded: i32,
}

/// Total value of a user's positions.
///
/// Returned by the `/value` endpoint.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Value {
    /// The user's address.
    pub user: Address,
    /// Total value of positions in USDC.
    pub value: f64,
}

/// Open interest for a market.
///
/// Returned by the `/oi` endpoint. Open interest represents the total
/// value of outstanding positions in a market.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct OpenInterest {
    /// The market condition ID.
    pub market: Hash64,
    /// Open interest value in USDC.
    pub value: f64,
}

/// Trading volume for a specific market.
///
/// Used within [`LiveVolume`] to show per-market volume breakdown.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct MarketVolume {
    /// The market condition ID.
    pub market: Hash64,
    /// Trading volume in USDC.
    pub value: f64,
}

/// Live trading volume for an event.
///
/// Returned by the `/live-volume` endpoint. Includes total volume
/// and per-market breakdown.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct LiveVolume {
    /// Total trading volume across all markets in the event.
    pub total: f64,
    /// Per-market volume breakdown.
    pub markets: Vec<MarketVolume>,
}

/// A builder's entry in the aggregated leaderboard.
///
/// Returned by the `/v1/builders/leaderboard` endpoint. Builders are third-party
/// applications that integrate with Polymarket.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BuilderLeaderboardEntry {
    /// Rank position in the leaderboard.
    pub rank: String,
    /// Builder name or identifier.
    pub builder: String,
    /// Total trading volume attributed to this builder.
    pub volume: f64,
    /// Number of active users for this builder.
    pub active_users: i32,
    /// Whether the builder is verified.
    pub verified: bool,
    /// URL to the builder's logo image.
    pub builder_logo: Option<String>,
}

/// A builder's daily volume data point.
///
/// Returned by the `/v1/builders/volume` endpoint. Each entry represents
/// a single day's volume and activity for a builder.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BuilderVolumeEntry {
    /// Timestamp for this entry in ISO 8601 format (e.g., "2025-11-15T00:00:00Z").
    pub dt: String,
    /// Builder name or identifier.
    pub builder: String,
    /// URL to the builder's logo image.
    pub builder_logo: Option<String>,
    /// Whether the builder is verified.
    pub verified: bool,
    /// Trading volume for this builder on this date.
    pub volume: f64,
    /// Number of active users for this builder on this date.
    pub active_users: i32,
    /// Rank position on this date.
    pub rank: String,
}

/// A trader's entry in the leaderboard.
///
/// Returned by the `/v1/leaderboard` endpoint. Shows trader rankings
/// by profit/loss or volume.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TraderLeaderboardEntry {
    /// Rank position in the leaderboard.
    pub rank: String,
    /// The trader's proxy wallet address.
    pub proxy_wallet: Address,
    /// The trader's username.
    pub user_name: Option<String>,
    /// Trading volume for this trader.
    pub vol: f64,
    /// Profit and loss for this trader.
    pub pnl: f64,
    /// URL to the trader's profile image.
    pub profile_image: Option<String>,
    /// The trader's X (Twitter) username.
    pub x_username: Option<String>,
    /// Whether the trader has a verified badge.
    pub verified_badge: Option<bool>,
}
