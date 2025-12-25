use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder, Default)]
#[builder(on(String, into))]
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
#[builder(on(String, into))]
#[serde(rename_all = "camelCase")]
pub struct ListedTeam {
    pub id: u32,
    pub name: Option<String>,
    pub league: Option<String>,
    pub record: Option<String>,
    pub logo: Option<String>,
    pub abbreviation: Option<String>,
    pub alias: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub type SportsMetadataResponse = Vec<Sport>;

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq, Clone)]
#[builder(on(String, into))]
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
#[serde(rename_all = "camelCase")]
pub struct SportsMarketTypesResponse {
    pub market_types: Vec<String>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder, Clone, Default)]
#[builder(on(String, into))]
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
#[builder(on(String, into))]
#[serde(rename_all = "camelCase")]
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

#[non_exhaustive]
#[derive(Debug, Deserialize, Builder, PartialEq, Clone)]
#[builder(on(String, into))]
#[serde(rename_all = "camelCase")]
pub struct TagRelationship {
    pub id: String,
    #[serde(rename = "tagID")]
    pub tag_id: Option<i64>,
    #[serde(rename = "relatedTagID")]
    pub related_tag_id: Option<u64>,
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
#[builder(on(String, into))]
pub struct RelatedTagsByIdRequest {
    #[serde(skip_serializing)]
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omit_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RelatedTagsStatus>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder, Clone)]
#[builder(on(String, into))]
pub struct RelatedTagsBySlugRequest {
    #[serde(skip_serializing)]
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omit_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RelatedTagsStatus>,
}
