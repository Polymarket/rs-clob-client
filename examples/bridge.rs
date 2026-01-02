//! Comprehensive Bridge API endpoint explorer.
//!
//! This example dynamically tests all Bridge API endpoints by:
//! 1. Fetching supported assets to show available chains and tokens
//! 2. Getting a real user address from the Data API leaderboard
//! 3. Using that address to fetch deposit addresses
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example bridge --features bridge,data,tracing
//! ```

use polymarket_client_sdk::bridge::Client;
use polymarket_client_sdk::bridge::types::DepositRequest;
use polymarket_client_sdk::data::Client as DataClient;
use polymarket_client_sdk::data::types::request::TraderLeaderboardRequest;
use polymarket_client_sdk::data::types::{LeaderboardCategory, TimePeriod};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();
    let data_client = DataClient::default();

    // Get supported assets
    match client.supported_assets().await {
        Ok(response) => {
            info!(
                endpoint = "supported_assets",
                count = response.supported_assets.len()
            );

            // Group by chain for better logging
            let mut chains: std::collections::HashMap<&str, Vec<&str>> =
                std::collections::HashMap::new();
            for asset in &response.supported_assets {
                chains
                    .entry(&asset.chain_name)
                    .or_default()
                    .push(&asset.token.symbol);
            }

            for (chain, tokens) in &chains {
                info!(
                    endpoint = "supported_assets",
                    chain = %chain,
                    tokens = ?tokens
                );
            }

            if let Some(note) = &response.note {
                info!(endpoint = "supported_assets", note = %note);
            }
        }
        Err(e) => debug!(endpoint = "supported_assets", error = %e),
    }

    // Get a real user address from the leaderboard
    let leaderboard_result = data_client
        .leaderboard(
            &TraderLeaderboardRequest::builder()
                .category(LeaderboardCategory::Overall)
                .time_period(TimePeriod::Week)
                .limit(5)?
                .build(),
        )
        .await;

    let user_address = match &leaderboard_result {
        Ok(entries) => {
            info!(endpoint = "data_leaderboard", count = entries.len());
            entries.first().map(|e| e.proxy_wallet)
        }
        Err(e) => {
            debug!(endpoint = "data_leaderboard", error = %e);
            None
        }
    };

    // Get deposit addresses using the real user address
    if let Some(address) = user_address {
        info!(user = %address, "Using trader from leaderboard for deposit addresses");

        match client
            .deposit(&DepositRequest::builder().address(address).build())
            .await
        {
            Ok(response) => {
                info!(
                    endpoint = "deposit",
                    user = %address,
                    evm = %response.address.evm,
                    svm = %response.address.svm,
                    btc = %response.address.btc
                );
                if let Some(note) = &response.note {
                    info!(endpoint = "deposit", note = %note);
                }
            }
            Err(e) => debug!(endpoint = "deposit", user = %address, error = %e),
        }
    } else {
        debug!(
            endpoint = "deposit",
            "skipped - no user address from leaderboard"
        );
    }

    Ok(())
}
