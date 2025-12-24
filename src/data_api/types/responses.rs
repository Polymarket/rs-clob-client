use serde::Deserialize;

use super::common::{ActivityType, Address, Hash64, Side};

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct HealthResponse {
    pub data: String,
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Position {
    pub proxy_wallet: Address,
    pub asset: String,
    pub condition_id: Hash64,
    pub size: f64,
    pub avg_price: f64,
    pub initial_value: f64,
    pub current_value: f64,
    pub cash_pnl: f64,
    pub percent_pnl: f64,
    pub total_bought: f64,
    pub realized_pnl: f64,
    pub percent_realized_pnl: f64,
    pub cur_price: f64,
    pub redeemable: bool,
    pub mergeable: bool,
    pub title: String,
    pub slug: String,
    pub icon: String,
    pub event_slug: String,
    pub outcome: String,
    pub outcome_index: i32,
    pub opposite_outcome: String,
    pub opposite_asset: String,
    pub end_date: String,
    pub negative_risk: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ClosedPosition {
    pub proxy_wallet: Address,
    pub asset: String,
    pub condition_id: Hash64,
    pub avg_price: f64,
    pub total_bought: f64,
    pub realized_pnl: f64,
    pub cur_price: f64,
    pub timestamp: i64,
    pub title: String,
    pub slug: String,
    pub icon: String,
    pub event_slug: String,
    pub outcome: String,
    pub outcome_index: i32,
    pub opposite_outcome: String,
    pub opposite_asset: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Trade {
    pub proxy_wallet: Address,
    pub side: Side,
    pub asset: String,
    pub condition_id: Hash64,
    pub size: f64,
    pub price: f64,
    pub timestamp: i64,
    pub title: String,
    pub slug: String,
    pub icon: String,
    pub event_slug: String,
    pub outcome: String,
    pub outcome_index: i32,
    pub name: Option<String>,
    pub pseudonym: Option<String>,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub profile_image_optimized: Option<String>,
    pub transaction_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Activity {
    pub proxy_wallet: Address,
    pub timestamp: i64,
    pub condition_id: Hash64,
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    pub size: f64,
    pub usdc_size: f64,
    pub transaction_hash: String,
    pub price: Option<f64>,
    pub asset: Option<String>,
    pub side: Option<Side>,
    pub outcome_index: Option<i32>,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub icon: Option<String>,
    pub event_slug: Option<String>,
    pub outcome: Option<String>,
    pub name: Option<String>,
    pub pseudonym: Option<String>,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub profile_image_optimized: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Holder {
    pub proxy_wallet: Address,
    pub bio: Option<String>,
    pub asset: String,
    pub pseudonym: Option<String>,
    pub amount: f64,
    pub display_username_public: Option<bool>,
    pub outcome_index: i32,
    pub name: Option<String>,
    pub profile_image: Option<String>,
    pub profile_image_optimized: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct MetaHolder {
    pub token: String,
    pub holders: Vec<Holder>,
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Traded {
    pub user: Address,
    pub traded: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Value {
    pub user: Address,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct OpenInterest {
    pub market: Hash64,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct MarketVolume {
    pub market: Hash64,
    pub value: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct LiveVolume {
    pub total: f64,
    pub markets: Vec<MarketVolume>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BuilderLeaderboardEntry {
    pub rank: String,
    pub builder: String,
    pub volume: f64,
    pub active_users: i32,
    pub verified: bool,
    pub builder_logo: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BuilderVolumeEntry {
    pub dt: String,
    pub builder: String,
    pub builder_logo: Option<String>,
    pub verified: bool,
    pub volume: f64,
    pub active_users: i32,
    pub rank: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TraderLeaderboardEntry {
    pub rank: String,
    pub proxy_wallet: Address,
    pub user_name: Option<String>,
    pub vol: f64,
    pub pnl: f64,
    pub profile_image: Option<String>,
    pub x_username: Option<String>,
    pub verified_badge: Option<bool>,
}
