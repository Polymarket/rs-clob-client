#![allow(
    clippy::module_name_repetitions,
    reason = "Response suffix is intentional for clarity"
)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::NoneAsEmptyString;
use serde_with::serde_as;

use crate::serde_helpers::StringFromAny;
use crate::types::{Address, B256, Decimal};

/// Image optimization metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImageOptimization {
    pub id: Option<String>,
    pub image_url_source: Option<String>,
    pub image_url_optimized: Option<String>,
    pub image_size_kb_source: Option<f64>,
    pub image_size_kb_optimized: Option<f64>,
    pub image_optimized_complete: Option<bool>,
    pub image_optimized_last_updated: Option<String>,
    #[serde(rename = "relID")]
    pub rel_id: Option<String>,
    pub field: Option<String>,
    pub relname: Option<String>,
}

/// Pagination information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Pagination {
    pub has_more: Option<bool>,
    pub total_results: Option<i32>,
}

/// Health check response.
pub type HealthResponse = String;

/// A sports team.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Team {
    pub id: i32,
    pub name: Option<String>,
    pub league: Option<String>,
    pub record: Option<String>,
    pub logo: Option<String>,
    pub abbreviation: Option<String>,
    pub alias: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub color: Option<String>,
    pub provider_id: Option<i32>,
}

/// Sports metadata information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SportsMetadata {
    pub id: Option<i32>,
    pub sport: String,
    pub image: String,
    pub resolution: String,
    pub ordering: String,
    pub tags: String,
    pub series: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Sports market types response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SportsMarketTypesResponse {
    pub market_types: Vec<String>,
}

/// A tag for categorizing content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Tag {
    pub id: String,
    pub label: Option<String>,
    pub slug: Option<String>,
    pub force_show: Option<bool>,
    pub published_at: Option<String>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub force_hide: Option<bool>,
    pub is_carousel: Option<bool>,
    pub requires_translation: Option<bool>,
}

/// A relationship between tags.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RelatedTag {
    #[serde_as(as = "StringFromAny")]
    pub id: String,
    #[serde_as(as = "Option<StringFromAny>")]
    #[serde(rename = "tagID")]
    pub tag_id: Option<String>,
    #[serde_as(as = "Option<StringFromAny>")]
    #[serde(rename = "relatedTagID")]
    pub related_tag_id: Option<String>,
    pub rank: Option<i32>,
}

/// A category for organizing content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Category {
    pub id: String,
    pub label: Option<String>,
    pub parent_category: Option<String>,
    pub slug: Option<String>,
    pub published_at: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// An event creator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EventCreator {
    pub id: String,
    pub creator_name: Option<String>,
    pub creator_handle: Option<String>,
    pub creator_url: Option<String>,
    pub creator_image: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A chat/live stream associated with an event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Chat {
    pub id: String,
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
    pub channel_image: Option<String>,
    pub live: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// A template for creating events/markets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Template {
    pub id: String,
    pub event_title: Option<String>,
    pub event_slug: Option<String>,
    pub event_image: Option<String>,
    pub market_title: Option<String>,
    pub description: Option<String>,
    pub resolution_source: Option<String>,
    pub neg_risk: Option<bool>,
    pub sort_by: Option<String>,
    pub show_market_images: Option<bool>,
    pub series_slug: Option<String>,
    pub outcomes: Option<String>,
}

/// A collection of events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Collection {
    pub id: String,
    pub ticker: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub collection_type: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub header_image: Option<String>,
    pub layout: Option<String>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub new: Option<bool>,
    pub featured: Option<bool>,
    pub restricted: Option<bool>,
    pub is_template: Option<bool>,
    pub template_variables: Option<String>,
    pub published_at: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub comments_enabled: Option<bool>,
    pub image_optimized: Option<ImageOptimization>,
    pub icon_optimized: Option<ImageOptimization>,
    pub header_image_optimized: Option<ImageOptimization>,
}

