//! Request types for the Polymarket Gamma API.
//!
//! This module contains builder-pattern request types for all Gamma API endpoints.
//! Each request type corresponds to an API endpoint and includes all optional
//! query parameters documented in the `OpenAPI` specification.

use bon::Builder;
use chrono::{DateTime, Utc};

use super::common::{Address, ParentEntityType, RelatedTagsStatus, join_array};

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

// =============================================================================
// Sports Endpoints
// =============================================================================

/// Request parameters for the `/teams` endpoint.
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct TeamsRequest {
    /// Maximum number of teams to return.
    pub limit: Option<u32>,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Comma-separated list of fields to order by.
    pub order: Option<String>,
    /// Sort in ascending order.
    pub ascending: Option<bool>,
    /// Filter by league names.
    pub league: Option<Vec<String>>,
    /// Filter by team names.
    pub name: Option<Vec<String>>,
    /// Filter by team abbreviations.
    pub abbreviation: Option<Vec<String>>,
}

impl QueryParams for TeamsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = &self.order {
            params.push(("order", v.clone()));
        }
        if let Some(v) = self.ascending {
            params.push(("ascending", v.to_string()));
        }
        if let Some(v) = &self.league {
            if !v.is_empty() {
                params.push(("league", join_array(v)));
            }
        }
        if let Some(v) = &self.name {
            if !v.is_empty() {
                params.push(("name", join_array(v)));
            }
        }
        if let Some(v) = &self.abbreviation {
            if !v.is_empty() {
                params.push(("abbreviation", join_array(v)));
            }
        }
        params
    }
}

// =============================================================================
// Tags Endpoints
// =============================================================================

/// Request parameters for the `/tags` endpoint.
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct TagsRequest {
    /// Maximum number of tags to return.
    pub limit: Option<u64>,
    /// Pagination offset.
    pub offset: Option<u64>,
    /// Comma-separated list of fields to order by.
    pub order: Option<String>,
    /// Sort in ascending order.
    pub ascending: Option<bool>,
    /// Include template information.
    pub include_template: Option<bool>,
    /// Filter to carousel tags only.
    pub is_carousel: Option<bool>,
}

