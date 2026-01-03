//! Comprehensive Data API endpoint explorer.
//!
//! This example dynamically tests all Data API endpoints by:
//! 1. Fetching leaderboard first to get real user addresses
//! 2. Using those addresses for user-based queries (positions, trades, activity, etc.)
//! 3. Extracting condition IDs from positions/trades for market-based queries
//! 4. Using Gamma API to get event IDs for live volume queries
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example data --features data,gamma,tracing
//! ```

use polymarket_client_sdk::data::Client;
use polymarket_client_sdk::data::types::request::{
    ActivityRequest, BuilderLeaderboardRequest, BuilderVolumeRequest, ClosedPositionsRequest,
    HoldersRequest, LiveVolumeRequest, OpenInterestRequest, PositionsRequest, TradedRequest,
    TraderLeaderboardRequest, TradesRequest, ValueRequest,
};
use polymarket_client_sdk::data::types::{LeaderboardCategory, TimePeriod};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::EventsRequest;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();
    let gamma = GammaClient::default();

    // Health check
    match client.health().await {
        Ok(h) => info!(endpoint = "health", status = %h.data),
        Err(e) => debug!(endpoint = "health", error = %e),
    }

    // Get leaderboard to extract user addresses
    let leaderboard_result = client
        .leaderboard(
            &TraderLeaderboardRequest::builder()
                .category(LeaderboardCategory::Overall)
                .time_period(TimePeriod::Week)
                .limit(10)?
                .build(),
        )
        .await;

    let user_address = match &leaderboard_result {
        Ok(entries) => {
            info!(endpoint = "leaderboard", count = entries.len());
            entries.first().map(|e| e.proxy_wallet)
        }
        Err(e) => {
            debug!(endpoint = "leaderboard", error = %e);
            None
        }
    };

    // Builder leaderboard
    match client
        .builder_leaderboard(
            &BuilderLeaderboardRequest::builder()
                .time_period(TimePeriod::Week)
                .limit(5)?
                .build(),
        )
        .await
    {
        Ok(entries) => info!(endpoint = "builder_leaderboard", count = entries.len()),
        Err(e) => debug!(endpoint = "builder_leaderboard", error = %e),
    }

    // Builder volume
    match client
        .builder_volume(
            &BuilderVolumeRequest::builder()
                .time_period(TimePeriod::Week)
                .build(),
        )
        .await
    {
        Ok(entries) => info!(endpoint = "builder_volume", count = entries.len()),
        Err(e) => debug!(endpoint = "builder_volume", error = %e),
    }

    // Open interest (without market filter first)
    let oi_result = client.open_interest(&OpenInterestRequest::default()).await;

    let condition_ids_from_oi: Vec<String> = match &oi_result {
        Ok(entries) => {
            info!(endpoint = "open_interest", count = entries.len());
            entries.iter().take(5).map(|e| e.market.clone()).collect()
        }
        Err(e) => {
            debug!(endpoint = "open_interest", error = %e);
            Vec::new()
        }
    };

    // Get events from Gamma for live_volume
    // Note: Data API live_volume uses numeric event IDs, while Gamma uses string IDs.
    // We try to get a game_id from sports events, or parse the event ID as a number.
    let events_result = gamma
        .events(
            &EventsRequest::builder()
                .active(true)
                .limit(10)
                .order(vec!["volume".to_owned()])
                .ascending(false)
                .build(),
        )
        .await;

    let event_id: Option<u64> = match &events_result {
        Ok(events) => {
            info!(endpoint = "gamma_events", count = events.len());
            // Try to find an event with a numeric game_id (sports events)
            // or try parsing the event ID as a number
            events.iter().find_map(|e| {
                e.game_id
                    .and_then(|id| u64::try_from(id).ok())
                    .or_else(|| e.id.parse::<u64>().ok())
            })
        }
        Err(e) => {
            debug!(endpoint = "gamma_events", error = %e);
            None
        }
    };

    // User-based queries (if we have a user address from leaderboard)
    let mut condition_ids_from_positions: Vec<String> = Vec::new();

    if let Some(user) = user_address {
        info!(user = %user, "Using trader from leaderboard");

        // Positions
        let positions_result = client
            .positions(&PositionsRequest::builder().user(user).limit(10)?.build())
            .await;

        match &positions_result {
            Ok(positions) => {
                info!(endpoint = "positions", user = %user, count = positions.len());
                condition_ids_from_positions = positions
                    .iter()
                    .take(5)
                    .map(|p| p.condition_id.clone())
                    .collect();
            }
            Err(e) => debug!(endpoint = "positions", user = %user, error = %e),
        }

        // Trades
        match client
            .trades(&TradesRequest::builder().user(user).limit(10)?.build())
            .await
        {
            Ok(trades) => info!(endpoint = "trades", user = %user, count = trades.len()),
            Err(e) => debug!(endpoint = "trades", user = %user, error = %e),
        }

        // Activity
        match client
            .activity(&ActivityRequest::builder().user(user).limit(10)?.build())
            .await
        {
            Ok(activities) => {
                info!(endpoint = "activity", user = %user, count = activities.len());
            }
            Err(e) => debug!(endpoint = "activity", user = %user, error = %e),
        }

        // Closed positions
        match client
            .closed_positions(
                &ClosedPositionsRequest::builder()
                    .user(user)
                    .limit(10)?
                    .build(),
            )
            .await
        {
            Ok(positions) => {
                info!(endpoint = "closed_positions", user = %user, count = positions.len());
            }
            Err(e) => debug!(endpoint = "closed_positions", user = %user, error = %e),
        }

        // Traded count
        match client
            .traded(&TradedRequest::builder().user(user).build())
            .await
        {
            Ok(traded) => {
                info!(endpoint = "traded", user = %user, count = traded.traded);
            }
            Err(e) => debug!(endpoint = "traded", user = %user, error = %e),
        }

        // Value
        match client
            .value(&ValueRequest::builder().user(user).build())
            .await
        {
            Ok(values) => {
                let total: rust_decimal::Decimal = values.iter().map(|v| v.value).sum();
                info!(endpoint = "value", user = %user, total = %total);
            }
            Err(e) => debug!(endpoint = "value", user = %user, error = %e),
        }
    } else {
        debug!("skipping user-based queries - no user address from leaderboard");
    }

    // Market-based queries using condition IDs
    let condition_ids = if condition_ids_from_positions.is_empty() {
        condition_ids_from_oi
    } else {
        condition_ids_from_positions
    };

    if condition_ids.is_empty() {
        debug!("skipping market-based queries - no condition IDs available");
    } else {
        info!(
            condition_ids = condition_ids.len(),
            "Using condition IDs for market queries"
        );

        // Holders
        match client
            .holders(
                &HoldersRequest::builder()
                    .markets(condition_ids.clone())
                    .limit(5)?
                    .build(),
            )
            .await
        {
            Ok(holders) => {
                let total_holders: usize = holders.iter().map(|h| h.holders.len()).sum();
                info!(
                    endpoint = "holders",
                    markets = condition_ids.len(),
                    total_holders = total_holders
                );
            }
            Err(e) => debug!(endpoint = "holders", error = %e),
        }

        // Open interest with specific markets
        match client
            .open_interest(
                &OpenInterestRequest::builder()
                    .markets(condition_ids.clone())
                    .build(),
            )
            .await
        {
            Ok(entries) => {
                info!(
                    endpoint = "open_interest_filtered",
                    markets = condition_ids.len(),
                    count = entries.len()
                );
            }
            Err(e) => debug!(endpoint = "open_interest_filtered", error = %e),
        }
    }

    // Live volume (requires event ID from Gamma)
    if let Some(id) = event_id {
        match client
            .live_volume(&LiveVolumeRequest::builder().id(id).build())
            .await
        {
            Ok(volumes) => {
                let total: rust_decimal::Decimal = volumes.iter().map(|v| v.total).sum();
                info!(endpoint = "live_volume", event_id = id, total = %total);
            }
            Err(e) => debug!(endpoint = "live_volume", event_id = id, error = %e),
        }
    } else {
        debug!(endpoint = "live_volume", "skipped - no event ID from Gamma");
    }

    Ok(())
}