/// A prediction market event.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Event {
    pub id: String,
    pub ticker: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub resolution_source: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub creation_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub new: Option<bool>,
    pub featured: Option<bool>,
    pub restricted: Option<bool>,
    pub liquidity: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub open_interest: Option<Decimal>,
    pub sort_by: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub is_template: Option<bool>,
    pub template_variables: Option<String>,
    #[serde(alias = "published_at")]
    pub published_at: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub comments_enabled: Option<bool>,
    pub competitive: Option<Decimal>,
    pub volume_24hr: Option<Decimal>,
    pub volume_1wk: Option<Decimal>,
    pub volume_1mo: Option<Decimal>,
    pub volume_1yr: Option<Decimal>,
    pub featured_image: Option<String>,
    pub disqus_thread: Option<String>,
    pub parent_event: Option<String>,
    #[serde_as(as = "Option<StringFromAny>")]
    pub parent_event_id: Option<String>,
    pub sportsradar_match_id: Option<String>,
    #[serde_as(as = "Option<StringFromAny>")]
    pub turn_provider_id: Option<String>,
    pub enable_order_book: Option<bool>,
    pub liquidity_amm: Option<Decimal>,
    pub liquidity_clob: Option<Decimal>,
    pub neg_risk: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default, rename = "negRiskMarketID")]
    pub neg_risk_market_id: Option<B256>,
    pub neg_risk_fee_bips: Option<i32>,
    pub comment_count: Option<i32>,
    pub image_optimized: Option<ImageOptimization>,
    pub icon_optimized: Option<ImageOptimization>,
    pub featured_image_optimized: Option<ImageOptimization>,
    pub sub_events: Option<Vec<String>>,
    pub markets: Option<Vec<Market>>,
    pub series: Option<Vec<Series>>,
    pub categories: Option<Vec<Category>>,
    pub collections: Option<Vec<Collection>>,
    pub tags: Option<Vec<Tag>>,
    pub cyom: Option<bool>,
    pub closed_time: Option<DateTime<Utc>>,
    pub show_all_outcomes: Option<bool>,
    pub show_market_images: Option<bool>,
    pub automatically_resolved: Option<bool>,
    pub enable_neg_risk: Option<bool>,
    pub automatically_active: Option<bool>,
    pub event_date: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub event_week: Option<i32>,
    pub series_slug: Option<String>,
    pub score: Option<String>,
    pub elapsed: Option<String>,
    pub period: Option<String>,
    pub live: Option<bool>,
    pub ended: Option<bool>,
    pub finished_timestamp: Option<DateTime<Utc>>,
    pub gmp_chart_mode: Option<String>,
    pub event_creators: Option<Vec<EventCreator>>,
    pub tweet_count: Option<i32>,
    pub chats: Option<Vec<Chat>>,
    pub featured_order: Option<i32>,
    pub estimate_value: Option<bool>,
    pub cant_estimate: Option<bool>,
    pub estimated_value: Option<String>,
    pub templates: Option<Vec<Template>>,
    pub spreads_main_line: Option<Decimal>,
    pub totals_main_line: Option<Decimal>,
    pub carousel_map: Option<String>,
    pub pending_deployment: Option<bool>,
    pub deploying: Option<bool>,
    pub deploying_timestamp: Option<DateTime<Utc>>,
    pub scheduled_deployment_timestamp: Option<DateTime<Utc>>,
    pub game_status: Option<String>,
    pub requires_translation: Option<bool>,
    pub neg_risk_augmented: Option<bool>,
    pub game_id: Option<i64>,
    pub election_type: Option<String>,
    pub country_name: Option<String>,
    pub color: Option<String>,
    pub cumulative_markets: Option<bool>,
}

