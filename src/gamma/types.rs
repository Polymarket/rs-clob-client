//! Types for the Polymarket Gamma API.
//!
//! This module contains all types used by the Gamma API client, organized into:
//!
//! - **Common types**: Fundamental types like [`Address`], as well as enums
//!   for filtering and categorization.
//!
//! - **Request types**: Builder-pattern structs for each API endpoint
//!   (e.g., [`EventsRequest`], [`MarketsRequest`]).
//!
//! - **Response types**: Structs representing API responses
//!   (e.g., [`Event`], [`Market`], [`Tag`]).
//!
//! # Request Building
//!
//! All request types use the builder pattern via the [`bon`](https://docs.rs/bon) crate:
//!
//! ```
//! use polymarket_client_sdk::gamma::types::{EventsRequest, MarketsRequest};
//!
//! // Simple request with defaults
//! let events = EventsRequest::builder().build();
//!
//! // Request with filters
//! let markets = MarketsRequest::builder()
//!     .limit(10)
//!     .closed(false)
//!     .build();
//! ```

use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ser::{comma_separated, is_empty_vec, rfc3339};

// =============================================================================
// Request Types - Trait
// =============================================================================

/// Trait for converting request types to URL query strings.
///
/// This trait is automatically implemented for all types that implement [`Serialize`].
/// It uses [`serde_urlencoded`] to serialize the struct fields into a query string.
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::gamma::types::{EventsRequest, QueryParams};
///
/// let request = EventsRequest::builder()
///     .limit(10)
///     .active(true)
///     .build();
///
/// let query = request.query_string();
/// assert!(query.starts_with("?"));
/// assert!(query.contains("limit=10"));
/// assert!(query.contains("active=true"));
/// ```
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

// =============================================================================
// Request Types - Sports Endpoints
// =============================================================================

#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct TeamsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub league: Option<Vec<String>>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub name: Option<Vec<String>>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub abbreviation: Option<Vec<String>>,
}

// =============================================================================
// Request Types - Tags Endpoints
// =============================================================================

#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct TagsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_template: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_carousel: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct TagByIdRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_template: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct TagBySlugRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_template: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct RelatedTagsByIdRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omit_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RelatedTagsStatus>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct RelatedTagsBySlugRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omit_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RelatedTagsStatus>,
}

// =============================================================================
// Request Types - Events Endpoints
// =============================================================================

#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct EventsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub id: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_id: Option<i32>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub exclude_tag_id: Option<Vec<i32>>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub slug: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_tags: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cyom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_chat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_template: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidity_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidity_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "rfc3339")]
    pub start_date_min: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "rfc3339")]
    pub start_date_max: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "rfc3339")]
    pub end_date_min: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "rfc3339")]
    pub end_date_max: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct EventByIdRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_chat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_template: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct EventBySlugRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_chat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_template: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct EventTagsRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub id: u32,
}

// =============================================================================
// Request Types - Markets Endpoints
// =============================================================================

#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct MarketsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub id: Option<Vec<i32>>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub slug: Option<Vec<String>>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub clob_token_ids: Option<Vec<String>>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub condition_ids: Option<Vec<String>>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub market_maker_address: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidity_num_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidity_num_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_num_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_num_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "rfc3339")]
    pub start_date_min: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "rfc3339")]
    pub start_date_max: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "rfc3339")]
    pub end_date_min: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "rfc3339")]
    pub end_date_max: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_tags: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cyom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uma_resolution_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub sports_market_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewards_min_size: Option<f64>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub question_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tag: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct MarketByIdRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tag: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct MarketBySlugRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tag: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct MarketTagsRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub id: u32,
}

// =============================================================================
// Request Types - Series Endpoints
// =============================================================================

#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct SeriesListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub slug: Option<Vec<String>>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub categories_ids: Option<Vec<i32>>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub categories_labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_chat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct SeriesByIdRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_chat: Option<bool>,
}

// =============================================================================
// Request Types - Comments Endpoints
// =============================================================================

#[derive(Debug, Clone, Builder, Default, Serialize)]
#[non_exhaustive]
pub struct CommentsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_entity_type: Option<ParentEntityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_entity_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_positions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holders_only: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct CommentsByIdRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_positions: Option<bool>,
}

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct CommentsByUserAddressRequest {
    #[serde(skip_serializing)]
    #[builder(into)]
    pub user_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
}

