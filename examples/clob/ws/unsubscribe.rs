//! Comprehensive WebSocket subscribe/unsubscribe demonstration.
//!
//! This example demonstrates CLOB WebSocket multiplexing behavior by:
//! 1. Fetching active markets from Gamma API to get real asset IDs
//! 2. Subscribing to orderbook updates (first subscription)
//! 3. Subscribing again (should multiplex, not send new message)
//! 4. Unsubscribing once (refcount decrements, no unsubscribe sent)
//! 5. Unsubscribing again (refcount 0, unsubscribe sent)
//! 6. Re-subscribing (proves unsubscribe worked)
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example websocket_unsubscribe --features ws,gamma,tracing
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
        .markets(&MarketsRequest::builder().closed(false).limit(3).build())
        .await;

    let asset_ids: Vec<String> = match &markets_result {
        Ok(markets) => {
            info!(endpoint = "gamma_markets", count = markets.len());
            markets
                .iter()
                .filter_map(|m| m.clob_token_ids.as_ref())
                .flatten()
                .take(1)
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
        "Using asset IDs from Gamma for subscription tests"
    );

    let client = Client::default();

    // Show connection state
    let state = client.connection_state();
    info!(endpoint = "connection_state", state = ?state);

    // === FIRST SUBSCRIPTION ===
    info!(
        phase = "subscribe_1",
        "First subscription (should send 'subscribe' to server)"
    );
    let stream1 = client.subscribe_orderbook(asset_ids.clone())?;
    let mut stream1 = Box::pin(stream1);

    match timeout(Duration::from_secs(10), stream1.next()).await {
        Ok(Some(Ok(book))) => {
            info!(
                phase = "subscribe_1",
                asset_id = %book.asset_id,
                bids = book.bids.len(),
                asks = book.asks.len(),
                "Received orderbook update"
            );
        }
        Ok(Some(Err(e))) => debug!(phase = "subscribe_1", error = %e),
        Ok(None) => debug!(phase = "subscribe_1", "Stream ended"),
        Err(_) => debug!(phase = "subscribe_1", "Timeout waiting for update"),
    }

    let sub_count = client.subscription_count();
    info!(phase = "subscribe_1", subscription_count = sub_count);

    // === SECOND SUBSCRIPTION (same asset - should multiplex) ===
    info!(
        phase = "subscribe_2",
        "Second subscription (should multiplex, NOT send new message)"
    );
    let stream2 = client.subscribe_orderbook(asset_ids.clone())?;
    let mut stream2 = Box::pin(stream2);

    match timeout(Duration::from_secs(10), stream2.next()).await {
        Ok(Some(Ok(book))) => {
            info!(
                phase = "subscribe_2",
                asset_id = %book.asset_id,
                bids = book.bids.len(),
                asks = book.asks.len(),
                "Stream2 received orderbook update"
            );
        }
        Ok(Some(Err(e))) => debug!(phase = "subscribe_2", error = %e),
        Ok(None) => debug!(phase = "subscribe_2", "Stream ended"),
        Err(_) => debug!(phase = "subscribe_2", "Timeout"),
    }

    let sub_count = client.subscription_count();
    info!(phase = "subscribe_2", subscription_count = sub_count);

    // === FIRST UNSUBSCRIBE ===
    info!(
        phase = "unsubscribe_1",
        "First unsubscribe (refcount still > 0, should NOT send unsubscribe)"
    );
    client.unsubscribe_orderbook(&asset_ids)?;
    drop(stream1);
    info!(phase = "unsubscribe_1", "Stream1 dropped");

    // stream2 should still work
    match timeout(Duration::from_secs(10), stream2.next()).await {
        Ok(Some(Ok(book))) => {
            info!(
                phase = "unsubscribe_1",
                asset_id = %book.asset_id,
                bids = book.bids.len(),
                asks = book.asks.len(),
                "Stream2 still receiving updates"
            );
        }
        Ok(Some(Err(e))) => debug!(phase = "unsubscribe_1", error = %e),
        Ok(None) => debug!(phase = "unsubscribe_1", "Stream ended"),
        Err(_) => debug!(phase = "unsubscribe_1", "Timeout"),
    }

    let sub_count = client.subscription_count();
    info!(phase = "unsubscribe_1", subscription_count = sub_count);

    // === SECOND UNSUBSCRIBE ===
    info!(
        phase = "unsubscribe_2",
        "Second unsubscribe (refcount 0, should send 'unsubscribe')"
    );
    client.unsubscribe_orderbook(&asset_ids)?;
    drop(stream2);
    info!(phase = "unsubscribe_2", "Stream2 dropped");

    let sub_count = client.subscription_count();
    info!(phase = "unsubscribe_2", subscription_count = sub_count);

    // === RE-SUBSCRIBE (proves unsubscribe worked) ===
    info!(
        phase = "resubscribe",
        "Re-subscribing (proves unsubscribe worked, should send new 'subscribe')"
    );
    let stream3 = client.subscribe_orderbook(asset_ids)?;
    let mut stream3 = Box::pin(stream3);

    match timeout(Duration::from_secs(10), stream3.next()).await {
        Ok(Some(Ok(book))) => {
            info!(
                phase = "resubscribe",
                asset_id = %book.asset_id,
                bids = book.bids.len(),
                asks = book.asks.len(),
                "Stream3 received orderbook update"
            );
        }
        Ok(Some(Err(e))) => debug!(phase = "resubscribe", error = %e),
        Ok(None) => debug!(phase = "resubscribe", "Stream ended"),
        Err(_) => debug!(phase = "resubscribe", "Timeout"),
    }

    let sub_count = client.subscription_count();
    info!(phase = "resubscribe", subscription_count = sub_count);

    info!(
        summary = "complete",
        "With RUST_LOG=debug, you should see multiplexing behavior in logs"
    );

    Ok(())
}
