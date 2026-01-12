//! Response types for the Polymarket Data API.
//!
//! This module contains structs representing API responses from the Data API endpoints.

use bon::Builder;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer};
use serde_with::{DefaultOnNull, DisplayFromStr, NoneAsEmptyString, serde_as};

use super::{ActivityType, Side};
use crate::types::{Address, B256, Decimal, U256};

/// Deserializes an optional Side, treating empty strings as None.
fn deserialize_optional_side<'de, D>(deserializer: D) -> Result<Option<Side>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => match s.to_uppercase().as_str() {
            "BUY" => Ok(Some(Side::Buy)),
            "SELL" => Ok(Some(Side::Sell)),
            _ => Ok(None),
        },
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum Market {
    /// All markets
    #[serde(alias = "global", alias = "GLOBAL")]
    Global,
    /// Specific market condition ID
    #[serde(untagged)]
    Market(B256),
}

/// Response from the health check endpoint (`/`).
///
/// Returns "OK" when the API is healthy and operational.
#[derive(Debug, Clone, Deserialize, Builder)]
#[non_exhaustive]
pub struct Health {
    /// Health status message (typically "OK").
    pub data: String,
}

/// Error response returned by the API on failure.
///
/// Contains an error message describing what went wrong.
#[derive(Debug, Clone, Deserialize, Builder)]
#[non_exhaustive]
pub struct ApiError {
    /// Human-readable error message.
    pub error: String,
}

/// A user's current (open) position in a prediction market.
///
/// Returned by the `/positions` endpoint. Represents holdings of outcome tokens
/// with associated profit/loss calculations.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Position {
    /// The user's proxy wallet address.
    pub proxy_wallet: Address,
    /// The outcome token asset identifier
    pub asset: U256,
    /// The market condition ID (unique market identifier).
    pub condition_id: B256,
    /// Number of outcome tokens held.
    pub size: Decimal,
    /// Average entry price for the position.
    pub avg_price: Decimal,
    /// Initial value (cost basis) of the position.
    pub initial_value: Decimal,
    /// Current market value of the position.
    pub current_value: Decimal,
    /// Unrealized cash profit/loss.
    pub cash_pnl: Decimal,
    /// Unrealized percentage profit/loss.
    pub percent_pnl: Decimal,
    /// Total amount bought (cumulative).
    pub total_bought: Decimal,
    /// Realized profit/loss from closed portions.
    pub realized_pnl: Decimal,
    /// Realized percentage profit/loss.
    pub percent_realized_pnl: Decimal,
    /// Current market price of the outcome.
    pub cur_price: Decimal,
    /// Whether the position can be redeemed (market resolved).
    pub redeemable: bool,
    /// Whether the position can be merged with opposite outcome.
    pub mergeable: bool,
    /// Market title/question.
    pub title: String,
    /// Market URL slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Parent event URL slug.
    pub event_slug: String,
    /// Parent event ID.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub event_id: Option<String>,
    /// Outcome name (e.g., "Yes", "No", candidate name).
    pub outcome: String,
    /// Outcome index within the market (0 or 1 for binary markets).
    pub outcome_index: i32,
    /// Name of the opposite outcome.
    pub opposite_outcome: String,
    /// Asset identifier of the opposite outcome.
    pub opposite_asset: U256,
    /// Market end/resolution date.
    pub end_date: NaiveDate,
    /// Whether this is a negative risk market.
    pub negative_risk: bool,
}

/// A user's closed (historical) position in a prediction market.
///
/// Returned by the `/closed-positions` endpoint. Represents positions that
/// have been fully sold or redeemed, with final profit/loss figures.
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ClosedPosition {
    /// The user's proxy wallet address.
    pub proxy_wallet: Address,
    /// The outcome token asset identifier (decimal string from API).
    pub asset: U256,
    /// The market condition ID (unique market identifier).
    pub condition_id: B256,
    /// Average entry price for the position.
    pub avg_price: Decimal,
    /// Total amount bought (cumulative).
    pub total_bought: Decimal,
    /// Realized profit/loss from the closed position.
    pub realized_pnl: Decimal,
    /// Final market price when position was closed.
    pub cur_price: Decimal,
    /// Unix timestamp when the position was closed.
    pub timestamp: i64,
    /// Market title/question.
    pub title: String,
    /// Market URL slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Parent event URL slug.
    pub event_slug: String,
    /// Outcome name (e.g., "Yes", "No", candidate name).
    pub outcome: String,
    /// Outcome index within the market (0 or 1 for binary markets).
    pub outcome_index: i32,
    /// Name of the opposite outcome.
    pub opposite_outcome: String,
    /// Asset identifier of the opposite outcome.
    pub opposite_asset: U256,
    /// Market end/resolution date.
    pub end_date: DateTime<Utc>,
}

