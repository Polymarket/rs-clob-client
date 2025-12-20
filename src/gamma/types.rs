use crate::error::Error;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

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
