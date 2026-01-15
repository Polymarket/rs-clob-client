//! Demonstrates fetching the RFQ configuration.
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example rfq_config --features clob,rfq,tracing
//! ```
//!
//! Requires `POLY_PRIVATE_KEY` environment variable to be set.
//!
//! Optional environment variables:
//! - `HOST` (default: <https://clob.polymarket.com>)
//! - `POLY_CHAIN_ID` (default: 137)

#![cfg(feature = "rfq")]

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need POLY_PRIVATE_KEY");
    let signer = LocalSigner::from_str(&private_key)?;

    let host = std::env::var("HOST").unwrap_or_else(|_| "https://clob.polymarket.com".to_owned());
    let chain_id = std::env::var("POLY_CHAIN_ID")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(POLYGON);
    let signer = signer.with_chain_id(Some(chain_id));

    let client = Client::new(&host, Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    match client.rfq_config().await {
        Ok(cfg) => info!(endpoint = "rfq_config", config = %cfg),
        Err(e) => error!(endpoint = "rfq_config", error = %e),
    }

    Ok(())
}
