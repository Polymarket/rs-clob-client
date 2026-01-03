//! Comprehensive authenticated WebSocket user events explorer.
//!
//! This example demonstrates authenticated CLOB WebSocket streams by:
//! 1. Authenticating with API credentials from environment variables
//! 2. Fetching active markets from Gamma API to get real market IDs
//! 3. Subscribing to user events (orders, trades)
//!
//! Required environment variables:
//! - `POLYMARKET_API_KEY`
//! - `POLYMARKET_API_SECRET`
//! - `POLYMARKET_API_PASSPHRASE`
//! - `POLYMARKET_ADDRESS`
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example websocket_user --features ws,gamma,tracing
//! ```

use std::str::FromStr as _;
use std::time::Duration;

use futures::StreamExt as _;
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::clob::ws::{Client, WsMessage};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::MarketsRequest;
use polymarket_client_sdk::types::Address;
use tokio::time::timeout;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Load credentials from environment
    let Ok(api_key_str) = std::env::var("POLYMARKET_API_KEY") else {
        warn!("POLYMARKET_API_KEY not set, cannot run authenticated example");
        return Ok(());
    };
    let api_key = Uuid::parse_str(&api_key_str)?;

    let Ok(api_secret) = std::env::var("POLYMARKET_API_SECRET") else {
        warn!("POLYMARKET_API_SECRET not set, cannot run authenticated example");
        return Ok(());
    };

    let Ok(api_passphrase) = std::env::var("POLYMARKET_API_PASSPHRASE") else {
        warn!("POLYMARKET_API_PASSPHRASE not set, cannot run authenticated example");
        return Ok(());
    };

    let Ok(address_str) = std::env::var("POLYMARKET_ADDRESS") else {
        warn!("POLYMARKET_ADDRESS not set, cannot run authenticated example");
        return Ok(());
    };
    let address = Address::from_str(&address_str)?;

    info!(address = %address, "Credentials loaded from environment");

    let gamma = GammaClient::default();

    // Fetch active markets from Gamma to get real market IDs
    let markets_result = gamma
        .markets(&MarketsRequest::builder().closed(false).limit(5).build())
        .await;

    let market_ids: Vec<String> = match &markets_result {
        Ok(markets) => {
            info!(endpoint = "gamma_markets", count = markets.len());
            markets
                .iter()
                .filter_map(|m| m.condition_id.clone())
                .take(3)
                .collect()
        }
        Err(e) => {
            debug!(endpoint = "gamma_markets", error = %e);
            Vec::new()
        }
    };

    info!(
        market_ids = market_ids.len(),
        "Using market IDs from Gamma (empty = all markets)"
    );

    // Build credentials and authenticate
    let credentials = Credentials::new(api_key, api_secret, api_passphrase);
    let client = Client::default().authenticate(credentials, address)?;
    info!(
        endpoint = "authenticate",
        "Authenticated WebSocket client created"
    );

    // Show connection state
    let state = client.connection_state();
    info!(endpoint = "connection_state", state = ?state);

    // Subscribe to user events
    info!(
        stream = "user_events",
        markets = market_ids.len(),
        "Subscribing to user events"
    );
    match client.subscribe_user_events(market_ids) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut order_count = 0;
            let mut trade_count = 0;

            // User events may be infrequent, use longer timeout
            while let Ok(Some(result)) = timeout(Duration::from_secs(30), stream.next()).await {
                match result {
                    Ok(WsMessage::Order(order)) => {
                        info!(
                            stream = "user_events",
                            event_type = "order",
                            order_id = %order.id,
                            market = %order.market,
                            msg_type = ?order.msg_type,
                            side = ?order.side,
                            price = %order.price
                        );
                        order_count += 1;
                    }
                    Ok(WsMessage::Trade(trade)) => {
                        info!(
                            stream = "user_events",
                            event_type = "trade",
                            trade_id = %trade.id,
                            market = %trade.market,
                            status = %trade.status,
                            side = ?trade.side,
                            size = %trade.size,
                            price = %trade.price
                        );
                        trade_count += 1;
                    }
                    Ok(other) => {
                        debug!(stream = "user_events", event = ?other, "Other event received");
                    }
                    Err(e) => {
                        debug!(stream = "user_events", error = %e);
                        break;
                    }
                }

                // Stop after receiving some events for the example
                if order_count + trade_count >= 5 {
                    break;
                }
            }
            info!(
                stream = "user_events",
                orders = order_count,
                trades = trade_count,
                "Stream complete"
            );
        }
        Err(e) => debug!(stream = "user_events", error = %e),
    }

    // Show final subscription count
    let sub_count = client.subscription_count();
    info!(endpoint = "subscription_count", count = sub_count);

    Ok(())
}
