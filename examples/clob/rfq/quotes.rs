//! Demonstrates fetching RFQ quotes from the CLOB API.
//!
//! This example shows how to:
//! 1. Authenticate with the CLOB API
//! 2. Build an RFQ quotes request with filters
//! 3. Fetch and display paginated quote results
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example rfq_quotes --features clob,rfq,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=rfq_quotes.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example rfq_quotes --features clob,rfq,tracing
//! ```
//!
//! Requires `POLY_PRIVATE_KEY` environment variable to be set.

#![cfg(feature = "rfq")]

use std::fs::File;
use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::types::{RfqQuotesRequest, RfqSortBy, RfqSortDir, RfqState};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
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

    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need POLY_PRIVATE_KEY");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    let request = RfqQuotesRequest::builder()
        .state(RfqState::Active)
        .limit(10)
        .offset("MA==")
        .sort_by(RfqSortBy::Price)
        .sort_dir(RfqSortDir::Asc)
        .build();

    match client.quotes(&request, None).await {
        Ok(quotes) => {
            info!(
                endpoint = "quotes",
                count = quotes.count,
                data_len = quotes.data.len(),
                next_cursor = %quotes.next_cursor
            );
            for quote in &quotes.data {
                debug!(endpoint = "quotes", quote = ?quote);
            }
        }
        Err(e) => error!(endpoint = "quotes", error = %e),
    }

    Ok(())
}
