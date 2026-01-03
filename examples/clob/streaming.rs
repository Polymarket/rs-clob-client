//! Comprehensive CLOB streaming API explorer.
//!
//! This example demonstrates CLOB streaming operations by:
//! 1. Streaming unauthenticated data (`sampling_markets`)
//! 2. Streaming authenticated data (`trades`) if credentials are available
//! 3. Using `stream_data` helper for paginated endpoints
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example streaming --features tracing
//! ```

use std::str::FromStr as _;
use std::time::Duration;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use futures::{StreamExt as _, future};
use polymarket_client_sdk::clob::types::request::TradesRequest;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use tokio::join;
use tokio::time::timeout;
use tracing::{debug, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (unauthenticated, authenticated) = join!(unauthenticated(), authenticated());
    unauthenticated?;
    authenticated
}

async fn unauthenticated() -> anyhow::Result<()> {
    let client = Client::new("https://clob.polymarket.com", Config::default())?;

    info!(
        stream = "sampling_markets",
        "Streaming unauthenticated sampling markets"
    );

    let mut stream = client
        .stream_data(Client::sampling_markets)
        .filter_map(|d| future::ready(d.ok()))
        .boxed();

    let mut count = 0;
    while let Ok(Some(market)) = timeout(Duration::from_secs(10), stream.next()).await {
        info!(
            stream = "sampling_markets",
            condition_id = %market.condition_id,
            question = %market.question,
            active = market.active,
            "Received market"
        );
        count += 1;
        if count >= 5 {
            break;
        }
    }

    info!(
        stream = "sampling_markets",
        received = count,
        "Stream complete"
    );

    Ok(())
}

async fn authenticated() -> anyhow::Result<()> {
    let Ok(private_key) = std::env::var(PRIVATE_KEY_VAR) else {
        warn!("POLYMARKET_PRIVATE_KEY not set, skipping authenticated streaming example");
        return Ok(());
    };

    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));
    info!(address = %signer.address(), "Signer created from private key");

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    info!(
        endpoint = "authenticate",
        "Authenticated CLOB client created"
    );

    info!(stream = "trades", "Streaming authenticated trades");

    let request = TradesRequest::builder().build();
    let mut stream = client
        .stream_data(|c, cursor| c.trades(&request, cursor))
        .boxed();

    let mut count = 0;
    while let Ok(Some(result)) = timeout(Duration::from_secs(10), stream.next()).await {
        match result {
            Ok(trade) => {
                info!(
                    stream = "trades",
                    id = %trade.id,
                    market = %trade.market,
                    side = ?trade.side,
                    price = %trade.price,
                    size = %trade.size,
                    "Received trade"
                );
                count += 1;
                if count >= 5 {
                    break;
                }
            }
            Err(e) => {
                debug!(stream = "trades", error = %e);
                break;
            }
        }
    }

    info!(stream = "trades", received = count, "Stream complete");

    Ok(())
}
