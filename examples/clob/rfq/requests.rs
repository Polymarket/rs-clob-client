//! Demonstrates fetching RFQ requests from the CLOB API.
//!
//! This example shows how to:
//! 1. Authenticate with the CLOB API
//! 2. Build an RFQ requests query with filters
//! 3. Fetch and display paginated request results
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example rfq_requests --features clob,rfq,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=rfq_requests.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example rfq_requests --features clob,rfq,tracing
//! ```
//!
//! Requires `POLY_PRIVATE_KEY` environment variable to be set.

#![cfg(feature = "rfq")]

use std::fs::File;
use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::types::{RfqRequestsRequest, RfqSortBy, RfqSortDir, RfqState};
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

    let request = RfqRequestsRequest::builder()
        .state(RfqState::Active)
        .limit(10)
        .offset("MA==")
        .sort_by(RfqSortBy::Created)
        .sort_dir(RfqSortDir::Desc)
        .build();

    match client.requests(&request, None).await {
        Ok(requests) => {
            info!(
                endpoint = "requests",
                count = requests.count,
                data_len = requests.data.len(),
                next_cursor = %requests.next_cursor
            );
            for req in &requests.data {
                debug!(endpoint = "requests", request = ?req);
            }
        }
        Err(e) => error!(endpoint = "requests", error = %e),
    }

    Ok(())
}
