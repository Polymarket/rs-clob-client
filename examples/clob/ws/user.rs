//! Demonstrates subscribing to authenticated user WebSocket channels.
//!
//! This example shows how to:
//! 1. Build credentials for authenticated WebSocket access
//! 2. Subscribe to user-specific order and trade events
//! 3. Process real-time order updates and trade notifications
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example websocket_user --features ws,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=websocket_user.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example websocket_user --features ws,tracing
//! ```
//!
//! Requires the following environment variables:
//! - `POLYMARKET_API_KEY`
//! - `POLYMARKET_API_SECRET`
//! - `POLYMARKET_API_PASSPHRASE`
//! - `POLYMARKET_ADDRESS`

use std::fs::File;
use std::str::FromStr as _;

use futures::StreamExt as _;
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::clob::ws::{Client, WsMessage};
use polymarket_client_sdk::types::{Address, B256};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use uuid::Uuid;

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

    let api_key = Uuid::parse_str(&std::env::var("POLYMARKET_API_KEY")?)?;
    let api_secret = std::env::var("POLYMARKET_API_SECRET")?;
    let api_passphrase = std::env::var("POLYMARKET_API_PASSPHRASE")?;
    let address = Address::from_str(&std::env::var("POLYMARKET_ADDRESS")?)?;

    let credentials = Credentials::new(api_key, api_secret, api_passphrase);

    let client = Client::default().authenticate(credentials, address)?;
    info!(
        endpoint = "websocket",
        authenticated = true,
        "connected to authenticated WebSocket"
    );

    // Provide specific market IDs, or leave empty for all events
    let markets: Vec<B256> = Vec::new();
    let mut stream = std::pin::pin!(client.subscribe_user_events(markets)?);
    info!(
        endpoint = "subscribe_user_events",
        "subscribed to user events"
    );

    while let Some(event) = stream.next().await {
        match event {
            Ok(WsMessage::Order(order)) => {
                info!(
                    endpoint = "user_events",
                    event_type = "order",
                    order_id = %order.id,
                    market = %order.market,
                    msg_type = ?order.msg_type,
                    side = ?order.side,
                    price = %order.price
                );
                if let Some(size) = &order.original_size {
                    debug!(endpoint = "user_events", original_size = %size);
                }
                if let Some(matched) = &order.size_matched {
                    debug!(endpoint = "user_events", size_matched = %matched);
                }
            }
            Ok(WsMessage::Trade(trade)) => {
                info!(
                    endpoint = "user_events",
                    event_type = "trade",
                    trade_id = %trade.id,
                    market = %trade.market,
                    status = ?trade.status,
                    side = ?trade.side,
                    size = %trade.size,
                    price = %trade.price
                );
                if let Some(trader_side) = &trade.trader_side {
                    debug!(endpoint = "user_events", trader_side = ?trader_side);
                }
            }
            Ok(other) => {
                debug!(endpoint = "user_events", event = ?other);
            }
            Err(e) => {
                error!(endpoint = "user_events", error = %e);
                break;
            }
        }
    }

    Ok(())
}