// =============================================================================
// Request Types - Profiles Endpoints
// =============================================================================

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct PublicProfileRequest {
    #[builder(into)]
    pub address: String,
}

// =============================================================================
// Request Types - Search Endpoints
// =============================================================================

#[derive(Debug, Clone, Builder, Serialize)]
#[non_exhaustive]
pub struct SearchRequest {
    #[builder(into)]
    pub q: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_per_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub events_tag: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_closed_markets: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_tags: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_profiles: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<String>,
    #[serde(
        skip_serializing_if = "is_empty_vec",
        serialize_with = "comma_separated"
    )]
    pub exclude_tag_id: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimized: Option<bool>,
}

// =============================================================================
// Response Types - Common/Shared
// =============================================================================

/// Image optimization metadata.
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
    pub rel_id: Option<i64>,
    pub field: Option<String>,
    pub relname: Option<String>,
}

/// Pagination information.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Pagination {
    pub has_more: Option<bool>,
    pub total_results: Option<i64>,
}

/// Count response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Count {
    pub count: Option<i64>,
}

// =============================================================================
// Response Types - Health
// =============================================================================

/// Health check response.
pub type HealthResponse = String;

// =============================================================================
// Response Types - Sports
// =============================================================================

/// A sports team.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Team {
    pub id: i64,
    pub name: Option<String>,
    pub league: Option<String>,
    pub record: Option<String>,
    pub logo: Option<String>,
    pub abbreviation: Option<String>,
    pub alias: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Sports metadata information.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct SportsMetadata {
    pub sport: String,
    pub image: String,
    pub resolution: String,
    pub ordering: String,
    pub tags: String,
    pub series: String,
}

/// Sports market types response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SportsMarketTypesResponse {
    pub market_types: Vec<String>,
}

// =============================================================================
// Response Types - Tags
// =============================================================================

/// A tag for categorizing content.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Tag {
    pub id: String,
    pub label: Option<String>,
    pub slug: Option<String>,
    pub force_show: Option<bool>,
    pub published_at: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub force_hide: Option<bool>,
    pub is_carousel: Option<bool>,
}

/// A relationship between tags.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RelatedTag {
    pub id: String,
    #[serde(rename = "tagID")]
    pub tag_id: Option<i64>,
    #[serde(rename = "relatedTagID")]
    pub related_tag_id: Option<i64>,
    pub rank: Option<i64>,
}

// =============================================================================
// Response Types - Categories
// =============================================================================

/// A category for organizing content.
#[derive(Debug, Clone, PartialEq, Deserialize)]
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

// =============================================================================
// Response Types - Events
// =============================================================================

/// An event creator.
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
    pub liquidity: Option<f64>,
    pub volume: Option<f64>,
    pub open_interest: Option<f64>,
    pub sort_by: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub is_template: Option<bool>,
    pub template_variables: Option<String>,
    pub published_at: Option<String>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub comments_enabled: Option<bool>,
    pub competitive: Option<f64>,
    pub volume_24hr: Option<f64>,
    pub volume_1wk: Option<f64>,
    pub volume_1mo: Option<f64>,
    pub volume_1yr: Option<f64>,
    pub featured_image: Option<String>,
    pub disqus_thread: Option<String>,
    pub parent_event: Option<String>,
    pub enable_order_book: Option<bool>,
    pub liquidity_amm: Option<f64>,
    pub liquidity_clob: Option<f64>,
    pub neg_risk: Option<bool>,
    #[serde(rename = "negRiskMarketID")]
    pub neg_risk_market_id: Option<String>,
    pub neg_risk_fee_bips: Option<i64>,
    pub comment_count: Option<i64>,
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
    pub event_week: Option<i64>,
    pub series_slug: Option<String>,
    pub score: Option<String>,
    pub elapsed: Option<String>,
    pub period: Option<String>,
    pub live: Option<bool>,
    pub ended: Option<bool>,
    pub finished_timestamp: Option<DateTime<Utc>>,
    pub gmp_chart_mode: Option<String>,
    pub event_creators: Option<Vec<EventCreator>>,
    pub tweet_count: Option<i64>,
    pub chats: Option<Vec<Chat>>,
    pub featured_order: Option<i64>,
    pub estimate_value: Option<bool>,
    pub cant_estimate: Option<bool>,
    pub estimated_value: Option<String>,
    pub templates: Option<Vec<Template>>,
    pub spreads_main_line: Option<f64>,
    pub totals_main_line: Option<f64>,
    pub carousel_map: Option<String>,
    pub pending_deployment: Option<bool>,
    pub deploying: Option<bool>,
    pub deploying_timestamp: Option<DateTime<Utc>>,
    pub scheduled_deployment_timestamp: Option<DateTime<Utc>>,
    pub game_status: Option<String>,
}