/// A trade (buy or sell) of outcome tokens.
///
/// Returned by the `/trades` endpoint. Represents an executed order where
/// outcome tokens were bought or sold.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Trade {
    /// The trader's proxy wallet address.
    pub proxy_wallet: Address,
    /// Trade side (BUY or SELL).
    pub side: Side,
    /// The outcome token asset identifier (decimal string from API).
    pub asset: U256,
    /// The market condition ID (unique market identifier).
    pub condition_id: B256,
    /// Number of tokens traded.
    pub size: Decimal,
    /// Execution price per token.
    pub price: Decimal,
    /// Unix timestamp when the trade occurred.
    pub timestamp: i64,
    /// Market title/question.
    pub title: String,
    /// Market URL slug.
    pub slug: String,
    /// Market icon URL.
    pub icon: String,
    /// Parent event URL slug.
    pub event_slug: String,
    /// Outcome name (e.g., "Yes", "No", candidate name).
    pub outcome: String,
    /// Outcome index within the market (0 or 1 for binary markets).
    pub outcome_index: i32,
    /// Trader's display name (if public).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub name: Option<String>,
    /// Trader's pseudonym (if set).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub pseudonym: Option<String>,
    /// Trader's bio (if public).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub bio: Option<String>,
    /// Trader's profile image URL.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub profile_image: Option<String>,
    /// Trader's optimized profile image URL.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub profile_image_optimized: Option<String>,
    /// On-chain transaction hash.
    pub transaction_hash: B256,
}

/// An on-chain activity record for a user.
///
/// Returned by the `/activity` endpoint. Represents various on-chain operations
/// including trades, splits, merges, redemptions, rewards, and conversions.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Activity {
    /// The user's proxy wallet address.
    pub proxy_wallet: Address,
    /// Unix timestamp when the activity occurred.
    pub timestamp: i64,
    /// The market condition ID (unique market identifier).
    /// Can be empty for some activity types (e.g., rewards, conversions).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub condition_id: Option<B256>,
    /// Type of activity (TRADE, SPLIT, MERGE, REDEEM, REWARD, CONVERSION).
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    /// Number of tokens involved in the activity.
    pub size: Decimal,
    /// USDC value of the activity.
    pub usdc_size: Decimal,
    /// On-chain transaction hash.
    pub transaction_hash: B256,
    /// Price per token (for trades).
    pub price: Option<Decimal>,
    /// Outcome token asset identifier
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub asset: Option<U256>,
    /// Trade side (for trades only).
    #[serde(default, deserialize_with = "deserialize_optional_side")]
    pub side: Option<Side>,
    /// Outcome index (for trades).
    pub outcome_index: Option<i32>,
    /// Market title/question.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub title: Option<String>,
    /// Market URL slug.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub slug: Option<String>,
    /// Market icon URL.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub icon: Option<String>,
    /// Parent event URL slug.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub event_slug: Option<String>,
    /// Outcome name.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub outcome: Option<String>,
    /// User's display name (if public).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub name: Option<String>,
    /// User's pseudonym (if set).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub pseudonym: Option<String>,
    /// User's bio (if public).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub bio: Option<String>,
    /// User's profile image URL.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub profile_image: Option<String>,
    /// User's optimized profile image URL.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub profile_image_optimized: Option<String>,
}

/// A holder of outcome tokens in a market.
///
/// Represents a user who holds a position in a specific outcome.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Holder {
    /// The holder's proxy wallet address.
    pub proxy_wallet: Address,
    /// Holder's bio (if public).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub bio: Option<String>,
    /// The outcome token asset identifier (decimal string from API).
    pub asset: U256,
    /// Holder's pseudonym (if set).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub pseudonym: Option<String>,
    /// Amount of tokens held.
    pub amount: Decimal,
    /// Whether the holder's username is publicly visible.
    pub display_username_public: Option<bool>,
    /// Outcome index within the market (0 or 1 for binary markets).
    pub outcome_index: i32,
    /// Holder's display name (if public).
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub name: Option<String>,
    /// Holder's profile image URL.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub profile_image: Option<String>,
    /// Holder's optimized profile image URL.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub profile_image_optimized: Option<String>,
    /// Whether the holder is verified.
    pub verified: Option<bool>,
}

