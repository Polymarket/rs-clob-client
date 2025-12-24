//! Request types for the Polymarket Data API.
//!
//! This module contains builder-pattern request types for all Data API endpoints.
//! Each request type corresponds to an API endpoint and includes all optional
//! query parameters documented in the `OpenAPI` specification.

use bon::Builder;

/// Formats a float for use in query parameters, avoiding scientific notation.
fn format_query_float(v: f64) -> String {
    let s = format!("{v:.15}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

use super::common::{
    ActivityLimit, ActivityOffset, ActivitySortBy, ActivityType, Address, BuilderLeaderboardLimit,
    BuilderLeaderboardOffset, ClosedPositionSortBy, ClosedPositionsLimit, ClosedPositionsOffset,
    EventId, Hash64, HoldersLimit, HoldersMinBalance, LeaderboardCategory, LeaderboardOrderBy,
    MarketFilter, PositionSortBy, PositionsLimit, PositionsOffset, Side, SortDirection, TimePeriod,
    Title, TradeFilter, TraderLeaderboardLimit, TraderLeaderboardOffset, TradesLimit, TradesOffset,
};

/// Trait for converting request types to query parameter vectors.
pub trait QueryParams {
    /// Converts the request to a vector of query parameter key-value pairs.
    #[must_use]
    fn query_params(&self) -> Vec<(&'static str, String)>;
}

impl QueryParams for () {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![]
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
/// use polymarket_client_sdk::data_api::types::{PositionsRequest, Address, PositionSortBy, SortDirection};
///
/// let request = PositionsRequest::builder()
///     .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
///     .sort_by(PositionSortBy::CashPnl)
///     .sort_direction(SortDirection::Desc)
///     .build();
/// ```
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct PositionsRequest {
    /// User address (required).
    pub user: Address,
    /// Filter by markets or events. Mutually exclusive options.
    pub filter: Option<MarketFilter>,
    /// Minimum position size to include (default: 1).
    pub size_threshold: Option<f64>,
    /// Only return positions that can be redeemed (default: false).
    pub redeemable: Option<bool>,
    /// Only return positions that can be merged (default: false).
    pub mergeable: Option<bool>,
    /// Maximum number of positions to return (0-500, default: 100).
    pub limit: Option<PositionsLimit>,
    /// Pagination offset (0-10000, default: 0).
    pub offset: Option<PositionsOffset>,
    /// Sort criteria (default: TOKENS).
    pub sort_by: Option<PositionSortBy>,
    /// Sort direction (default: DESC).
    pub sort_direction: Option<SortDirection>,
    /// Filter by market title substring (max 100 chars).
    pub title: Option<Title>,
}

impl QueryParams for PositionsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("user", self.user.to_string())];
        if let Some(f) = &self.filter {
            f.append_to_params(&mut params);
        }
        if let Some(v) = self.size_threshold {
            params.push(("sizeThreshold", format_query_float(v)));
        }
        if let Some(v) = self.redeemable {
            params.push(("redeemable", v.to_string()));
        }
        if let Some(v) = self.mergeable {
            params.push(("mergeable", v.to_string()));
        }
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = self.sort_by {
            params.push(("sortBy", v.to_string()));
        }
        if let Some(v) = self.sort_direction {
            params.push(("sortDirection", v.to_string()));
        }
        if let Some(v) = &self.title {
            params.push(("title", v.to_string()));
        }
        params
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
/// use polymarket_client_sdk::data_api::types::{TradesRequest, Address, Side, TradeFilter};
///
/// let request = TradesRequest::builder()
///     .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
///     .side(Side::Buy)
///     .trade_filter(TradeFilter::cash(100.0).unwrap())
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct TradesRequest {
    /// Filter by user address.
    pub user: Option<Address>,
    /// Filter by markets or events. Mutually exclusive options.
    pub filter: Option<MarketFilter>,
    /// Maximum number of trades to return (0-10000, default: 100).
    pub limit: Option<TradesLimit>,
    /// Pagination offset (0-10000, default: 0).
    pub offset: Option<TradesOffset>,
    /// Only return taker trades (default: true).
    pub taker_only: Option<bool>,
    /// Filter by minimum trade size. Must provide both type and amount.
    pub trade_filter: Option<TradeFilter>,
    /// Filter by trade side (BUY or SELL).
    pub side: Option<Side>,
}

impl QueryParams for TradesRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = &self.user {
            params.push(("user", v.to_string()));
        }
        if let Some(f) = &self.filter {
            f.append_to_params(&mut params);
        }
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = self.taker_only {
            params.push(("takerOnly", v.to_string()));
        }
        if let Some(f) = &self.trade_filter {
            params.push(("filterType", f.filter_type.to_string()));
            params.push(("filterAmount", format_query_float(f.filter_amount)));
        }
        if let Some(v) = self.side {
            params.push(("side", v.to_string()));
        }
        params
    }
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
/// use polymarket_client_sdk::data_api::types::{ActivityRequest, Address, ActivityType};
///
/// let request = ActivityRequest::builder()
///     .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
///     .activity_types(vec![ActivityType::Trade, ActivityType::Redeem])
///     .build();
/// ```
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct ActivityRequest {
    /// User address (required).
    pub user: Address,
    /// Filter by markets or events. Mutually exclusive options.
    pub filter: Option<MarketFilter>,
    /// Filter by activity types.
    pub activity_types: Option<Vec<ActivityType>>,
    /// Maximum number of activities to return (0-500, default: 100).
    pub limit: Option<ActivityLimit>,
    /// Pagination offset (0-10000, default: 0).
    pub offset: Option<ActivityOffset>,
    /// Start timestamp filter (Unix timestamp, minimum: 0).
    pub start: Option<u64>,
    /// End timestamp filter (Unix timestamp, minimum: 0).
    pub end: Option<u64>,
    /// Sort criteria (default: TIMESTAMP).
    pub sort_by: Option<ActivitySortBy>,
    /// Sort direction (default: DESC).
    pub sort_direction: Option<SortDirection>,
    /// Filter by trade side (only applies to TRADE activities).
    pub side: Option<Side>,
}

impl QueryParams for ActivityRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("user", self.user.to_string())];
        if let Some(f) = &self.filter {
            f.append_to_params(&mut params);
        }
        if let Some(types) = &self.activity_types
            && !types.is_empty()
        {
            let s = types
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("type", s));
        }
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = self.start {
            params.push(("start", v.to_string()));
        }
        if let Some(v) = self.end {
            params.push(("end", v.to_string()));
        }
        if let Some(v) = self.sort_by {
            params.push(("sortBy", v.to_string()));
        }
        if let Some(v) = self.sort_direction {
            params.push(("sortDirection", v.to_string()));
        }
        if let Some(v) = self.side {
            params.push(("side", v.to_string()));
        }
        params
    }
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
/// use polymarket_client_sdk::data_api::types::{HoldersRequest, Hash64};
///
/// let market = Hash64::new("0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917").unwrap();
/// let request = HoldersRequest::builder()
///     .markets(vec![market])
///     .build();
/// ```
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct HoldersRequest {
    /// Condition IDs of markets to query (required).
    pub markets: Vec<Hash64>,
    /// Maximum holders to return per token (0-20, default: 20).
    pub limit: Option<HoldersLimit>,
    /// Minimum balance to include (0-999999, default: 1).
    pub min_balance: Option<HoldersMinBalance>,
}

