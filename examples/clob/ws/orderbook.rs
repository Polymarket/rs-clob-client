//! Demonstrates subscribing to real-time orderbook updates via WebSocket.
//!
//! This example shows how to:
//! 1. Connect to the CLOB WebSocket API
//! 2. Subscribe to orderbook updates for multiple assets
//! 3. Process and display bid/ask updates in real-time
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example websocket_orderbook --features ws,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=websocket_orderbook.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example websocket_orderbook --features ws,tracing
//! ```

use std::fs::File;
use std::str::FromStr as _;

use futures::StreamExt as _;
use polymarket_client_sdk::clob::ws::Client;
use polymarket_client_sdk::types::U256;
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

    let asset_ids = vec![
        U256::from_str(
            "92703761682322480664976766247614127878023988651992837287050266308961660624165",
        )?,
        U256::from_str(
            "34551606549875928972193520396544368029176529083448203019529657908155427866742",
        )?,
    ];

    let stream = client.subscribe_orderbook(asset_ids.clone())?;
    let mut stream = Box::pin(stream);
    info!(
        endpoint = "subscribe_orderbook",
        asset_count = asset_ids.len(),
        "subscribed to orderbook updates"
    );

    while let Some(book_result) = stream.next().await {
        match book_result {
            Ok(book) => {
                info!(
                    endpoint = "orderbook",
                    asset_id = %book.asset_id,
                    market = %book.market,
                    timestamp = %book.timestamp,
                    bids = book.bids.len(),
                    asks = book.asks.len()
                );

                for (i, bid) in book.bids.iter().take(5).enumerate() {
                    debug!(
                        endpoint = "orderbook",
                        side = "bid",
                        rank = i + 1,
                        size = %bid.size,
                        price = %bid.price
                    );
                }

                for (i, ask) in book.asks.iter().take(5).enumerate() {
                    debug!(
                        endpoint = "orderbook",
                        side = "ask",
                        rank = i + 1,
                        size = %ask.size,
                        price = %ask.price
                    );
                }

                if let Some(hash) = &book.hash {
                    debug!(endpoint = "orderbook", hash = %hash);
                }
            }
            Err(e) => error!(endpoint = "orderbook", error = %e),
        }
    }

    Ok(())
}
