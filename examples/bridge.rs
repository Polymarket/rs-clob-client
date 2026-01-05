//! Bridge API example demonstrating deposit and supported assets endpoints.
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example bridge --features bridge,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=bridge.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example bridge --features bridge,tracing
//! ```

use std::fs::File;

use polymarket_client_sdk::bridge::Client;
use polymarket_client_sdk::bridge::types::DepositRequest;
use polymarket_client_sdk::types::address;
use tracing::{debug, info};
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

    match client.supported_assets().await {
        Ok(response) => {
            info!(
                endpoint = "supported_assets",
                count = response.supported_assets.len()
            );
            for asset in &response.supported_assets {
                info!(
                    endpoint = "supported_assets",
                    name = %asset.token.name,
                    symbol = %asset.token.symbol,
                    chain = %asset.chain_name,
                    chain_id = asset.chain_id,
                    min_usd = %asset.min_checkout_usd
                );
            }
        }
        Err(e) => debug!(endpoint = "supported_assets", error = %e),
    }

    let request = DepositRequest::builder()
        .address(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
        .build();

    match client.deposit(&request).await {
        Ok(response) => {
            info!(
                endpoint = "deposit",
                evm = %response.address.evm,
                svm = %response.address.svm,
                btc = %response.address.btc,
                note = ?response.note
            );
        }
        Err(e) => debug!(endpoint = "deposit", error = %e),
    }

    Ok(())
}