impl QueryParams for HoldersRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if !self.markets.is_empty() {
            let s = self
                .markets
                .iter()
                .map(Hash64::as_str)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("market", s));
        }
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.min_balance {
            params.push(("minBalance", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/traded` endpoint.
///
/// Fetches the total count of unique markets a user has traded.
///
/// # Required Parameters
///
/// - `user`: The Ethereum address of the user to query.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct TradedRequest {
    /// User address (required).
    pub user: Address,
}

impl QueryParams for TradedRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![("user", self.user.to_string())]
    }
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
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct ValueRequest {
    /// User address (required).
    pub user: Address,
    /// Optional list of condition IDs to filter by.
    pub markets: Option<Vec<Hash64>>,
}

impl QueryParams for ValueRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("user", self.user.to_string())];
        if let Some(markets) = &self.markets
            && !markets.is_empty()
        {
            let s = markets
                .iter()
                .map(Hash64::as_str)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("market", s));
        }
        params
    }
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
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct OpenInterestRequest {
    /// Optional list of condition IDs to filter by.
    pub markets: Option<Vec<Hash64>>,
}

impl QueryParams for OpenInterestRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(markets) = &self.markets
            && !markets.is_empty()
        {
            let s = markets
                .iter()
                .map(Hash64::as_str)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("market", s));
        }
        params
    }
}

/// Request parameters for the `/live-volume` endpoint.
///
/// Fetches live trading volume for an event, including total volume
/// and per-market breakdown.
///
/// # Required Parameters
///
/// - `id`: The event ID to query.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct LiveVolumeRequest {
    /// Event ID (required, must be >= 1).
    pub id: EventId,
}

impl QueryParams for LiveVolumeRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![("id", self.id.to_string())]
    }
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
/// use polymarket_client_sdk::data_api::types::{ClosedPositionsRequest, Address, ClosedPositionSortBy};
///
/// let request = ClosedPositionsRequest::builder()
///     .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
///     .sort_by(ClosedPositionSortBy::Timestamp)
///     .build();
/// ```
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct ClosedPositionsRequest {
    /// User address (required).
    pub user: Address,
    /// Filter by markets or events. Mutually exclusive options.
    pub filter: Option<MarketFilter>,
    /// Filter by market title substring (max 100 chars).
    pub title: Option<Title>,
    /// Maximum number of positions to return (0-50, default: 10).
    pub limit: Option<ClosedPositionsLimit>,
    /// Pagination offset (0-100000, default: 0).
    pub offset: Option<ClosedPositionsOffset>,
    /// Sort criteria (default: REALIZEDPNL).
    pub sort_by: Option<ClosedPositionSortBy>,
    /// Sort direction (default: DESC).
    pub sort_direction: Option<SortDirection>,
}

impl QueryParams for ClosedPositionsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("user", self.user.to_string())];
        if let Some(f) = &self.filter {
            f.append_to_params(&mut params);
        }
        if let Some(v) = &self.title {
            params.push(("title", v.to_string()));
        }
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = self.sort_by {
            params.push(("sortBy", v.to_string()));
        }
        if let Some(v) = self.sort_direction {
            params.push(("sortDirection", v.to_string()));
        }
        params
    }
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
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct BuilderLeaderboardRequest {
    /// Time period to aggregate results over (default: DAY).
    pub time_period: Option<TimePeriod>,
    /// Maximum number of builders to return (0-50, default: 25).
    pub limit: Option<BuilderLeaderboardLimit>,
    /// Pagination offset (0-1000, default: 0).
    pub offset: Option<BuilderLeaderboardOffset>,
}

impl QueryParams for BuilderLeaderboardRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.time_period {
            params.push(("timePeriod", v.to_string()));
        }
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        params
    }
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
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct BuilderVolumeRequest {
    /// Time period to fetch daily records for (default: DAY).
    pub time_period: Option<TimePeriod>,
}