/// A prediction market.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Market {
    pub id: String,
    pub question: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub condition_id: Option<B256>,
    pub slug: Option<String>,
    pub twitter_card_image: Option<String>,
    pub resolution_source: Option<String>,
    pub end_date: Option<DateTime<Utc>>,
    pub category: Option<String>,
    pub amm_type: Option<String>,
    pub liquidity: Option<String>,
    pub sponsor_name: Option<String>,
    pub sponsor_image: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub x_axis_value: Option<String>,
    pub y_axis_value: Option<String>,
    pub denomination_token: Option<String>,
    pub fee: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub lower_bound: Option<String>,
    pub upper_bound: Option<String>,
    pub description: Option<String>,
    pub outcomes: Option<String>,
    pub outcome_prices: Option<String>,
    pub volume: Option<String>,
    pub active: Option<bool>,
    pub market_type: Option<String>,
    pub format_type: Option<String>,
    pub lower_bound_date: Option<String>,
    pub upper_bound_date: Option<String>,
    pub closed: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub market_maker_address: Option<Address>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub closed_time: Option<String>,
    pub wide_format: Option<bool>,
    pub new: Option<bool>,
    pub mailchimp_tag: Option<String>,
    pub featured: Option<bool>,
    pub archived: Option<bool>,
    pub resolved_by: Option<String>,
    pub restricted: Option<bool>,
    pub market_group: Option<i32>,
    pub group_item_title: Option<String>,
    pub group_item_threshold: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default, rename = "questionID")]
    pub question_id: Option<B256>,
    pub uma_end_date: Option<String>,
    pub enable_order_book: Option<bool>,
    pub order_price_min_tick_size: Option<Decimal>,
    pub order_min_size: Option<Decimal>,
    pub uma_resolution_status: Option<String>,
    pub curation_order: Option<i32>,
    pub volume_num: Option<Decimal>,
    pub liquidity_num: Option<Decimal>,
    pub end_date_iso: Option<String>,
    pub start_date_iso: Option<String>,
    pub uma_end_date_iso: Option<String>,
    pub has_reviewed_dates: Option<bool>,
    pub ready_for_cron: Option<bool>,
    pub comments_enabled: Option<bool>,
    pub volume_24hr: Option<Decimal>,
    pub volume_1wk: Option<Decimal>,
    pub volume_1mo: Option<Decimal>,
    pub volume_1yr: Option<Decimal>,
    pub game_start_time: Option<String>,
    pub seconds_delay: Option<i32>,
    pub clob_token_ids: Option<String>,
    pub disqus_thread: Option<String>,
    pub short_outcomes: Option<String>,
    #[serde(rename = "teamAID")]
    pub team_a_id: Option<String>,
    #[serde(rename = "teamBID")]
    pub team_b_id: Option<String>,
    pub uma_bond: Option<String>,
    pub uma_reward: Option<String>,
    pub fpmm_live: Option<bool>,
    pub volume_24hr_amm: Option<Decimal>,
    pub volume_1wk_amm: Option<Decimal>,
    pub volume_1mo_amm: Option<Decimal>,
    pub volume_1yr_amm: Option<Decimal>,
    pub volume_24hr_clob: Option<Decimal>,
    pub volume_1wk_clob: Option<Decimal>,
    pub volume_1mo_clob: Option<Decimal>,
    pub volume_1yr_clob: Option<Decimal>,
    pub volume_amm: Option<Decimal>,
    pub volume_clob: Option<Decimal>,
    pub liquidity_amm: Option<Decimal>,
    pub liquidity_clob: Option<Decimal>,
    pub maker_base_fee: Option<i32>,
    pub taker_base_fee: Option<i32>,
    pub maker_rebates_fee_share_bps: Option<i32>,
    pub custom_liveness: Option<i32>,
    pub accepting_orders: Option<bool>,
    pub notifications_enabled: Option<bool>,
    pub score: Option<i32>,
    pub image_optimized: Option<ImageOptimization>,
    pub icon_optimized: Option<ImageOptimization>,
    pub events: Option<Vec<Event>>,
    pub categories: Option<Vec<Category>>,
    pub tags: Option<Vec<Tag>>,
    pub creator: Option<String>,
    pub ready: Option<bool>,
    pub funded: Option<bool>,
    pub past_slugs: Option<String>,
    pub ready_timestamp: Option<DateTime<Utc>>,
    pub funded_timestamp: Option<DateTime<Utc>>,
    pub accepting_orders_timestamp: Option<DateTime<Utc>>,
    pub competitive: Option<Decimal>,
    pub rewards_min_size: Option<Decimal>,
    pub rewards_max_spread: Option<Decimal>,
    pub spread: Option<Decimal>,
    pub automatically_resolved: Option<bool>,
    pub one_day_price_change: Option<Decimal>,
    pub one_hour_price_change: Option<Decimal>,
    pub one_week_price_change: Option<Decimal>,
    pub one_month_price_change: Option<Decimal>,
    pub one_year_price_change: Option<Decimal>,
    pub last_trade_price: Option<Decimal>,
    pub best_bid: Option<Decimal>,
    pub best_ask: Option<Decimal>,
    pub automatically_active: Option<bool>,
    pub clear_book_on_start: Option<bool>,
    pub chart_color: Option<String>,
    pub series_color: Option<String>,
    pub show_gmp_series: Option<bool>,
    pub show_gmp_outcome: Option<bool>,
    pub manual_activation: Option<bool>,
    pub neg_risk_other: Option<bool>,
    pub game_id: Option<String>,
    pub group_item_range: Option<String>,
    pub sports_market_type: Option<String>,
    pub line: Option<Decimal>,
    pub uma_resolution_statuses: Option<String>,
    pub pending_deployment: Option<bool>,
    pub deploying: Option<bool>,
    pub deploying_timestamp: Option<DateTime<Utc>>,
    pub scheduled_deployment_timestamp: Option<DateTime<Utc>>,
    pub rfq_enabled: Option<bool>,
    pub event_start_time: Option<DateTime<Utc>>,
    #[serde(alias = "submitted_by")]
    pub submitted_by: Option<String>,
    pub requires_translation: Option<bool>,
    pub pager_duty_notification_enabled: Option<bool>,
    pub approved: Option<bool>,
    pub cyom: Option<bool>,
    pub fees_enabled: Option<bool>,
    pub holding_rewards_enabled: Option<bool>,
    pub neg_risk: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default, rename = "negRiskRequestID")]
    pub neg_risk_request_id: Option<B256>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default, rename = "negRiskMarketID")]
    pub neg_risk_market_id: Option<B256>,
    pub sent_discord: Option<bool>,
    pub twitter_card_last_refreshed: Option<String>,
    pub twitter_card_location: Option<String>,
    pub twitter_card_last_validated: Option<String>,
    pub clob_rewards: Option<Vec<ClobReward>>,
}

