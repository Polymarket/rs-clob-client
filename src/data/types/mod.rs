use std::fmt;

use serde::de::StdError;
use serde::{Deserialize, Serialize};
use serde_with::{StringWithSeparator, formats::CommaSeparator, serde_as};

use crate::types::{B256, Decimal};

pub mod request;
pub mod response;

/// The side of a trade (buy or sell).
///
/// Used to indicate whether a trade was a purchase or sale of outcome tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum Side {
    /// Buying outcome tokens (going long on an outcome).
    Buy,
    /// Selling outcome tokens (going short or closing a long position).
    Sell,
    /// Unknown side from the API (captures the raw value for debugging).
    #[serde(untagged)]
    Unknown(String),
}

/// The type of on-chain activity for a user.
///
/// Activities represent various operations that users can perform on the Polymarket protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
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
    /// Yield
    Yield,
    /// Maker rebate (fee rebate for providing liquidity).
    MakerRebate,
    /// Unknown activity type from the API (captures the raw value for debugging).
    #[serde(untagged)]
    Unknown(String),
}

/// Sort criteria for position queries.
///
/// Determines how positions are ordered in the response. Default is [`Tokens`](Self::Tokens).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display,
)]
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
    #[default]
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
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display,
)]
#[non_exhaustive]
pub enum ClosedPositionSortBy {
    /// Sort by realized profit and loss (default).
    #[default]
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
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum ActivitySortBy {
    /// Sort by activity timestamp (default).
    #[default]
    Timestamp,
    /// Sort by number of tokens involved in the activity.
    Tokens,
    /// Sort by cash (USDC) value of the activity.
    Cash,
}

/// Sort direction for query results.
///
/// Default is [`Desc`](Self::Desc) (descending) for most endpoints.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum SortDirection {
    /// Ascending order (smallest/earliest first).
    Asc,
    /// Descending order (largest/latest first, default).
    #[default]
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
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum TimePeriod {
    /// Last 24 hours (default).
    #[default]
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
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum LeaderboardCategory {
    /// All categories combined (default).
    #[default]
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
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum LeaderboardOrderBy {
    /// Order by profit and loss (default).
    #[default]
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
/// use polymarket_client_sdk::data::types::MarketFilter;
/// use polymarket_client_sdk::types::b256;
///
/// // Filter by specific markets (condition IDs)
/// let by_markets = MarketFilter::markets([b256!("dd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917")]);
///
/// // Or filter by events (which may contain multiple markets)
/// let by_events = MarketFilter::event_ids(["123".to_owned()]);
/// ```
#[serde_as]
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub enum MarketFilter {
    /// Filter by condition IDs (market identifiers).
    #[serde(rename = "market")]
    Markets(#[serde_as(as = "StringWithSeparator::<CommaSeparator, B256>")] Vec<B256>),
    /// Filter by event IDs (groups of related markets).
    #[serde(rename = "eventId")]
    EventIds(#[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")] Vec<String>),
}

impl MarketFilter {
    /// Creates a filter for specific markets by their condition IDs.
    #[must_use]
    pub fn markets<I: IntoIterator<Item = B256>>(ids: I) -> Self {
        Self::Markets(ids.into_iter().collect())
    }

    /// Creates a filter for all markets within the specified events.
    #[must_use]
    pub fn event_ids<I: IntoIterator<Item = String>>(ids: I) -> Self {
        Self::EventIds(ids.into_iter().collect())
    }
}

/// Error type for bounded integer values that are out of range.
#[derive(Debug)]
#[non_exhaustive]
pub struct BoundedIntError {
    /// The value that was out of range.
    pub value: i32,
    /// The minimum allowed value.
    pub min: i32,
    /// The maximum allowed value.
    pub max: i32,
    /// The name of the parameter.
    pub param_name: &'static str,
}

impl BoundedIntError {
    /// Creates a new `BoundedIntError`.
    #[must_use]
    pub const fn new(value: i32, min: i32, max: i32, param_name: &'static str) -> Self {
        Self {
            value,
            min,
            max,
            param_name,
        }
    }
}

impl fmt::Display for BoundedIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} must be between {} and {} (got {})",
            self.param_name, self.min, self.max, self.value
        )
    }
}

impl StdError for BoundedIntError {}

/// A filter for minimum trade size.
///
/// Used to filter trades by a minimum value, either in USDC (cash) or tokens.
/// Both `filter_type` and `filter_amount` must be provided together to the API.
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::data::types::TradeFilter;
/// use rust_decimal_macros::dec;
///
/// // Filter trades with at least $100 USDC value
/// let filter = TradeFilter::cash(dec!(100)).unwrap();
///
/// // Filter trades with at least 50 tokens
/// let filter = TradeFilter::tokens(dec!(50)).unwrap();
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TradeFilter {
    /// The type of filter (cash or tokens).
    pub filter_type: FilterType,
    /// The minimum amount to filter by (must be >= 0).
    pub filter_amount: Decimal,
}

impl TradeFilter {
    /// Creates a new trade filter with the specified type and amount.
    ///
    /// # Errors
    ///
    /// Returns [`TradeFilterError`] if the amount is negative.
    pub fn new(filter_type: FilterType, filter_amount: Decimal) -> Result<Self, TradeFilterError> {
        if filter_amount.is_sign_negative() {
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
    pub fn cash(amount: Decimal) -> Result<Self, TradeFilterError> {
        Self::new(FilterType::Cash, amount)
    }

    /// Creates a token quantity filter.
    ///
    /// # Errors
    ///
    /// Returns [`TradeFilterError`] if the amount is negative.
    pub fn tokens(amount: Decimal) -> Result<Self, TradeFilterError> {
        Self::new(FilterType::Tokens, amount)
    }
}

/// Error type for invalid trade filter values.
#[derive(Debug)]
#[non_exhaustive]
pub enum TradeFilterError {
    /// The filter amount was negative.
    NegativeAmount(Decimal),
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
