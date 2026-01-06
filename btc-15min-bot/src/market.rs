//! Market discovery and selection logic
//!
//! This module handles discovering Bitcoin 15min markets and selecting
//! the next upcoming market that hasn't started yet.

use anyhow::{Context, Result};
use chrono::{DateTime, Timelike, Utc};
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::response::SimplifiedMarketResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

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

/// Market discovery service
pub struct MarketDiscovery {
    client: Client,
    query: String,
}

impl MarketDiscovery {
    /// Create a new market discovery service
    pub fn new(client: Client, query: String) -> Self {
        Self { client, query }
    }

    /// Find all Bitcoin 15min markets
    ///
    /// This method searches for markets matching the query string and parses
    /// their metadata to extract start/end times and token IDs.
    pub async fn find_all_btc_markets(&self) -> Result<Vec<BtcMarket>> {
        info!("Searching for markets matching: '{}'", self.query);

        // Use simplified markets endpoint for faster queries
        let page = self.client
            .simplified_markets(None)
            .await
            .context("Failed to fetch simplified markets")?;

        debug!("Fetched {} total markets from API", page.data.len());

        let mut btc_markets = Vec::new();

        for market in &page.data {
            // Filter by question containing our query
            if !market.question.contains(&self.query) {
                continue;
            }

            // Skip if market is already closed
            if market.closed {
                debug!("Skipping closed market: {}", market.question);
                continue;
            }

            // Parse start and end times from the question
            // Expected format: "Bitcoin 15min Up or Down? (HH:MM - HH:MM UTC)"
            let (start_time, end_time) = match Self::parse_market_times(&market.question) {
                Ok(times) => times,
                Err(e) => {
                    warn!("Failed to parse times from '{}': {}", market.question, e);
                    continue;
                }
            };

            // Get token IDs for UP and DOWN outcomes
            let (up_token_id, down_token_id) = Self::get_outcome_token_ids(&market)?;

            let btc_market = BtcMarket {
                condition_id: market.condition_id.clone(),
                question: market.question.clone(),
                start_time,
                end_time,
                up_token_id,
                down_token_id,
                active: market.active,
                closed: market.closed,
            };

            debug!(
                "Found BTC market: {} | Start: {} | End: {}",
                btc_market.question,
                btc_market.start_time,
                btc_market.end_time
            );

            btc_markets.push(btc_market);
        }

        info!("Discovered {} Bitcoin 15min markets", btc_markets.len());
        Ok(btc_markets)
    }

    /// Find the next upcoming market that hasn't started yet
    pub async fn find_next_upcoming_market(&self) -> Result<Option<BtcMarket>> {
        let mut markets = self.find_all_btc_markets().await?;

        if markets.is_empty() {
            warn!("No Bitcoin 15min markets found");
            return Ok(None);
        }

        let now = Utc::now();

        // Filter to only upcoming markets (not started yet)
        markets.retain(|m| m.start_time > now);

        if markets.is_empty() {
            warn!("No upcoming markets found (all have already started)");
            return Ok(None);
        }

        // Sort by start time (earliest first)
        markets.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        let next_market = markets.into_iter().next();

        if let Some(ref market) = next_market {
            let time_until_start = market.start_time - now;
            info!(
                "Next upcoming market: {} | Starts in: {} seconds",
                market.question,
                time_until_start.num_seconds()
            );
        }

        Ok(next_market)
    }

    /// Parse start and end times from market question
    ///
    /// Expected formats:
    /// - "Bitcoin 15min Up or Down? (HH:MM - HH:MM UTC)"
    /// - "Bitcoin 15min Up or Down? HH:MM - HH:MM UTC"
    fn parse_market_times(question: &str) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
        // Extract time range from parentheses or after "?"
        let time_str = if let Some(start) = question.find('(') {
            if let Some(end) = question.find(')') {
                &question[start + 1..end]
            } else {
                anyhow::bail!("No closing parenthesis found");
            }
        } else {
            // Try to find after "?"
            if let Some(idx) = question.find('?') {
                question[idx + 1..].trim()
            } else {
                anyhow::bail!("Cannot find time range in question");
            }
        };

        // Parse "HH:MM - HH:MM UTC" or "HH:MM - HH:MM"
        let parts: Vec<&str> = time_str
            .trim()
            .trim_end_matches("UTC")
            .trim()
            .split('-')
            .map(|s| s.trim())
            .collect();

        if parts.len() != 2 {
            anyhow::bail!("Invalid time range format: {}", time_str);
        }

        let start_time_str = parts[0];
        let end_time_str = parts[1];

        // Parse HH:MM format
        let start_time = Self::parse_time_today(start_time_str)?;
        let mut end_time = Self::parse_time_today(end_time_str)?;

        // If end time is before start time, it's the next day
        if end_time <= start_time {
            end_time = end_time + chrono::Duration::days(1);
        }

        Ok((start_time, end_time))
    }

    /// Parse HH:MM format to DateTime<Utc> for today
    fn parse_time_today(time_str: &str) -> Result<DateTime<Utc>> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid time format: {}", time_str);
        }

        let hour: u32 = parts[0]
            .parse()
            .context("Invalid hour")?;
        let minute: u32 = parts[1]
            .parse()
            .context("Invalid minute")?;

        let now = Utc::now();
        let today = now.date_naive();

        let datetime = today
            .and_hms_opt(hour, minute, 0)
            .context("Invalid time")?
            .and_utc();

        // If the time has already passed today, it's for the next occurrence
        if datetime < now {
            // Could be tomorrow or could be a future market
            // For now, return as-is and let the caller filter
            Ok(datetime)
        } else {
            Ok(datetime)
        }
    }

    /// Get token IDs for UP and DOWN outcomes
    fn get_outcome_token_ids(
        market: &SimplifiedMarketResponse,
    ) -> Result<(String, String)> {
        // Simplified markets should have tokens array
        if market.tokens.len() < 2 {
            anyhow::bail!("Market has less than 2 tokens");
        }

        // Typically index 0 is UP (Yes) and index 1 is DOWN (No)
        // But we should verify by checking outcome names if available
        let up_token_id = market.tokens.first()
            .context("No first token")?
            .token_id.clone();

        let down_token_id = market.tokens.get(1)
            .context("No second token")?
            .token_id.clone();

        Ok((up_token_id, down_token_id))
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
    fn test_parse_market_times() {
        let question = "Bitcoin 15min Up or Down? (14:30 - 14:45 UTC)";
        let result = MarketDiscovery::parse_market_times(question);
        assert!(result.is_ok());

        let (start, end) = result.unwrap();
        assert_eq!(start.hour(), 14);
        assert_eq!(start.minute(), 30);
        assert_eq!(end.hour(), 14);
        assert_eq!(end.minute(), 45);
    }

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
}
