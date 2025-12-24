use bon::Builder;

use super::common::{
    ActivityLimit, ActivityOffset, ActivitySortBy, ActivityType, Address, BuilderLeaderboardLimit,
    BuilderLeaderboardOffset, ClosedPositionSortBy, ClosedPositionsLimit, ClosedPositionsOffset,
    EventId, Hash64, HoldersLimit, HoldersMinBalance, LeaderboardCategory, LeaderboardOrderBy,
    MarketFilter, PositionSortBy, PositionsLimit, PositionsOffset, Side, SortDirection, TimePeriod,
    Title, TradeFilter, TraderLeaderboardLimit, TraderLeaderboardOffset, TradesLimit, TradesOffset,
};

pub trait QueryParams {
    #[must_use]
    fn query_params(&self) -> Vec<(&'static str, String)>;
}

impl QueryParams for () {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}

#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct PositionsRequest {
    pub user: Address,
    pub filter: Option<MarketFilter>,
    pub size_threshold: Option<f64>,
    pub redeemable: Option<bool>,
    pub mergeable: Option<bool>,
    pub limit: Option<PositionsLimit>,
    pub offset: Option<PositionsOffset>,
    pub sort_by: Option<PositionSortBy>,
    pub sort_direction: Option<SortDirection>,
    pub title: Option<Title>,
}

impl QueryParams for PositionsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("user", self.user.to_string())];
        if let Some(f) = &self.filter {
            f.append_to_params(&mut params);
        }
        if let Some(v) = self.size_threshold {
            params.push(("sizeThreshold", v.to_string()));
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

#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct TradesRequest {
    pub user: Option<Address>,
    pub filter: Option<MarketFilter>,
    pub limit: Option<TradesLimit>,
    pub offset: Option<TradesOffset>,
    pub taker_only: Option<bool>,
    pub trade_filter: Option<TradeFilter>,
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
            params.push(("filterAmount", f.filter_amount.to_string()));
        }
        if let Some(v) = self.side {
            params.push(("side", v.to_string()));
        }
        params
    }
}

#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct ActivityRequest {
    pub user: Address,
    pub filter: Option<MarketFilter>,
    pub activity_types: Option<Vec<ActivityType>>,
    pub limit: Option<ActivityLimit>,
    pub offset: Option<ActivityOffset>,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub sort_by: Option<ActivitySortBy>,
    pub sort_direction: Option<SortDirection>,
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

#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct HoldersRequest {
    pub markets: Vec<Hash64>,
    pub limit: Option<HoldersLimit>,
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

#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct TradedRequest {
    pub user: Address,
}

impl QueryParams for TradedRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![("user", self.user.to_string())]
    }
}

#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct ValueRequest {
    pub user: Address,
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

#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct OpenInterestRequest {
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

#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct LiveVolumeRequest {
    pub id: EventId,
}

impl QueryParams for LiveVolumeRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![("id", self.id.to_string())]
    }
}

#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct ClosedPositionsRequest {
    pub user: Address,
    pub filter: Option<MarketFilter>,
    pub title: Option<Title>,
    pub limit: Option<ClosedPositionsLimit>,
    pub offset: Option<ClosedPositionsOffset>,
    pub sort_by: Option<ClosedPositionSortBy>,
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

#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct BuilderLeaderboardRequest {
    pub time_period: Option<TimePeriod>,
    pub limit: Option<BuilderLeaderboardLimit>,
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

#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct BuilderVolumeRequest {
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

#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct TraderLeaderboardRequest {
    pub category: Option<LeaderboardCategory>,
    pub time_period: Option<TimePeriod>,
    pub order_by: Option<LeaderboardOrderBy>,
    pub limit: Option<TraderLeaderboardLimit>,
    pub offset: Option<TraderLeaderboardOffset>,
    pub user: Option<Address>,
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