/// Container for holders grouped by token.
///
/// Returned by the `/holders` endpoint. Groups holders by outcome token.
#[derive(Debug, Clone, Deserialize, Builder)]
#[non_exhaustive]
pub struct MetaHolder {
    /// The outcome token identifier
    pub token: U256,
    /// List of holders for this token.
    pub holders: Vec<Holder>,
}

/// Count of unique markets a user has traded.
///
/// Returned by the `/traded` endpoint.
#[derive(Debug, Clone, Deserialize, Builder)]
#[non_exhaustive]
pub struct Traded {
    /// The user's address.
    pub user: Address,
    /// Number of unique markets traded.
    pub traded: i32,
}

/// Total value of a user's positions.
///
/// Returned by the `/value` endpoint.
#[derive(Debug, Clone, Deserialize, Builder)]
#[non_exhaustive]
pub struct Value {
    /// The user's address.
    pub user: Address,
    /// Total value of positions in USDC.
    pub value: Decimal,
}

/// Open interest for a market.
///
/// Returned by the `/oi` endpoint. Open interest represents the total
/// value of outstanding positions in a market.
#[derive(Debug, Clone, Deserialize, Builder)]
#[non_exhaustive]
pub struct OpenInterest {
    /// The market condition ID
    pub market: Market,
    /// Open interest value in USDC.
    pub value: Decimal,
}

/// Trading volume for a specific market.
///
/// Used within [`LiveVolume`] to show per-market volume breakdown.
#[derive(Debug, Clone, Deserialize, Builder)]
#[non_exhaustive]
pub struct MarketVolume {
    /// The market condition ID
    pub market: Market,
    /// Trading volume in USDC.
    pub value: Decimal,
}

/// Live trading volume for an event.
///
/// Returned by the `/live-volume` endpoint. Includes total volume
/// and per-market breakdown.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
#[non_exhaustive]
pub struct LiveVolume {
    /// Total trading volume across all markets in the event.
    pub total: Decimal,
    /// Per-market volume breakdown.
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub markets: Vec<MarketVolume>,
}

/// A builder's entry in the aggregated leaderboard.
///
/// Returned by the `/v1/builders/leaderboard` endpoint. Builders are third-party
/// applications that integrate with Polymarket.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BuilderLeaderboardEntry {
    /// Rank position in the leaderboard.
    #[serde_as(as = "DisplayFromStr")]
    pub rank: i32,
    /// Builder name or identifier.
    pub builder: String,
    /// Total trading volume attributed to this builder.
    pub volume: Decimal,
    /// Number of active users for this builder.
    pub active_users: i32,
    /// Whether the builder is verified.
    pub verified: bool,
    /// URL to the builder's logo image.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub builder_logo: Option<String>,
}

/// A builder's daily volume data point.
///
/// Returned by the `/v1/builders/volume` endpoint. Each entry represents
/// a single day's volume and activity for a builder.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BuilderVolumeEntry {
    /// Timestamp for this entry in ISO 8601 format (e.g., "2025-11-15T00:00:00Z").
    pub dt: DateTime<Utc>,
    /// Builder name or identifier.
    pub builder: String,
    /// URL to the builder's logo image.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub builder_logo: Option<String>,
    /// Whether the builder is verified.
    pub verified: bool,
    /// Trading volume for this builder on this date.
    pub volume: Decimal,
    /// Number of active users for this builder on this date.
    pub active_users: i32,
    /// Rank position on this date.
    #[serde_as(as = "DisplayFromStr")]
    pub rank: i32,
}

/// A trader's entry in the leaderboard.
///
/// Returned by the `/v1/leaderboard` endpoint. Shows trader rankings
/// by profit/loss or volume.
#[serde_as]
#[derive(Debug, Clone, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TraderLeaderboardEntry {
    /// Rank position in the leaderboard.
    #[serde_as(as = "DisplayFromStr")]
    pub rank: i32,
    /// The trader's proxy wallet address.
    pub proxy_wallet: Address,
    /// The trader's username.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub user_name: Option<String>,
    /// Trading volume for this trader.
    pub vol: Decimal,
    /// Profit and loss for this trader.
    pub pnl: Decimal,
    /// URL to the trader's profile image.
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub profile_image: Option<String>,
    /// The trader's X (Twitter) username
    #[serde(default)]
    #[serde_as(as = "NoneAsEmptyString")]
    pub x_username: Option<String>,
    /// Whether the trader has a verified badge.
    pub verified_badge: Option<bool>,
}
