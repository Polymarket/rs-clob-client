//! Comprehensive CLOB WebSocket endpoint explorer.
//!
//! This example dynamically tests CLOB WebSocket streaming endpoints by:
//! 1. Fetching active markets from Gamma API to get real asset IDs
//! 2. Subscribing to orderbook updates
//! 3. Subscribing to price changes
//! 4. Subscribing to best bid/ask updates
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example websocket_orderbook --features ws,gamma,tracing
//! ```

use std::time::Duration;

use futures::StreamExt as _;
use polymarket_client_sdk::clob::ws::Client;
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::MarketsRequest;
use tokio::time::timeout;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let gamma = GammaClient::default();

    // Fetch active markets from Gamma to get real asset IDs
    let markets_result = gamma
        .markets(&MarketsRequest::builder().closed(false).limit(5).build())
        .await;

    let asset_ids: Vec<String> = match &markets_result {
        Ok(markets) => {
            info!(endpoint = "gamma_markets", count = markets.len());
            markets
                .iter()
                .filter_map(|m| m.clob_token_ids.as_ref())
                .flatten()
                .take(4)
                .cloned()
                .collect()
        }
        Err(e) => {
            debug!(endpoint = "gamma_markets", error = %e);
            Vec::new()
        }
    };

    if asset_ids.is_empty() {
        debug!("No asset IDs available from Gamma, cannot test WebSocket streams");
        return Ok(());
    }

    info!(
        asset_ids = asset_ids.len(),
        "Using asset IDs from Gamma for WebSocket subscriptions"
    );

    let client = Client::default();

    // Show connection state
    let state = client.connection_state();
    info!(endpoint = "connection_state", state = ?state);

    // Subscribe to orderbook updates
    info!(
        stream = "orderbook",
        assets = asset_ids.len(),
        "Subscribing to orderbook updates"
    );
    match client.subscribe_orderbook(asset_ids.clone()) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut count = 0;

            while let Ok(Some(result)) = timeout(Duration::from_secs(10), stream.next()).await {
                match result {
                    Ok(book) => {
                        let best_bid = book.bids.first().map(|b| b.price);
                        let best_ask = book.asks.first().map(|a| a.price);
                        info!(
                            stream = "orderbook",
                            asset_id = %book.asset_id,
                            market = %book.market,
                            bids = book.bids.len(),
                            asks = book.asks.len(),
                            best_bid = ?best_bid,
                            best_ask = ?best_ask
                        );
                        count += 1;
                        if count >= 3 {
                            break;
                        }
                    }
                    Err(e) => debug!(stream = "orderbook", error = %e),
                }
            }
            info!(stream = "orderbook", received = count);
        }
        Err(e) => debug!(stream = "orderbook", error = %e),
    }

    // Subscribe to price changes
    info!(
        stream = "prices",
        assets = asset_ids.len(),
        "Subscribing to price changes"
    );
    match client.subscribe_prices(asset_ids.clone()) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut count = 0;

            while let Ok(Some(result)) = timeout(Duration::from_secs(10), stream.next()).await {
                match result {
                    Ok(price_change) => {
                        for entry in &price_change.price_changes {
                            info!(
                                stream = "prices",
                                market = %price_change.market,
                                asset_id = %entry.asset_id,
                                price = %entry.price,
                                side = ?entry.side
                            );
                        }
                        count += 1;
                        if count >= 3 {
                            break;
                        }
                    }
                    Err(e) => debug!(stream = "prices", error = %e),
                }
            }
            info!(stream = "prices", received = count);
        }
        Err(e) => debug!(stream = "prices", error = %e),
    }

    // Subscribe to best bid/ask updates
    info!(
        stream = "best_bid_ask",
        assets = asset_ids.len(),
        "Subscribing to best bid/ask updates"
    );
    match client.subscribe_best_bid_ask(asset_ids.clone()) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut count = 0;

            while let Ok(Some(result)) = timeout(Duration::from_secs(10), stream.next()).await {
                match result {
                    Ok(bba) => {
                        info!(
                            stream = "best_bid_ask",
                            asset_id = %bba.asset_id,
                            market = %bba.market,
                            best_bid = ?bba.best_bid,
                            best_ask = ?bba.best_ask
                        );
                        count += 1;
                        if count >= 3 {
                            break;
                        }
                    }
                    Err(e) => debug!(stream = "best_bid_ask", error = %e),
                }
            }
            info!(stream = "best_bid_ask", received = count);
        }
        Err(e) => debug!(stream = "best_bid_ask", error = %e),
    }

    // Subscribe to midpoint updates
    info!(
        stream = "midpoints",
        assets = asset_ids.len(),
        "Subscribing to midpoint updates"
    );
    match client.subscribe_midpoints(asset_ids.clone()) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut count = 0;

            while let Ok(Some(result)) = timeout(Duration::from_secs(10), stream.next()).await {
                match result {
                    Ok(midpoint) => {
                        info!(
                            stream = "midpoints",
                            asset_id = %midpoint.asset_id,
                            market = %midpoint.market,
                            midpoint = %midpoint.midpoint
                        );
                        count += 1;
                        if count >= 3 {
                            break;
                        }
                    }
                    Err(e) => debug!(stream = "midpoints", error = %e),
                }
            }
            info!(stream = "midpoints", received = count);
        }
        Err(e) => debug!(stream = "midpoints", error = %e),
    }

    // Show final subscription count
    let sub_count = client.subscription_count();
    info!(endpoint = "subscription_count", count = sub_count);

    Ok(())
}