/// Event tweet count response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EventTweetCount {
    pub tweet_count: Option<i64>,
}

/// Paginated events response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct EventsPagination {
    pub data: Option<Vec<Event>>,
    pub pagination: Option<Pagination>,
}

// =============================================================================
// Response Types - Markets
// =============================================================================

/// A prediction market.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Market {
    pub id: String,
    pub question: Option<String>,
    pub condition_id: Option<String>,
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
    pub market_maker_address: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
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
    pub market_group: Option<i64>,
    pub group_item_title: Option<String>,
    pub group_item_threshold: Option<String>,
    #[serde(rename = "questionID")]
    pub question_id: Option<String>,
    pub uma_end_date: Option<String>,
    pub enable_order_book: Option<bool>,
    pub order_price_min_tick_size: Option<f64>,
    pub order_min_size: Option<f64>,
    pub uma_resolution_status: Option<String>,
    pub curation_order: Option<i64>,
    pub volume_num: Option<f64>,
    pub liquidity_num: Option<f64>,
    pub end_date_iso: Option<String>,
    pub start_date_iso: Option<String>,
    pub uma_end_date_iso: Option<String>,
    pub has_reviewed_dates: Option<bool>,
    pub ready_for_cron: Option<bool>,
    pub comments_enabled: Option<bool>,
    pub volume_24hr: Option<f64>,
    pub volume_1wk: Option<f64>,
    pub volume_1mo: Option<f64>,
    pub volume_1yr: Option<f64>,
    pub game_start_time: Option<String>,
    pub seconds_delay: Option<i64>,
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
    pub volume_24hr_amm: Option<f64>,
    pub volume_1wk_amm: Option<f64>,
    pub volume_1mo_amm: Option<f64>,
    pub volume_1yr_amm: Option<f64>,
    pub volume_24hr_clob: Option<f64>,
    pub volume_1wk_clob: Option<f64>,
    pub volume_1mo_clob: Option<f64>,
    pub volume_1yr_clob: Option<f64>,
    pub volume_amm: Option<f64>,
    pub volume_clob: Option<f64>,
    pub liquidity_amm: Option<f64>,
    pub liquidity_clob: Option<f64>,
    pub maker_base_fee: Option<i64>,
    pub taker_base_fee: Option<i64>,
    pub custom_liveness: Option<i64>,
    pub accepting_orders: Option<bool>,
    pub notifications_enabled: Option<bool>,
    pub score: Option<i64>,
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
    pub competitive: Option<f64>,
    pub rewards_min_size: Option<f64>,
    pub rewards_max_spread: Option<f64>,
    pub spread: Option<f64>,
    pub automatically_resolved: Option<bool>,
    pub one_day_price_change: Option<f64>,
    pub one_hour_price_change: Option<f64>,
    pub one_week_price_change: Option<f64>,
    pub one_month_price_change: Option<f64>,
    pub one_year_price_change: Option<f64>,
    pub last_trade_price: Option<f64>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
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
    pub line: Option<f64>,
    pub uma_resolution_statuses: Option<String>,
    pub pending_deployment: Option<bool>,
    pub deploying: Option<bool>,
    pub deploying_timestamp: Option<DateTime<Utc>>,
    pub scheduled_deployment_timestamp: Option<DateTime<Utc>>,
    pub rfq_enabled: Option<bool>,
    pub event_start_time: Option<DateTime<Utc>>,
}

/// Market description response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct MarketDescription {
    pub description: Option<String>,
}

// =============================================================================
// Response Types - Series
// =============================================================================

