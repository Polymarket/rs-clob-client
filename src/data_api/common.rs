//! Common types for the Polymarket Data API.
//!
//! This module contains fundamental types used across the Data API,
//! including type aliases, enums for sorting and filtering, bounded
//! integer types, and filter constructs.

use std::error::Error as StdError;
use std::fmt;

/// Re-export of alloy's Address type for Ethereum addresses.
pub use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

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
/// use polymarket_client_sdk::data_api::common::MarketFilter;
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
/// use polymarket_client_sdk::data_api::common::TradeFilter;
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
