//! Comprehensive AWS KMS authenticated CLOB API explorer.
//!
//! This example demonstrates AWS KMS signer authentication by:
//! 1. Loading AWS credentials from environment/config
//! 2. Creating an AWS KMS signer for transaction signing
//! 3. Authenticating with the CLOB API
//! 4. Fetching active markets from Gamma API to get real token IDs
//! 5. Running authenticated endpoints (`api_keys`, `tick_size`)
//!
//! Required environment variables:
//! - `AWS_KMS_KEY_ID` - Your AWS KMS key ID
//! - Standard AWS credentials (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, etc.)
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example aws_authenticated --features gamma,tracing
//! ```

use alloy::signers::Signer as _;
use alloy::signers::aws::AwsSigner;
use aws_config::BehaviorVersion;
use polymarket_client_sdk::POLYGON;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::MarketsRequest;
use tracing::{debug, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let Ok(key_id) = std::env::var("AWS_KMS_KEY_ID") else {
        warn!("AWS_KMS_KEY_ID not set, cannot run AWS authenticated example");
        return Ok(());
    };

    info!(key_id = %key_id, "Loading AWS KMS signer");

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let kms_client = aws_sdk_kms::Client::new(&config);

    let alloy_signer = AwsSigner::new(kms_client, key_id, Some(POLYGON))
        .await?
        .with_chain_id(Some(POLYGON));

    info!(
        address = %alloy_signer.address(),
        "AWS KMS signer created"
    );

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&alloy_signer)
        .authenticate()
        .await?;

    info!(
        endpoint = "authenticate",
        "Authenticated CLOB client created with AWS KMS"
    );

    // Test authenticated endpoint
    match client.api_keys().await {
        Ok(keys) => info!(endpoint = "api_keys", keys = ?keys),
        Err(e) => debug!(endpoint = "api_keys", error = %e),
    }

    // Fetch active markets from Gamma to get real token IDs
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

    // Test with token ID if available
    if let Some(token_id) = token_ids.first() {
        info!(
            token_id = %token_id,
            "Testing endpoints with token ID from Gamma"
        );

        match client.tick_size(token_id).await {
            Ok(tick) => {
                info!(endpoint = "tick_size", tick_size = ?tick.minimum_tick_size);
            }
            Err(e) => debug!(endpoint = "tick_size", error = %e),
        }

        match client.neg_risk(token_id).await {
            Ok(neg) => info!(endpoint = "neg_risk", neg_risk = neg.neg_risk),
            Err(e) => debug!(endpoint = "neg_risk", error = %e),
        }
    } else {
        debug!("No token IDs available from Gamma, skipping token-specific endpoints");
    }

    Ok(())
}
