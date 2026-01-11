//! Comprehensive Data API endpoint explorer.
//!
//! This example dynamically tests all Data API endpoints by:
//! 1. Fetching leaderboard data to discover real trader addresses
//! 2. Using those addresses for user-specific queries
//! 3. Extracting market IDs from positions for holder lookups
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example data --features data,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=data.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example data --features data,tracing
//! ```

use std::fs::File;

use polymarket_client_sdk::data::Client;
use polymarket_client_sdk::data::types::request::{
    ActivityRequest, BuilderLeaderboardRequest, BuilderVolumeRequest, ClosedPositionsRequest,
    HoldersRequest, LiveVolumeRequest, OpenInterestRequest, PositionsRequest, TradedRequest,
    TraderLeaderboardRequest, TradesRequest, ValueRequest,
};
use polymarket_client_sdk::data::types::{LeaderboardCategory, TimePeriod};
use polymarket_client_sdk::types::{Address, B256, address, b256};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Ok(path) = std::env::var("LOG_FILE") {
        let file = File::create(path)?;
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file)
                    .with_ansi(false),
            )
            .init();
    } else {
        tracing_subscriber::fmt::init();
    }

    let client = Client::default();

    // Fallback test data when dynamic discovery fails
    let fallback_user = address!("56687bf447db6ffa42ffe2204a05edaa20f55839");
    let fallback_market = b256!("dd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917");

    // Health check
    match client.health().await {
        Ok(status) => info!(endpoint = "health", status = %status.data),
        Err(e) => error!(endpoint = "health", error = %e),
    }

    // Fetch leaderboard to get real trader addresses
    let leaderboard_result = client
        .leaderboard(
            &TraderLeaderboardRequest::builder()
                .category(LeaderboardCategory::Overall)
                .time_period(TimePeriod::Week)
                .limit(10)?
                .build(),
        )
        .await;

    let user: Option<Address> = match &leaderboard_result {
        Ok(traders) => {
            info!(endpoint = "leaderboard", count = traders.len());
            if let Some(trader) = traders.first() {
                info!(
                    endpoint = "leaderboard",
                    rank = %trader.rank,
                    address = %trader.proxy_wallet,
                    pnl = %trader.pnl,
                    volume = %trader.vol
                );
                Some(trader.proxy_wallet)
            } else {
                None
            }
        }
        Err(e) => {
            warn!(endpoint = "leaderboard", error = %e, "using fallback user");
            Some(fallback_user)
        }
    };

    // Fetch positions for the discovered user
    let market_id: Option<B256> = if let Some(user) = user {
        let positions_result = client
            .positions(&PositionsRequest::builder().user(user).limit(10)?.build())
            .await;

        match &positions_result {
            Ok(positions) => {
                info!(endpoint = "positions", user = %user, count = positions.len());
                if let Some(pos) = positions.first() {
                    info!(
                        endpoint = "positions",
                        market = %pos.condition_id,
                        size = %pos.size,
                        value = %pos.current_value
                    );
                    Some(pos.condition_id)
                } else {
                    // No positions found, use fallback market
                    warn!(
                        endpoint = "positions",
                        "no positions, using fallback market"
                    );
                    Some(fallback_market)
                }
            }
            Err(e) => {
                warn!(endpoint = "positions", user = %user, error = %e, "using fallback market");
                Some(fallback_market)
            }
        }
    } else {
        debug!(endpoint = "positions", "skipped - no user address found");
        Some(fallback_market)
    };

    // Fetch holders for the discovered market
    if let Some(market) = market_id {
        match client
            .holders(
                &HoldersRequest::builder()
                    .markets(vec![market])
                    .limit(5)?
                    .build(),
            )
            .await
        {
            Ok(meta_holders) => {
                info!(endpoint = "holders", market = %market, tokens = meta_holders.len());
                if let Some(meta) = meta_holders.first() {
                    info!(
                        endpoint = "holders",
                        token = %meta.token,
                        holders_count = meta.holders.len()
                    );
                    if let Some(holder) = meta.holders.first() {
                        info!(
                            endpoint = "holders",
                            address = %holder.proxy_wallet,
                            amount = %holder.amount
                        );
                    }
                }
            }
            Err(e) => error!(endpoint = "holders", market = %market, error = %e),
        }
    }

    // User activity, value, closed positions, and traded count
    if let Some(user) = user {
        match client
            .activity(&ActivityRequest::builder().user(user).limit(5)?.build())
            .await
        {
            Ok(activities) => {
                info!(endpoint = "activity", user = %user, count = activities.len());
                if let Some(act) = activities.first() {
                    info!(
                        endpoint = "activity",
                        activity_type = ?act.activity_type,
                        transaction = %act.transaction_hash
                    );
                }
            }
            Err(e) => error!(endpoint = "activity", user = %user, error = %e),
        }

        match client
            .value(&ValueRequest::builder().user(user).build())
            .await
        {
            Ok(values) => {
                info!(endpoint = "value", user = %user, count = values.len());
                if let Some(value) = values.first() {
                    info!(
                        endpoint = "value",
                        user = %value.user,
                        total = %value.value
                    );
                }
            }
            Err(e) => error!(endpoint = "value", user = %user, error = %e),
        }

        match client
            .closed_positions(
                &ClosedPositionsRequest::builder()
                    .user(user)
                    .limit(5)?
                    .build(),
            )
            .await
        {
            Ok(positions) => {
                info!(endpoint = "closed_positions", user = %user, count = positions.len());
                if let Some(pos) = positions.first() {
                    info!(
                        endpoint = "closed_positions",
                        market = %pos.condition_id,
                        realized_pnl = %pos.realized_pnl
                    );
                }
            }
            Err(e) => error!(endpoint = "closed_positions", user = %user, error = %e),
        }

        match client
            .traded(&TradedRequest::builder().user(user).build())
            .await
        {
            Ok(traded) => {
                info!(
                    endpoint = "traded",
                    user = %user,
                    markets_traded = traded.traded
                );
            }
            Err(e) => error!(endpoint = "traded", user = %user, error = %e),
        }
    }

    // Trades - global trade feed
    match client.trades(&TradesRequest::default()).await {
        Ok(trades) => {
            info!(endpoint = "trades", count = trades.len());
            if let Some(trade) = trades.first() {
                info!(
                    endpoint = "trades",
                    market = %trade.condition_id,
                    side = ?trade.side,
                    size = %trade.size,
                    price = %trade.price
                );
            }
        }
        Err(e) => error!(endpoint = "trades", error = %e),
    }

    // Open interest
    match client.open_interest(&OpenInterestRequest::default()).await {
        Ok(oi_list) => {
            info!(endpoint = "open_interest", count = oi_list.len());
            if let Some(oi) = oi_list.first() {
                info!(
                    endpoint = "open_interest",
                    market = ?oi.market,
                    value = %oi.value
                );
            }
        }
        Err(e) => error!(endpoint = "open_interest", error = %e),
    }

    // Live volume (using event ID 1 as example)
    match client
        .live_volume(&LiveVolumeRequest::builder().id(1).build())
        .await
    {
        Ok(volumes) => {
            info!(
                endpoint = "live_volume",
                event_id = 1,
                count = volumes.len()
            );
            if let Some(vol) = volumes.first() {
                info!(
                    endpoint = "live_volume",
                    total = %vol.total,
                    markets = vol.markets.len()
                );
            }
        }
        Err(e) => error!(endpoint = "live_volume", event_id = 1, error = %e),
    }

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
        Ok(builders) => {
            info!(endpoint = "builder_leaderboard", count = builders.len());
            if let Some(builder) = builders.first() {
                info!(
                    endpoint = "builder_leaderboard",
                    name = %builder.builder,
                    volume = %builder.volume,
                    rank = %builder.rank
                );
            }
        }
        Err(e) => error!(endpoint = "builder_leaderboard", error = %e),
    }

    // Builder volume time series
    match client
        .builder_volume(
            &BuilderVolumeRequest::builder()
                .time_period(TimePeriod::Week)
                .build(),
        )
        .await
    {
        Ok(volumes) => {
            info!(endpoint = "builder_volume", count = volumes.len());
            if let Some(vol) = volumes.first() {
                info!(
                    endpoint = "builder_volume",
                    builder = %vol.builder,
                    date = %vol.dt,
                    volume = %vol.volume
                );
            }
        }
        Err(e) => error!(endpoint = "builder_volume", error = %e),
    }

    Ok(())
}
