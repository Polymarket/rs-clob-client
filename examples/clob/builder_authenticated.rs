//! Comprehensive builder-authenticated CLOB API explorer.
//!
//! This example demonstrates builder API authentication by:
//! 1. Authenticating with a private key
//! 2. Creating builder API credentials
//! 3. Promoting the client to a builder client
//! 4. Fetching active markets from Gamma API to get real asset IDs
//! 5. Running builder endpoints (`builder_api_keys`, `builder_trades`)
//!
//! Required environment variables:
//! - `POLYMARKET_PRIVATE_KEY` - Your wallet private key
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example builder_authenticated --features gamma,tracing
//! ```

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::auth::builder::Config as BuilderConfig;
use polymarket_client_sdk::clob::types::request::TradesRequest;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::MarketsRequest;
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use tracing::{debug, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let Ok(private_key) = std::env::var(PRIVATE_KEY_VAR) else {
        warn!("POLYMARKET_PRIVATE_KEY not set, cannot run builder authenticated example");
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

    // Create builder credentials and promote to builder client
    let builder_credentials = client.create_builder_api_key().await?;
    info!(
        endpoint = "create_builder_api_key",
        "Builder credentials created"
    );

    let config = BuilderConfig::local(builder_credentials);
    let client = client.promote_to_builder(config)?;
    info!(
        endpoint = "promote_to_builder",
        "Client promoted to builder"
    );

    // Test builder API keys endpoint
    match client.builder_api_keys().await {
        Ok(keys) => info!(endpoint = "builder_api_keys", keys = ?keys),
        Err(e) => debug!(endpoint = "builder_api_keys", error = %e),
    }

    // Fetch active markets from Gamma to get real asset IDs
    let gamma = GammaClient::default();
    let markets_result = gamma
        .markets(&MarketsRequest::builder().closed(false).limit(3).build())
        .await;

    let token_ids: Vec<String> = match &markets_result {
        Ok(markets) => {
            info!(endpoint = "gamma_markets", count = markets.len());
            markets
                .iter()
                .filter_map(|m| m.clob_token_ids.as_ref())
                .flatten()
                .take(2)
                .cloned()
                .collect()
        }
        Err(e) => {
            debug!(endpoint = "gamma_markets", error = %e);
            Vec::new()
        }
    };

    // Test builder trades endpoint with real asset ID
    if let Some(token_id) = token_ids.first() {
        info!(
            token_id = %token_id,
            "Testing builder trades with token ID from Gamma"
        );

        let request = TradesRequest::builder().asset_id(token_id).build();
        match client.builder_trades(&request, None).await {
            Ok(trades) => info!(endpoint = "builder_trades", count = trades.data.len()),
            Err(e) => debug!(endpoint = "builder_trades", error = %e),
        }
    } else {
        debug!("No token IDs available from Gamma, skipping builder_trades endpoint");
    }

    Ok(())
}