/// A series of related events.
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
    pub volume_24hr: Option<f64>,
    pub volume: Option<f64>,
    pub liquidity: Option<f64>,
    pub start_date: Option<DateTime<Utc>>,
    #[serde(rename = "pythTokenID")]
    pub pyth_token_id: Option<String>,
    pub cg_asset_name: Option<String>,
    pub score: Option<i64>,
    pub events: Option<Vec<Event>>,
    pub collections: Option<Vec<Collection>>,
    pub categories: Option<Vec<Category>>,
    pub tags: Option<Vec<Tag>>,
    pub comment_count: Option<i64>,
    pub chats: Option<Vec<Chat>>,
}

/// A summary of a series with event dates and weeks.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SeriesSummary {
    pub id: String,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub event_dates: Option<Vec<String>>,
    pub event_weeks: Option<Vec<i64>>,
    pub earliest_open_week: Option<i64>,
    pub earliest_open_date: Option<String>,
}

// =============================================================================
// Response Types - Comments
// =============================================================================

/// A comment position.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CommentPosition {
    pub token_id: Option<String>,
    pub position_size: Option<String>,
}

/// A comment profile.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CommentProfile {
    pub name: Option<String>,
    pub pseudonym: Option<String>,
    pub display_username_public: Option<bool>,
    pub bio: Option<String>,
    pub is_mod: Option<bool>,
    pub is_creator: Option<bool>,
    pub proxy_wallet: Option<String>,
    pub base_address: Option<String>,
    pub profile_image: Option<String>,
    pub profile_image_optimized: Option<ImageOptimization>,
    pub positions: Option<Vec<CommentPosition>>,
}

/// A reaction to a comment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Reaction {
    pub id: String,
    #[serde(rename = "commentID")]
    pub comment_id: Option<i64>,
    pub reaction_type: Option<String>,
    pub icon: Option<String>,
    pub user_address: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub profile: Option<CommentProfile>,
}

/// A comment on an event, series, or market.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Comment {
    pub id: String,
    pub body: Option<String>,
    pub parent_entity_type: Option<String>,
    #[serde(rename = "parentEntityID")]
    pub parent_entity_id: Option<i64>,
    #[serde(rename = "parentCommentID")]
    pub parent_comment_id: Option<String>,
    pub user_address: Option<String>,
    pub reply_address: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub profile: Option<CommentProfile>,
    pub reactions: Option<Vec<Reaction>>,
    pub report_count: Option<i64>,
    pub reaction_count: Option<i64>,
}

// =============================================================================
// Response Types - Profiles
// =============================================================================

/// A user associated with a public profile.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct PublicProfileUser {
    pub id: Option<String>,
    pub creator: Option<bool>,
    #[serde(rename = "mod")]
    pub is_mod: Option<bool>,
}

/// Public profile response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PublicProfile {
    pub created_at: Option<DateTime<Utc>>,
    pub proxy_wallet: Option<String>,
    pub profile_image: Option<String>,
    pub display_username_public: Option<bool>,
    pub bio: Option<String>,
    pub pseudonym: Option<String>,
    pub name: Option<String>,
    pub users: Option<Vec<PublicProfileUser>>,
    pub x_username: Option<String>,
    pub verified_badge: Option<bool>,
}

/// Error response for public profile endpoint.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PublicProfileError {
    /// Error type classification.
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    /// Error message.
    pub error: Option<String>,
}

// =============================================================================
// Response Types - Search
// =============================================================================

/// A search tag result.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct SearchTag {
    pub id: Option<String>,
    pub label: Option<String>,
    pub slug: Option<String>,
    pub event_count: Option<i64>,
}

/// A profile in search results.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Profile {
    pub id: String,
    pub name: Option<String>,
    pub user: Option<i64>,
    pub referral: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
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
    pub proxy_wallet: Option<String>,
    pub profile_image_optimized: Option<ImageOptimization>,
    pub is_close_only: Option<bool>,
    pub is_cert_req: Option<bool>,
    pub cert_req_date: Option<DateTime<Utc>>,
}

/// Search results.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct SearchResults {
    pub events: Option<Vec<Event>>,
    pub tags: Option<Vec<SearchTag>>,
    pub profiles: Option<Vec<Profile>>,
    pub pagination: Option<Pagination>,
}

// =============================================================================
// Common Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[non_exhaustive]
pub enum RelatedTagsStatus {
    Active,
    Closed,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[non_exhaustive]
pub enum ParentEntityType {
    Event,
    Series,
    #[serde(rename = "market")]
    #[strum(serialize = "market")]
    Market,
}
