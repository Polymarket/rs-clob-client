//! Request types for the Polymarket Data API.
//!
//! This module contains builder-pattern structs for each API endpoint.
//! All request types use the [`bon`](https://docs.rs/bon) crate for the builder pattern.

#![allow(
    clippy::module_name_repetitions,
    reason = "Request suffix is intentional for clarity"
)]

use bon::Builder;
use serde::Serialize;
use serde_with::{StringWithSeparator, formats::CommaSeparator, serde_as, skip_serializing_none};

use super::{
    ActivitySortBy, ActivityType, BoundedIntError, ClosedPositionSortBy, LeaderboardCategory,
    LeaderboardOrderBy, MarketFilter, PositionSortBy, Side, SortDirection, TimePeriod, TradeFilter,
};
use crate::types::{Address, B256, Decimal};

/// Validates that an i32 value is within the specified bounds.
fn validate_bound(
    value: i32,
    min: i32,
    max: i32,
    param_name: &'static str,
) -> Result<i32, BoundedIntError> {
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(BoundedIntError::new(value, min, max, param_name))
    }
}

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
/// use polymarket_client_sdk::types::address;
/// use polymarket_client_sdk::data::{types::request::PositionsRequest, types::{PositionSortBy, SortDirection}};
///
/// let request = PositionsRequest::builder()
///     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
///     .sort_by(PositionSortBy::CashPnl)
///     .sort_direction(SortDirection::Desc)
///     .build();
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct PositionsRequest {
    /// User address (required).
    #[builder(into)]
    pub user: Address,
    /// Filter by markets or events. Mutually exclusive options.
    #[serde(flatten, skip_serializing_if = "filter_is_none_or_empty")]
    pub filter: Option<MarketFilter>,
    /// Minimum position size to include (default: 1).
    #[serde(rename = "sizeThreshold")]
    pub size_threshold: Option<Decimal>,
    /// Only return positions that can be redeemed (default: false).
    pub redeemable: Option<bool>,
    /// Only return positions that can be merged (default: false).
    pub mergeable: Option<bool>,
    /// Maximum number of positions to return (0-500, default: 100).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 500, "limit") })]
    pub limit: Option<i32>,
    /// Pagination offset (0-10000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 10000, "offset") })]
    pub offset: Option<i32>,
    /// Sort criteria (default: TOKENS).
    #[serde(rename = "sortBy")]
    pub sort_by: Option<PositionSortBy>,
    /// Sort direction (default: DESC).
    #[serde(rename = "sortDirection")]
    pub sort_direction: Option<SortDirection>,
    /// Filter by market title substring (max 100 chars).
    #[builder(into)]
    pub title: Option<String>,
}

