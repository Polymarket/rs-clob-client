use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder, Default)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
#[builder(default)]
pub struct ListTeamsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Comma-separated list of fields to order by
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub league: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<Vec<String>>,
}

pub type ListTeamsResponse = Vec<ListedTeam>;

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq, Clone)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ListedTeam {
    pub id: u32,
    #[builder(default)]
    pub name: Option<String>,
    #[builder(default)]
    pub league: Option<String>,
    #[builder(default)]
    pub record: Option<String>,
    #[builder(default)]
    pub logo: Option<String>,
    #[builder(default)]
    pub abbreviation: Option<String>,
    #[builder(default)]
    pub alias: Option<String>,
    #[builder(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[builder(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

pub type SportsMetadataResponse = Vec<Sport>;

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq, Clone)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into))]
#[serde(rename_all = "camelCase")]
pub struct Sport {
    pub sport: String,
    pub image: String,
    pub resolution: String,
    pub ordering: String,
    pub tags: String,
    pub series: String,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq, Clone)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[serde(rename_all = "camelCase")]
#[builder(setter(into))]
pub struct SportsMarketTypesResponse {
    pub market_types: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder, Clone, Default)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
#[builder(default)]
pub struct TagsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Comma-separated list of fields to order by
    pub order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_template: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_carousel: Option<bool>,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq, Clone)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    #[builder(default)]
    pub label: Option<String>,
    #[builder(default)]
    pub slug: Option<String>,
    #[builder(default)]
    pub force_show: Option<bool>,
    #[builder(default)]
    pub published_at: Option<String>,
    #[builder(default)]
    pub created_by: Option<i64>,
    #[builder(default)]
    pub updated_by: Option<i64>,
    #[builder(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[builder(default)]
    pub updated_at: Option<DateTime<Utc>>,
    #[builder(default)]
    pub force_hide: Option<bool>,
    #[builder(default)]
    pub is_carousel: Option<bool>,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq, Clone)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct TagRelationship {
    pub id: String,
    #[serde(rename = "tagID")]
    #[builder(default)]
    pub tag_id: Option<i64>,
    #[serde(rename = "relatedTagID")]
    #[builder(default)]
    pub related_tag_id: Option<u64>,
    #[builder(default)]
    pub rank: Option<u64>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelatedTagsStatus {
    Active,
    Closed,
    All,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
pub struct RelatedTagsByIdRequest {
    #[serde(skip_serializing)]
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub omit_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub status: Option<RelatedTagsStatus>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "Error"))]
#[builder(setter(into, strip_option))]
pub struct RelatedTagsBySlugRequest {
    #[serde(skip_serializing)]
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub omit_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub status: Option<RelatedTagsStatus>,
}