impl QueryParams for TagsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = &self.order {
            params.push(("order", v.clone()));
        }
        if let Some(v) = self.ascending {
            params.push(("ascending", v.to_string()));
        }
        if let Some(v) = self.include_template {
            params.push(("include_template", v.to_string()));
        }
        if let Some(v) = self.is_carousel {
            params.push(("is_carousel", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/tags/{id}` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct TagByIdRequest {
    /// Tag ID (path parameter).
    #[builder(into)]
    pub id: u32,
    /// Include template information.
    pub include_template: Option<bool>,
}

impl QueryParams for TagByIdRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.include_template {
            params.push(("include_template", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/tags/slug/{slug}` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct TagBySlugRequest {
    /// Tag slug (path parameter).
    #[builder(into)]
    pub slug: String,
    /// Include template information.
    pub include_template: Option<bool>,
}

impl QueryParams for TagBySlugRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.include_template {
            params.push(("include_template", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/tags/{id}/related-tags` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct RelatedTagsByIdRequest {
    /// Tag ID (path parameter).
    #[builder(into)]
    pub id: u64,
    /// Omit tags with no related markets.
    pub omit_empty: Option<bool>,
    /// Filter by market status.
    pub status: Option<RelatedTagsStatus>,
}

impl QueryParams for RelatedTagsByIdRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.omit_empty {
            params.push(("omit_empty", v.to_string()));
        }
        if let Some(v) = self.status {
            params.push(("status", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/tags/slug/{slug}/related-tags` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct RelatedTagsBySlugRequest {
    /// Tag slug (path parameter).
    #[builder(into)]
    pub slug: String,
    /// Omit tags with no related markets.
    pub omit_empty: Option<bool>,
    /// Filter by market status.
    pub status: Option<RelatedTagsStatus>,
}

impl QueryParams for RelatedTagsBySlugRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.omit_empty {
            params.push(("omit_empty", v.to_string()));
        }
        if let Some(v) = self.status {
            params.push(("status", v.to_string()));
        }
        params
    }
}

// =============================================================================
// Events Endpoints
// =============================================================================

/// Request parameters for the `/events` endpoint.
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct EventsRequest {
    /// Maximum number of events to return.
    pub limit: Option<u32>,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Comma-separated list of fields to order by.
    pub order: Option<String>,
    /// Sort in ascending order.
    pub ascending: Option<bool>,
    /// Filter by event IDs.
    pub id: Option<Vec<i32>>,
    /// Filter by tag ID.
    pub tag_id: Option<i32>,
    /// Exclude events with these tag IDs.
    pub exclude_tag_id: Option<Vec<i32>>,
    /// Filter by event slugs.
    pub slug: Option<Vec<String>>,
    /// Filter by tag slug.
    pub tag_slug: Option<String>,
    /// Include related tags.
    pub related_tags: Option<bool>,
    /// Filter by active status.
    pub active: Option<bool>,
    /// Filter by archived status.
    pub archived: Option<bool>,
    /// Filter by featured status.
    pub featured: Option<bool>,
    /// Filter CYOM (Create Your Own Market) events.
    pub cyom: Option<bool>,
    /// Include chat information.
    pub include_chat: Option<bool>,
    /// Include template information.
    pub include_template: Option<bool>,
    /// Filter by recurrence pattern.
    pub recurrence: Option<String>,
    /// Filter by closed status.
    pub closed: Option<bool>,
    /// Minimum liquidity filter.
    pub liquidity_min: Option<f64>,
    /// Maximum liquidity filter.
    pub liquidity_max: Option<f64>,
    /// Minimum volume filter.
    pub volume_min: Option<f64>,
    /// Maximum volume filter.
    pub volume_max: Option<f64>,
    /// Minimum start date filter.
    pub start_date_min: Option<DateTime<Utc>>,
    /// Maximum start date filter.
    pub start_date_max: Option<DateTime<Utc>>,
    /// Minimum end date filter.
    pub end_date_min: Option<DateTime<Utc>>,
    /// Maximum end date filter.
    pub end_date_max: Option<DateTime<Utc>>,
}

impl QueryParams for EventsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = &self.order {
            params.push(("order", v.clone()));
        }
        if let Some(v) = self.ascending {
            params.push(("ascending", v.to_string()));
        }
        if let Some(v) = &self.id {
            if !v.is_empty() {
                params.push(("id", join_array(v)));
            }
        }
        if let Some(v) = self.tag_id {
            params.push(("tag_id", v.to_string()));
        }
        if let Some(v) = &self.exclude_tag_id {
            if !v.is_empty() {
                params.push(("exclude_tag_id", join_array(v)));
            }
        }
        if let Some(v) = &self.slug {
            if !v.is_empty() {
                params.push(("slug", join_array(v)));
            }
        }
        if let Some(v) = &self.tag_slug {
            params.push(("tag_slug", v.clone()));
        }
        if let Some(v) = self.related_tags {
            params.push(("related_tags", v.to_string()));
        }
        if let Some(v) = self.active {
            params.push(("active", v.to_string()));
        }
        if let Some(v) = self.archived {
            params.push(("archived", v.to_string()));
        }
        if let Some(v) = self.featured {
            params.push(("featured", v.to_string()));
        }
        if let Some(v) = self.cyom {
            params.push(("cyom", v.to_string()));
        }
        if let Some(v) = self.include_chat {
            params.push(("include_chat", v.to_string()));
        }
        if let Some(v) = self.include_template {
            params.push(("include_template", v.to_string()));
        }
        if let Some(v) = &self.recurrence {
            params.push(("recurrence", v.clone()));
        }
        if let Some(v) = self.closed {
            params.push(("closed", v.to_string()));
        }
        if let Some(v) = self.liquidity_min {
            params.push(("liquidity_min", v.to_string()));
        }
        if let Some(v) = self.liquidity_max {
            params.push(("liquidity_max", v.to_string()));
        }
        if let Some(v) = self.volume_min {
            params.push(("volume_min", v.to_string()));
        }
        if let Some(v) = self.volume_max {
            params.push(("volume_max", v.to_string()));
        }
        if let Some(v) = self.start_date_min {
            params.push(("start_date_min", v.to_rfc3339()));
        }
        if let Some(v) = self.start_date_max {
            params.push(("start_date_max", v.to_rfc3339()));
        }
        if let Some(v) = self.end_date_min {
            params.push(("end_date_min", v.to_rfc3339()));
        }
        if let Some(v) = self.end_date_max {
            params.push(("end_date_max", v.to_rfc3339()));
        }
        params
    }
}

/// Request parameters for the `/events/{id}` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct EventByIdRequest {
    /// Event ID (path parameter).
    #[builder(into)]
    pub id: String,
    /// Include chat information.
    pub include_chat: Option<bool>,
    /// Include template information.
    pub include_template: Option<bool>,
}

impl QueryParams for EventByIdRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.include_chat {
            params.push(("include_chat", v.to_string()));
        }
        if let Some(v) = self.include_template {
            params.push(("include_template", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/events/slug/{slug}` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct EventBySlugRequest {
    /// Event slug (path parameter).
    #[builder(into)]
    pub slug: String,
    /// Include chat information.
    pub include_chat: Option<bool>,
    /// Include template information.
    pub include_template: Option<bool>,
}

impl QueryParams for EventBySlugRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.include_chat {
            params.push(("include_chat", v.to_string()));
        }
        if let Some(v) = self.include_template {
            params.push(("include_template", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/events/{id}/tags` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct EventTagsRequest {
    /// Event ID (path parameter).
    #[builder(into)]
    pub id: u32,
}

impl QueryParams for EventTagsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}

// =============================================================================
// Markets Endpoints
// =============================================================================

/// Request parameters for the `/markets` endpoint.
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct MarketsRequest {
    /// Maximum number of markets to return.
    pub limit: Option<u32>,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Comma-separated list of fields to order by.
    pub order: Option<String>,
    /// Sort in ascending order.
    pub ascending: Option<bool>,
    /// Filter by market IDs.
    pub id: Option<Vec<i32>>,
    /// Filter by market slugs.
    pub slug: Option<Vec<String>>,
    /// Filter by CLOB token IDs.
    pub clob_token_ids: Option<Vec<String>>,
    /// Filter by condition IDs.
    pub condition_ids: Option<Vec<String>>,
    /// Filter by market maker addresses.
    pub market_maker_address: Option<Vec<String>>,
    /// Minimum liquidity filter.
    pub liquidity_num_min: Option<f64>,
    /// Maximum liquidity filter.
    pub liquidity_num_max: Option<f64>,
    /// Minimum volume filter.
    pub volume_num_min: Option<f64>,
    /// Maximum volume filter.
    pub volume_num_max: Option<f64>,
    /// Minimum start date filter.
    pub start_date_min: Option<DateTime<Utc>>,
    /// Maximum start date filter.
    pub start_date_max: Option<DateTime<Utc>>,
    /// Minimum end date filter.
    pub end_date_min: Option<DateTime<Utc>>,
    /// Maximum end date filter.
    pub end_date_max: Option<DateTime<Utc>>,
    /// Filter by tag ID.
    pub tag_id: Option<i32>,
    /// Include related tags.
    pub related_tags: Option<bool>,
    /// Filter CYOM (Create Your Own Market) markets.
    pub cyom: Option<bool>,
    /// Filter by UMA resolution status.
    pub uma_resolution_status: Option<String>,
    /// Filter by game ID.
    pub game_id: Option<String>,
    /// Filter by sports market types.
    pub sports_market_types: Option<Vec<String>>,
    /// Minimum rewards size filter.
    pub rewards_min_size: Option<f64>,
    /// Filter by question IDs.
    pub question_ids: Option<Vec<String>>,
    /// Include tag information.
    pub include_tag: Option<bool>,
    /// Filter by closed status.
    pub closed: Option<bool>,
}

impl QueryParams for MarketsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = &self.order {
            params.push(("order", v.clone()));
        }
        if let Some(v) = self.ascending {
            params.push(("ascending", v.to_string()));
        }
        if let Some(v) = &self.id {
            if !v.is_empty() {
                params.push(("id", join_array(v)));
            }
        }
        if let Some(v) = &self.slug {
            if !v.is_empty() {
                params.push(("slug", join_array(v)));
            }
        }
        if let Some(v) = &self.clob_token_ids {
            if !v.is_empty() {
                params.push(("clob_token_ids", join_array(v)));
            }
        }
        if let Some(v) = &self.condition_ids {
            if !v.is_empty() {
                params.push(("condition_ids", join_array(v)));
            }
        }
        if let Some(v) = &self.market_maker_address {
            if !v.is_empty() {
                params.push(("market_maker_address", join_array(v)));
            }
        }
        if let Some(v) = self.liquidity_num_min {
            params.push(("liquidity_num_min", v.to_string()));
        }
        if let Some(v) = self.liquidity_num_max {
            params.push(("liquidity_num_max", v.to_string()));
        }
        if let Some(v) = self.volume_num_min {
            params.push(("volume_num_min", v.to_string()));
        }
        if let Some(v) = self.volume_num_max {
            params.push(("volume_num_max", v.to_string()));
        }
        if let Some(v) = self.start_date_min {
            params.push(("start_date_min", v.to_rfc3339()));
        }
        if let Some(v) = self.start_date_max {
            params.push(("start_date_max", v.to_rfc3339()));
        }
        if let Some(v) = self.end_date_min {
            params.push(("end_date_min", v.to_rfc3339()));
        }
        if let Some(v) = self.end_date_max {
            params.push(("end_date_max", v.to_rfc3339()));
        }
        if let Some(v) = self.tag_id {
            params.push(("tag_id", v.to_string()));
        }
        if let Some(v) = self.related_tags {
            params.push(("related_tags", v.to_string()));
        }
        if let Some(v) = self.cyom {
            params.push(("cyom", v.to_string()));
        }
        if let Some(v) = &self.uma_resolution_status {
            params.push(("uma_resolution_status", v.clone()));
        }
        if let Some(v) = &self.game_id {
            params.push(("game_id", v.clone()));
        }
        if let Some(v) = &self.sports_market_types {
            if !v.is_empty() {
                params.push(("sports_market_types", join_array(v)));
            }
        }
        if let Some(v) = self.rewards_min_size {
            params.push(("rewards_min_size", v.to_string()));
        }
        if let Some(v) = &self.question_ids {
            if !v.is_empty() {
                params.push(("question_ids", join_array(v)));
            }
        }
        if let Some(v) = self.include_tag {
            params.push(("include_tag", v.to_string()));
        }
        if let Some(v) = self.closed {
            params.push(("closed", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/markets/{id}` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct MarketByIdRequest {
    /// Market ID (path parameter).
    #[builder(into)]
    pub id: u32,
    /// Include tag information.
    pub include_tag: Option<bool>,
}

impl QueryParams for MarketByIdRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.include_tag {
            params.push(("include_tag", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/markets/slug/{slug}` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct MarketBySlugRequest {
    /// Market slug (path parameter).
    #[builder(into)]
    pub slug: String,
    /// Include tag information.
    pub include_tag: Option<bool>,
}

impl QueryParams for MarketBySlugRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.include_tag {
            params.push(("include_tag", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/markets/{id}/tags` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct MarketTagsRequest {
    /// Market ID (path parameter).
    #[builder(into)]
    pub id: u32,
}

impl QueryParams for MarketTagsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}

/// Request body for the `/markets/information` POST endpoint.
#[derive(Debug, Clone, Builder, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MarketsInformationBody {
    /// Filter by market IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<i32>>,
    /// Filter by market slugs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<Vec<String>>,
    /// Filter by closed status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    /// Filter by CLOB token IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clob_token_ids: Option<Vec<String>>,
    /// Filter by condition IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_ids: Option<Vec<String>>,
    /// Filter by market maker addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_maker_address: Option<Vec<String>>,
    /// Minimum liquidity filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidity_num_min: Option<f64>,
    /// Maximum liquidity filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidity_num_max: Option<f64>,
    /// Minimum volume filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_num_min: Option<f64>,
    /// Maximum volume filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_num_max: Option<f64>,
    /// Minimum start date filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date_min: Option<DateTime<Utc>>,
    /// Maximum start date filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date_max: Option<DateTime<Utc>>,
    /// Minimum end date filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date_min: Option<DateTime<Utc>>,
    /// Maximum end date filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date_max: Option<DateTime<Utc>>,
    /// Include related tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_tags: Option<bool>,
    /// Filter by tag ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_id: Option<i32>,
    /// Filter CYOM (Create Your Own Market) markets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cyom: Option<bool>,
    /// Filter by UMA resolution status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uma_resolution_status: Option<String>,
    /// Filter by game ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
    /// Filter by sports market types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sports_market_types: Option<Vec<String>>,
    /// Minimum rewards size filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewards_min_size: Option<f64>,
    /// Filter by question IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question_ids: Option<Vec<String>>,
    /// Include tag information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tags: Option<bool>,
}

// =============================================================================
// Series Endpoints
// =============================================================================

/// Request parameters for the `/series` endpoint.
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct SeriesListRequest {
    /// Maximum number of series to return.
    pub limit: Option<u32>,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Comma-separated list of fields to order by.
    pub order: Option<String>,
    /// Sort in ascending order.
    pub ascending: Option<bool>,
    /// Filter by series slugs.
    pub slug: Option<Vec<String>>,
    /// Filter by category IDs.
    pub categories_ids: Option<Vec<i32>>,
    /// Filter by category labels.
    pub categories_labels: Option<Vec<String>>,
    /// Filter by closed status.
    pub closed: Option<bool>,
    /// Include chat information.
    pub include_chat: Option<bool>,
    /// Filter by recurrence pattern.
    pub recurrence: Option<String>,
}

impl QueryParams for SeriesListRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = &self.order {
            params.push(("order", v.clone()));
        }
        if let Some(v) = self.ascending {
            params.push(("ascending", v.to_string()));
        }
        if let Some(v) = &self.slug {
            if !v.is_empty() {
                params.push(("slug", join_array(v)));
            }
        }
        if let Some(v) = &self.categories_ids {
            if !v.is_empty() {
                params.push(("categories_ids", join_array(v)));
            }
        }
        if let Some(v) = &self.categories_labels {
            if !v.is_empty() {
                params.push(("categories_labels", join_array(v)));
            }
        }
        if let Some(v) = self.closed {
            params.push(("closed", v.to_string()));
        }
        if let Some(v) = self.include_chat {
            params.push(("include_chat", v.to_string()));
        }
        if let Some(v) = &self.recurrence {
            params.push(("recurrence", v.clone()));
        }
        params
    }
}

/// Request parameters for the `/series/{id}` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct SeriesByIdRequest {
    /// Series ID (path parameter).
    #[builder(into)]
    pub id: u32,
    /// Include chat information.
    pub include_chat: Option<bool>,
}

impl QueryParams for SeriesByIdRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.include_chat {
            params.push(("include_chat", v.to_string()));
        }
        params
    }
}

// =============================================================================
// Comments Endpoints
// =============================================================================

/// Request parameters for the `/comments` endpoint.
#[derive(Debug, Clone, Builder, Default)]
#[non_exhaustive]
pub struct CommentsRequest {
    /// Maximum number of comments to return.
    pub limit: Option<u32>,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Comma-separated list of fields to order by.
    pub order: Option<String>,
    /// Sort in ascending order.
    pub ascending: Option<bool>,
    /// Parent entity type (Event, Series, or market).
    pub parent_entity_type: Option<ParentEntityType>,
    /// Parent entity ID.
    pub parent_entity_id: Option<i32>,
    /// Include position information.
    pub get_positions: Option<bool>,
    /// Only return comments from token holders.
    pub holders_only: Option<bool>,
}

impl QueryParams for CommentsRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = &self.order {
            params.push(("order", v.clone()));
        }
        if let Some(v) = self.ascending {
            params.push(("ascending", v.to_string()));
        }
        if let Some(v) = self.parent_entity_type {
            params.push(("parent_entity_type", v.to_string()));
        }
        if let Some(v) = self.parent_entity_id {
            params.push(("parent_entity_id", v.to_string()));
        }
        if let Some(v) = self.get_positions {
            params.push(("get_positions", v.to_string()));
        }
        if let Some(v) = self.holders_only {
            params.push(("holders_only", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/comments/{id}` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct CommentsByIdRequest {
    /// Comment ID (path parameter).
    #[builder(into)]
    pub id: i32,
    /// Include position information.
    pub get_positions: Option<bool>,
}

impl QueryParams for CommentsByIdRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.get_positions {
            params.push(("get_positions", v.to_string()));
        }
        params
    }
}

/// Request parameters for the `/comments/user_address/{user_address}` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct CommentsByUserAddressRequest {
    /// User address (path parameter).
    pub user_address: Address,
    /// Maximum number of comments to return.
    pub limit: Option<u32>,
    /// Pagination offset.
    pub offset: Option<u32>,
    /// Comma-separated list of fields to order by.
    pub order: Option<String>,
    /// Sort in ascending order.
    pub ascending: Option<bool>,
}

impl QueryParams for CommentsByUserAddressRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(v) = self.limit {
            params.push(("limit", v.to_string()));
        }
        if let Some(v) = self.offset {
            params.push(("offset", v.to_string()));
        }
        if let Some(v) = &self.order {
            params.push(("order", v.clone()));
        }
        if let Some(v) = self.ascending {
            params.push(("ascending", v.to_string()));
        }
        params
    }
}

// =============================================================================
// Profiles Endpoints
// =============================================================================

/// Request parameters for the `/public-profile` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct PublicProfileRequest {
    /// Wallet address (proxy wallet or user address).
    pub address: Address,
}

impl QueryParams for PublicProfileRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![("address", self.address.to_string())]
    }
}

// =============================================================================
// Search Endpoints
// =============================================================================

/// Request parameters for the `/public-search` endpoint.
#[derive(Debug, Clone, Builder)]
#[non_exhaustive]
pub struct SearchRequest {
    /// Search query (required).
    #[builder(into)]
    pub q: String,
    /// Use cached results.
    pub cache: Option<bool>,
    /// Filter events by status.
    pub events_status: Option<String>,
    /// Maximum results per type.
    pub limit_per_type: Option<i32>,
    /// Page number for pagination.
    pub page: Option<i32>,
    /// Filter by event tags.
    pub events_tag: Option<Vec<String>>,
    /// Number of closed markets to keep in results.
    pub keep_closed_markets: Option<i32>,
    /// Sort field.
    pub sort: Option<String>,
    /// Sort in ascending order.
    pub ascending: Option<bool>,
    /// Include tags in search.
    pub search_tags: Option<bool>,
    /// Include profiles in search.
    pub search_profiles: Option<bool>,
    /// Filter by recurrence pattern.
    pub recurrence: Option<String>,
    /// Exclude events with these tag IDs.
    pub exclude_tag_id: Option<Vec<i32>>,
    /// Use optimized search.
    pub optimized: Option<bool>,
}

impl QueryParams for SearchRequest {
    fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("q", self.q.clone())];
        if let Some(v) = self.cache {
            params.push(("cache", v.to_string()));
        }
        if let Some(v) = &self.events_status {
            params.push(("events_status", v.clone()));
        }
        if let Some(v) = self.limit_per_type {
            params.push(("limit_per_type", v.to_string()));
        }
        if let Some(v) = self.page {
            params.push(("page", v.to_string()));
        }
        if let Some(v) = &self.events_tag {
            if !v.is_empty() {
                params.push(("events_tag", join_array(v)));
            }
        }
        if let Some(v) = self.keep_closed_markets {
            params.push(("keep_closed_markets", v.to_string()));
        }
        if let Some(v) = &self.sort {
            params.push(("sort", v.clone()));
        }
        if let Some(v) = self.ascending {
            params.push(("ascending", v.to_string()));
        }
        if let Some(v) = self.search_tags {
            params.push(("search_tags", v.to_string()));
        }
        if let Some(v) = self.search_profiles {
            params.push(("search_profiles", v.to_string()));
        }
        if let Some(v) = &self.recurrence {
            params.push(("recurrence", v.clone()));
        }
        if let Some(v) = &self.exclude_tag_id {
            if !v.is_empty() {
                params.push(("exclude_tag_id", join_array(v)));
            }
        }
        if let Some(v) = self.optimized {
            params.push(("optimized", v.to_string()));
        }
        params
    }
}
