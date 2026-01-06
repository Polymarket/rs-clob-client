//! Simplified market discovery for Bitcoin 15min markets
//!
//! This version uses HARDCODED token IDs to bypass complex API discovery.
//! Perfect for testing and beginners!

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ============================================================================
// CONFIGURE YOUR MARKET HERE
// ============================================================================
// TODO: Replace these with actual token IDs from Polymarket
// Find them at: https://polymarket.com (search "Bitcoin 15min")
//
// Example format: "71321045880685926196226762161781486336990587556547664985916499732654256447732"

/// Token ID for UP outcome (YES the price will go up)
pub const UP_TOKEN_ID: &str = "REPLACE_WITH_UP_TOKEN_ID";

/// Token ID for DOWN outcome (NO the price will not go up)
pub const DOWN_TOKEN_ID: &str = "REPLACE_WITH_DOWN_TOKEN_ID";

/// Market condition ID (can be any string for testing)
pub const CONDITION_ID: &str = "test_btc_15min_market";

// ============================================================================

/// Represents a Bitcoin 15min market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcMarket {
    /// Unique condition ID for the market
    pub condition_id: String,

    /// Market question/description
    pub question: String,

    /// Market start time (UTC)
    pub start_time: DateTime<Utc>,

    /// Market end time (UTC)
    pub end_time: DateTime<Utc>,

    /// Token ID for UP outcome
    pub up_token_id: String,

    /// Token ID for DOWN outcome
    pub down_token_id: String,

    /// Whether market is active
    pub active: bool,

    /// Whether market has closed
    pub closed: bool,
}

/// Market outcome side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketSide {
    Up,
    Down,
}

impl MarketSide {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "UP",
            Self::Down => "DOWN",
        }
    }
}

impl std::fmt::Display for MarketSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Simplified market discovery (uses hardcoded values)
pub struct MarketDiscovery {
    // We don't need the client for the simplified version
    _query: String,
}

impl MarketDiscovery {
    /// Create a new simplified market discovery
    pub fn new(_query: String) -> Self {
        Self { _query }
    }

    /// Find the next upcoming market (returns hardcoded market)
    pub async fn find_next_upcoming_market(&self) -> Result<Option<BtcMarket>> {
        // Check if token IDs are configured
        if UP_TOKEN_ID == "REPLACE_WITH_UP_TOKEN_ID" || DOWN_TOKEN_ID == "REPLACE_WITH_DOWN_TOKEN_ID" {
            warn!("⚠️  Token IDs not configured!");
            warn!("⚠️  Please edit src/market_simple.rs and add real token IDs");
            warn!("⚠️  Find them at https://polymarket.com (search 'Bitcoin 15min')");
            return Ok(None);
        }

        let now = Utc::now();

        // Create a market that "starts" in 2 minutes and "ends" in 17 minutes
        // This gives the bot time to analyze and enter
        let market = BtcMarket {
            condition_id: CONDITION_ID.to_string(),
            question: "Bitcoin 15min Up or Down? (Hardcoded Test Market)".to_string(),
            start_time: now + chrono::Duration::minutes(2),
            end_time: now + chrono::Duration::minutes(17),
            up_token_id: UP_TOKEN_ID.to_string(),
            down_token_id: DOWN_TOKEN_ID.to_string(),
            active: true,
            closed: false,
        };

        info!("📊 Using hardcoded Bitcoin 15min market");
        info!("   UP Token:   {}", &market.up_token_id[..20]);
        info!("   DOWN Token: {}", &market.down_token_id[..20]);
        info!("   Starts in:  {} seconds", (market.start_time - now).num_seconds());

        Ok(Some(market))
    }

    /// Check if a market has started
    pub fn has_market_started(market: &BtcMarket) -> bool {
        Utc::now() >= market.start_time
    }

    /// Check if a market has ended
    pub fn has_market_ended(market: &BtcMarket) -> bool {
        Utc::now() >= market.end_time
    }

    /// Get time until market starts (in seconds)
    pub fn seconds_until_start(market: &BtcMarket) -> i64 {
        (market.start_time - Utc::now()).num_seconds()
    }

    /// Get time until market ends (in seconds)
    pub fn seconds_until_end(market: &BtcMarket) -> i64 {
        (market.end_time - Utc::now()).num_seconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_side_display() {
        assert_eq!(MarketSide::Up.to_string(), "UP");
        assert_eq!(MarketSide::Down.to_string(), "DOWN");
    }

    #[test]
    fn test_market_side_opposite() {
        assert_eq!(MarketSide::Up.opposite(), MarketSide::Down);
        assert_eq!(MarketSide::Down.opposite(), MarketSide::Up);
    }

    #[tokio::test]
    async fn test_find_market() {
        let discovery = MarketDiscovery::new("Bitcoin 15min".to_string());

        // This will return None because token IDs aren't configured
        let result = discovery.find_next_upcoming_market().await;
        assert!(result.is_ok());
    }
}