#[expect(clippy::ref_option, reason = "Need an explicit reference for serde")]
fn filter_is_none_or_empty(f: &Option<MarketFilter>) -> bool {
    match f {
        None => true,
        Some(MarketFilter::Markets(v)) => v.is_empty(),
        Some(MarketFilter::EventIds(v)) => v.is_empty(),
    }
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
/// use polymarket_client_sdk::types::address;
/// use polymarket_client_sdk::data::{types::request::TradesRequest, types::{Side, TradeFilter}};
/// use rust_decimal_macros::dec;
///
/// let request = TradesRequest::builder()
///     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
///     .side(Side::Buy)
///     .trade_filter(TradeFilter::cash(dec!(100)).unwrap())
///     .build();
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct TradesRequest {
    /// Filter by user address.
    #[builder(into)]
    pub user: Option<Address>,
    /// Filter by markets or events. Mutually exclusive options.
    #[serde(flatten)]
    pub filter: Option<MarketFilter>,
    /// Maximum number of trades to return (0-10000, default: 100).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 10000, "limit") })]
    pub limit: Option<i32>,
    /// Pagination offset (0-10000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 10000, "offset") })]
    pub offset: Option<i32>,
    /// Only return taker trades (default: true).
    #[serde(rename = "takerOnly")]
    pub taker_only: Option<bool>,
    /// Filter by minimum trade size. Must provide both type and amount.
    #[serde(flatten)]
    pub trade_filter: Option<TradeFilter>,
    /// Filter by trade side (BUY or SELL).
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
/// use polymarket_client_sdk::types::address;
/// use polymarket_client_sdk::data::{types::request::ActivityRequest, types::ActivityType};
///
/// let request = ActivityRequest::builder()
///     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
///     .activity_types(vec![ActivityType::Trade, ActivityType::Redeem])
///     .build();
/// ```
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct ActivityRequest {
    /// User address (required).
    #[builder(into)]
    pub user: Address,
    /// Filter by markets or events. Mutually exclusive options.
    #[serde(flatten)]
    pub filter: Option<MarketFilter>,
    /// Filter by activity types.
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, ActivityType>")]
    #[builder(default)]
    #[serde(rename = "type", skip_serializing_if = "Vec::is_empty")]
    pub activity_types: Vec<ActivityType>,
    /// Maximum number of activities to return (0-500, default: 100).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 500, "limit") })]
    pub limit: Option<i32>,
    /// Pagination offset (0-10000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 10000, "offset") })]
    pub offset: Option<i32>,
    /// Start timestamp filter (Unix timestamp, minimum: 0).
    pub start: Option<u64>,
    /// End timestamp filter (Unix timestamp, minimum: 0).
    pub end: Option<u64>,
    /// Sort criteria (default: TIMESTAMP).
    #[serde(rename = "sortBy")]
    pub sort_by: Option<ActivitySortBy>,
    /// Sort direction (default: DESC).
    #[serde(rename = "sortDirection")]
    pub sort_direction: Option<SortDirection>,
    /// Filter by trade side (only applies to TRADE activities).
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
/// use polymarket_client_sdk::data::types::request::HoldersRequest;
/// use polymarket_client_sdk::types::b256;
///
/// let request = HoldersRequest::builder()
///     .markets(vec![b256!("dd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917")])
///     .build();
/// ```
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct HoldersRequest {
    /// Condition IDs of markets to query (required).
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, B256>")]
    #[serde(rename = "market", skip_serializing_if = "Vec::is_empty")]
    pub markets: Vec<B256>,
    /// Maximum holders to return per token (0-20, default: 20).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 20, "limit") })]
    pub limit: Option<i32>,
    /// Minimum balance to include (0-999999, default: 1).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 999_999, "min_balance") })]
    #[serde(rename = "minBalance")]
    pub min_balance: Option<i32>,
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
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct ValueRequest {
    /// User address (required).
    #[builder(into)]
    pub user: Address,
    /// Optional list of condition IDs to filter by.
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, B256>")]
    #[builder(default)]
    #[serde(rename = "market", skip_serializing_if = "Vec::is_empty")]
    pub markets: Vec<B256>,
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
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct OpenInterestRequest {
    /// Optional list of condition IDs to filter by.
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, B256>")]
    #[builder(default)]
    #[serde(rename = "market", skip_serializing_if = "Vec::is_empty")]
    pub markets: Vec<B256>,
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
/// use polymarket_client_sdk::types::address;
/// use polymarket_client_sdk::data::{types::request::ClosedPositionsRequest, types::ClosedPositionSortBy};
///
/// let request = ClosedPositionsRequest::builder()
///     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
///     .sort_by(ClosedPositionSortBy::Timestamp)
///     .build();
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct ClosedPositionsRequest {
    /// User address (required).
    #[builder(into)]
    pub user: Address,
    /// Filter by markets or events. Mutually exclusive options.
    #[serde(flatten)]
    pub filter: Option<MarketFilter>,
    /// Filter by market title substring (max 100 chars).
    #[builder(into)]
    pub title: Option<String>,
    /// Maximum number of positions to return (0-50, default: 10).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 50, "limit") })]
    pub limit: Option<i32>,
    /// Pagination offset (0-100000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 100_000, "offset") })]
    pub offset: Option<i32>,
    /// Sort criteria (default: REALIZEDPNL).
    #[serde(rename = "sortBy")]
    pub sort_by: Option<ClosedPositionSortBy>,
    /// Sort direction (default: DESC).
    #[serde(rename = "sortDirection")]
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
/// use polymarket_client_sdk::data::{types::request::BuilderLeaderboardRequest, types::TimePeriod};
///
/// let request = BuilderLeaderboardRequest::builder()
///     .time_period(TimePeriod::Week)
///     .build();
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct BuilderLeaderboardRequest {
    /// Time period to aggregate results over (default: DAY).
    #[serde(rename = "timePeriod")]
    pub time_period: Option<TimePeriod>,
    /// Maximum number of builders to return (0-50, default: 25).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 50, "limit") })]
    pub limit: Option<i32>,
    /// Pagination offset (0-1000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 1000, "offset") })]
    pub offset: Option<i32>,
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
/// use polymarket_client_sdk::data::{types::request::BuilderVolumeRequest, types::TimePeriod};
///
/// let request = BuilderVolumeRequest::builder()
///     .time_period(TimePeriod::Month)
///     .build();
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct BuilderVolumeRequest {
    /// Time period to fetch daily records for (default: DAY).
    #[serde(rename = "timePeriod")]
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
/// use polymarket_client_sdk::data::{types::request::TraderLeaderboardRequest, types::{LeaderboardCategory, TimePeriod, LeaderboardOrderBy}};
///
/// let request = TraderLeaderboardRequest::builder()
///     .category(LeaderboardCategory::Politics)
///     .time_period(TimePeriod::Week)
///     .order_by(LeaderboardOrderBy::Vol)
///     .build();
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct TraderLeaderboardRequest {
    /// Market category filter (default: OVERALL).
    pub category: Option<LeaderboardCategory>,
    /// Time period for leaderboard results (default: DAY).
    #[serde(rename = "timePeriod")]
    pub time_period: Option<TimePeriod>,
    /// Ordering criteria (default: PNL).
    #[serde(rename = "orderBy")]
    pub order_by: Option<LeaderboardOrderBy>,
    /// Maximum number of traders to return (1-50, default: 25).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 1, 50, "limit") })]
    pub limit: Option<i32>,
    /// Pagination offset (0-1000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 1000, "offset") })]
    pub offset: Option<i32>,
    /// Filter to a single user by address.
    #[builder(into)]
    pub user: Option<Address>,
    /// Filter to a single user by username.
    #[builder(into)]
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
}
