//! Request types for the Polymarket Data API.
//!
//! This module contains builder-pattern structs for each API endpoint.
//! All request types use the [`bon`](https://docs.rs/bon) crate for the builder pattern.

#![allow(
    clippy::module_name_repetitions,
    reason = "Request suffix is intentional for clarity"
)]

use bon::Builder;
use rust_decimal::Decimal;
use serde::Serialize;

use super::common::{
    ActivitySortBy, ActivityType, Address, BoundedIntError, ClosedPositionSortBy, Hash64,
    LeaderboardCategory, LeaderboardOrderBy, MarketFilter, PositionSortBy, Side, SortDirection,
    TimePeriod, Title, TradeFilter,
};
use crate::data_api::ser::{comma_separated, comma_separated_vec, is_empty_vec, vec_is_empty};

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

/// Trait for converting request types to URL query strings.
///
/// This trait is automatically implemented for all types that implement [`Serialize`].
/// It uses [`serde_urlencoded`] to serialize the struct fields into a query string.
pub trait ToQueryString: Serialize {
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

impl<T: Serialize> ToQueryString for T {}

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
/// use alloy::primitives::address;
/// use polymarket_client_sdk::data_api::{request::PositionsRequest, common::{PositionSortBy, SortDirection}};
///
/// let request = PositionsRequest::builder()
///     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
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
    pub size_threshold: Option<Decimal>,
    /// Only return positions that can be redeemed (default: false).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redeemable: Option<bool>,
    /// Only return positions that can be merged (default: false).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mergeable: Option<bool>,
    /// Maximum number of positions to return (0-500, default: 100).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 500, "limit") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Pagination offset (0-10000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 10000, "offset") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
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
/// use alloy::primitives::address;
/// use polymarket_client_sdk::data_api::{request::TradesRequest, common::{Side, TradeFilter}};
/// use rust_decimal_macros::dec;
///
/// let request = TradesRequest::builder()
///     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
///     .side(Side::Buy)
///     .trade_filter(TradeFilter::cash(dec!(100)).unwrap())
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
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 10000, "limit") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Pagination offset (0-10000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 10000, "offset") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
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
/// use alloy::primitives::address;
/// use polymarket_client_sdk::data_api::{request::ActivityRequest, common::ActivityType};
///
/// let request = ActivityRequest::builder()
///     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
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
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 500, "limit") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Pagination offset (0-10000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 10000, "offset") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
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
/// use polymarket_client_sdk::data_api::request::HoldersRequest;
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
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 20, "limit") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Minimum balance to include (0-999999, default: 1).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 999_999, "min_balance") })]
    #[serde(rename = "minBalance", skip_serializing_if = "Option::is_none")]
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
/// use alloy::primitives::address;
/// use polymarket_client_sdk::data_api::{request::ClosedPositionsRequest, common::ClosedPositionSortBy};
///
/// let request = ClosedPositionsRequest::builder()
///     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
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
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 50, "limit") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Pagination offset (0-100000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 100_000, "offset") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
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
/// use polymarket_client_sdk::data_api::{request::BuilderLeaderboardRequest, common::TimePeriod};
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
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 50, "limit") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Pagination offset (0-1000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 1000, "offset") })]
    #[serde(skip_serializing_if = "Option::is_none")]
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
/// use polymarket_client_sdk::data_api::{request::BuilderVolumeRequest, common::TimePeriod};
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
/// use polymarket_client_sdk::data_api::{request::TraderLeaderboardRequest, common::{LeaderboardCategory, TimePeriod, LeaderboardOrderBy}};
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
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 1, 50, "limit") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Pagination offset (0-1000, default: 0).
    #[builder(with = |v: i32| -> Result<_, BoundedIntError> { validate_bound(v, 0, 1000, "offset") })]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    /// Filter to a single user by address.
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Address>,
    /// Filter to a single user by username.
    #[builder(into)]
    #[serde(rename = "userName", skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
}