impl QueryParams for BuilderVolumeRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.time_period {
            params.push(("timePeriod", v.to_string()));
        }
        params
    }
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
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct TraderLeaderboardRequest {
    /// Market category filter (default: OVERALL).
    pub category: Option<LeaderboardCategory>,
    /// Time period for leaderboard results (default: DAY).
    pub time_period: Option<TimePeriod>,
    /// Ordering criteria (default: PNL).
    pub order_by: Option<LeaderboardOrderBy>,
    /// Maximum number of traders to return (1-50, default: 25).
    pub limit: Option<TraderLeaderboardLimit>,
    /// Pagination offset (0-1000, default: 0).
    pub offset: Option<TraderLeaderboardOffset>,
    /// Filter to a single user by address.
    pub user: Option<Address>,
    /// Filter to a single user by username.
    pub user_name: Option<String>,
}

impl QueryParams for TraderLeaderboardRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.category {
            params.push(("category", v.to_string()));
        }
        if let Some(v) = self.time_period {
            params.push(("timePeriod", v.to_string()));
        }
        if let Some(v) = self.order_by {
            params.push(("orderBy", v.to_string()));
        }
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = &self.user {
            params.push(("user", v.to_string()));
        }
        if let Some(v) = &self.user_name {
            params.push(("userName", v.clone()));
        }
        params
    }
}