/// CLOB rewards configuration for a market.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ClobReward {
    pub id: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub asset_address: Option<Address>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub condition_id: Option<B256>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub rewards_amount: Option<Decimal>,
    pub rewards_daily_rate: Option<Decimal>,
}

/// A series of related events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Series {
    pub id: String,
    pub ticker: Option<String>,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub series_type: Option<String>,
    pub recurrence: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    pub layout: Option<String>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub new: Option<bool>,
    pub featured: Option<bool>,
    pub restricted: Option<bool>,
    pub is_template: Option<bool>,
    pub template_variables: Option<bool>,
    pub published_at: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub comments_enabled: Option<bool>,
    pub competitive: Option<String>,
    pub volume_24hr: Option<Decimal>,
    pub volume: Option<Decimal>,
    pub liquidity: Option<Decimal>,
    pub start_date: Option<DateTime<Utc>>,
    #[serde(rename = "pythTokenID")]
    pub pyth_token_id: Option<String>,
    pub cg_asset_name: Option<String>,
    pub score: Option<i32>,
    pub events: Option<Vec<Event>>,
    pub collections: Option<Vec<Collection>>,
    pub categories: Option<Vec<Category>>,
    pub tags: Option<Vec<Tag>>,
    pub comment_count: Option<i32>,
    pub chats: Option<Vec<Chat>>,
    pub requires_translation: Option<bool>,
}

/// A comment position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CommentPosition {
    pub token_id: Option<String>,
    pub position_size: Option<String>,
}

/// A comment profile.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CommentProfile {
    pub name: Option<String>,
    pub pseudonym: Option<String>,
    pub display_username_public: Option<bool>,
    pub bio: Option<String>,
    pub is_mod: Option<bool>,
    pub is_creator: Option<bool>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub proxy_wallet: Option<Address>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub base_address: Option<Address>,
    pub profile_image: Option<String>,
    pub profile_image_optimized: Option<ImageOptimization>,
    pub positions: Option<Vec<CommentPosition>>,
}

/// A reaction to a comment.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Reaction {
    pub id: String,
    #[serde(rename = "commentID")]
    pub comment_id: Option<i32>,
    pub reaction_type: Option<String>,
    pub icon: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub user_address: Option<Address>,
    pub created_at: Option<DateTime<Utc>>,
    pub profile: Option<CommentProfile>,
}

/// A comment on an event, series, or market.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Comment {
    pub id: String,
    pub body: Option<String>,
    pub parent_entity_type: Option<String>,
    #[serde(rename = "parentEntityID")]
    pub parent_entity_id: Option<i32>,
    #[serde(rename = "parentCommentID")]
    pub parent_comment_id: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub user_address: Option<Address>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub reply_address: Option<Address>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub profile: Option<CommentProfile>,
    pub reactions: Option<Vec<Reaction>>,
    pub report_count: Option<i32>,
    pub reaction_count: Option<i32>,
}

/// A user associated with a public profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PublicProfileUser {
    pub id: Option<String>,
    pub creator: Option<bool>,
    #[serde(rename = "mod")]
    pub is_mod: Option<bool>,
}

/// Public profile response.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PublicProfile {
    pub created_at: Option<DateTime<Utc>>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub proxy_wallet: Option<Address>,
    pub profile_image: Option<String>,
    pub display_username_public: Option<bool>,
    pub bio: Option<String>,
    pub pseudonym: Option<String>,
    pub name: Option<String>,
    pub users: Option<Vec<PublicProfileUser>>,
    pub x_username: Option<String>,
    pub verified_badge: Option<bool>,
}

/// A search tag result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SearchTag {
    pub id: Option<String>,
    pub label: Option<String>,
    pub slug: Option<String>,
    pub event_count: Option<i32>,
}

/// A profile in search results.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Profile {
    pub id: String,
    pub name: Option<String>,
    pub user: Option<i32>,
    pub referral: Option<String>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
    pub wallet_activated: Option<bool>,
    pub pseudonym: Option<String>,
    pub display_username_public: Option<bool>,
    pub profile_image: Option<String>,
    pub bio: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(default)]
    pub proxy_wallet: Option<Address>,
    pub profile_image_optimized: Option<ImageOptimization>,
    pub is_close_only: Option<bool>,
    pub is_cert_req: Option<bool>,
    pub cert_req_date: Option<DateTime<Utc>>,
}

/// Search results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SearchResults {
    pub events: Option<Vec<Event>>,
    pub tags: Option<Vec<SearchTag>>,
    pub profiles: Option<Vec<Profile>>,
    pub pagination: Option<Pagination>,
}
