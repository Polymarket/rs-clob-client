//! CLOB API streaming endpoint explorer.
//!
//! This example demonstrates streaming data from CLOB API endpoints by:
//! 1. Streaming `sampling_markets` (unauthenticated) to discover market data
//! 2. Streaming trades (authenticated) if credentials are available
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example streaming --features tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=streaming.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example streaming --features tracing
//! ```
//!
//! For authenticated streaming, set the `POLY_PRIVATE_KEY` environment variable:
//! ```sh
//! POLY_PRIVATE_KEY=0x... RUST_LOG=info cargo run --example streaming --features tracing
//! ```

use std::fs::File;
use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use futures::{StreamExt as _, future};
use polymarket_client_sdk::clob::types::request::TradesRequest;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use tokio::join;
use tracing::{debug, info, warn};
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

    let (unauthenticated, authenticated) = join!(unauthenticated(), authenticated());
    unauthenticated?;
    authenticated
}

async fn unauthenticated() -> anyhow::Result<()> {
    let client = Client::new("https://clob.polymarket.com", Config::default())?;

    info!(
        stream = "sampling_markets",
        "starting unauthenticated stream"
    );

    let mut stream = client
        .stream_data(Client::sampling_markets)
        .filter_map(|d| future::ready(d.ok()))
        .boxed();

    let mut count = 0_u32;

    while let Some(market) = stream.next().await {
        count += 1;

        // Log every 100th market to avoid flooding logs
        if count % 100 == 1 {
            if let Some(cid) = &market.condition_id {
                info!(
                    stream = "sampling_markets",
                    count = count,
                    condition_id = %cid,
                    question = %market.question,
                    active = market.active
                );
            } else {
                info!(
                    stream = "sampling_markets",
                    count = count,
                    question = %market.question,
                    active = market.active
                );
            }
        }
    }

    info!(
        stream = "sampling_markets",
        total_markets = count,
        "stream completed"
    );

    Ok(())
}

async fn authenticated() -> anyhow::Result<()> {
    let Ok(private_key) = std::env::var(PRIVATE_KEY_VAR) else {
        warn!(
            stream = "trades",
            "skipping authenticated stream - {} not set", PRIVATE_KEY_VAR
        );
        return Ok(());
    };

    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    info!(stream = "trades", "starting authenticated stream");

    let request = TradesRequest::builder().build();
    let mut stream = client
        .stream_data(|c, cursor| c.trades(&request, cursor))
        .boxed();

    let mut count = 0_u32;

    while let Some(result) = stream.next().await {
        match result {
            Ok(trade) => {
                count += 1;

                // Log every 100th trade to avoid flooding logs
                if count % 100 == 1 {
                    info!(
                        stream = "trades",
                        count = count,
                        market = %trade.market,
                        side = ?trade.side,
                        size = %trade.size,
                        price = %trade.price
                    );
                }
            }
            Err(e) => {
                debug!(stream = "trades", error = %e, "stream error");
            }
        }
    }

    info!(stream = "trades", total_trades = count, "stream completed");

    Ok(())
}
