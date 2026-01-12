//! Demonstrates WebSocket subscribe/unsubscribe and multiplexing behavior.
//!
//! This example shows how to:
//! 1. Subscribe multiple streams to the same asset (multiplexing)
//! 2. Unsubscribe streams while others remain active
//! 3. Verify reference counting works correctly
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example websocket_unsubscribe --features ws,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=websocket_unsubscribe.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example websocket_unsubscribe --features ws,tracing
//! ```
//!
//! With debug level, you can see subscribe/unsubscribe wire messages:
//! ```sh
//! RUST_LOG=debug,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example websocket_unsubscribe --features ws,tracing
//! ```

use std::fs::File;
use std::str::FromStr as _;
use std::time::Duration;

use futures::StreamExt as _;
use polymarket_client_sdk::clob::ws::Client;
use polymarket_client_sdk::types::U256;
use tokio::time::timeout;
use tracing::{debug, error, info};
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
    info!(endpoint = "websocket", "connected to CLOB WebSocket API");

    let asset_ids = vec![U256::from_str(
        "92703761682322480664976766247614127878023988651992837287050266308961660624165",
    )?];

    // === FIRST SUBSCRIPTION ===
    info!(
        step = 1,
        "first subscription - should send 'subscribe' to server"
    );
    let stream1 = client.subscribe_orderbook(asset_ids.clone())?;
    let mut stream1 = Box::pin(stream1);

    match timeout(Duration::from_secs(10), stream1.next()).await {
        Ok(Some(Ok(book))) => {
            info!(
                step = 1,
                endpoint = "orderbook",
                bids = book.bids.len(),
                asks = book.asks.len(),
                "received update on stream1"
            );
        }
        Ok(Some(Err(e))) => error!(step = 1, error = %e),
        Ok(None) => error!(step = 1, "stream ended"),
        Err(_) => error!(step = 1, "timeout"),
    }

    // === SECOND SUBSCRIPTION (same asset - should multiplex) ===
    info!(
        step = 2,
        "second subscription (same asset) - should NOT send message (multiplexing)"
    );
    let stream2 = client.subscribe_orderbook(asset_ids.clone())?;
    let mut stream2 = Box::pin(stream2);

    match timeout(Duration::from_secs(10), stream2.next()).await {
        Ok(Some(Ok(book))) => {
            info!(
                step = 2,
                endpoint = "orderbook",
                bids = book.bids.len(),
                asks = book.asks.len(),
                "received update on stream2"
            );
        }
        Ok(Some(Err(e))) => error!(step = 2, error = %e),
        Ok(None) => error!(step = 2, "stream ended"),
        Err(_) => error!(step = 2, "timeout"),
    }

    // === FIRST UNSUBSCRIBE ===
    info!(
        step = 3,
        "first unsubscribe - should NOT send message (refcount still 1)"
    );
    client.unsubscribe_orderbook(&asset_ids)?;
    drop(stream1);
    info!(step = 3, "stream1 unsubscribed and dropped");

    // stream2 should still work
    match timeout(Duration::from_secs(10), stream2.next()).await {
        Ok(Some(Ok(book))) => {
            info!(
                step = 3,
                endpoint = "orderbook",
                bids = book.bids.len(),
                asks = book.asks.len(),
                "stream2 still receiving updates"
            );
        }
        Ok(Some(Err(e))) => error!(step = 3, error = %e),
        Ok(None) => error!(step = 3, "stream ended"),
        Err(_) => error!(step = 3, "timeout"),
    }

    // === SECOND UNSUBSCRIBE ===
    info!(
        step = 4,
        "second unsubscribe - should send 'unsubscribe' (refcount now 0)"
    );
    client.unsubscribe_orderbook(&asset_ids)?;
    drop(stream2);
    info!(step = 4, "stream2 unsubscribed and dropped");

    // === RE-SUBSCRIBE (proves unsubscribe worked) ===
    info!(
        step = 5,
        "re-subscribe - should send 'subscribe' (proves unsubscribe worked)"
    );
    let stream3 = client.subscribe_orderbook(asset_ids)?;
    let mut stream3 = Box::pin(stream3);

    match timeout(Duration::from_secs(10), stream3.next()).await {
        Ok(Some(Ok(book))) => {
            info!(
                step = 5,
                endpoint = "orderbook",
                bids = book.bids.len(),
                asks = book.asks.len(),
                "stream3 receiving updates"
            );
        }
        Ok(Some(Err(e))) => error!(step = 5, error = %e),
        Ok(None) => error!(step = 5, "stream ended"),
        Err(_) => error!(step = 5, "timeout"),
    }

    info!("example complete");
    debug!(
        "with debug logging, you should see subscribe/unsubscribe wire messages at steps 1, 4, and 5"
    );

    Ok(())
}
