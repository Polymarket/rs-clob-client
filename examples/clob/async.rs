//! Comprehensive async CLOB API explorer.
//!
//! This example demonstrates async CLOB operations by:
//! 1. Fetching active markets from Gamma API to get real token IDs
//! 2. Running unauthenticated endpoints (ok, `tick_size`, `neg_risk`)
//! 3. Running authenticated endpoints (`api_keys`) if credentials are available
//! 4. Demonstrating concurrent requests using `tokio::spawn`
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example async --features gamma,tracing
//! ```

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::MarketsRequest;
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use tokio::join;
use tracing::{debug, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let gamma = GammaClient::default();

    // Fetch active markets from Gamma to get real token IDs
    let markets_result = gamma
        .markets(&MarketsRequest::builder().closed(false).limit(5).build())
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

    if token_ids.is_empty() {
        debug!("No token IDs available from Gamma, cannot test CLOB endpoints");
        return Ok(());
    }

    info!(
        token_ids = token_ids.len(),
        "Using token IDs from Gamma for CLOB requests"
    );

    let (unauthenticated, authenticated) =
        join!(unauthenticated(&token_ids), authenticated(&token_ids));
    unauthenticated?;
    authenticated
}

async fn unauthenticated(token_ids: &[String]) -> anyhow::Result<()> {
    let client = Client::new("https://clob.polymarket.com", Config::default())?;
    let client_clone = client.clone();

    let token_id = token_ids.first().map_or_else(String::new, Clone::clone);
    let token_id_clone = token_id.clone();

    // Spawn concurrent requests in a separate task
    let thread = tokio::spawn(async move {
        let results = join!(
            client_clone.ok(),
            client_clone.tick_size(&token_id_clone),
            client_clone.neg_risk(&token_id_clone)
        );

        match &results.0 {
            Ok(ok) => info!(endpoint = "ok", thread = true, result = %ok),
            Err(e) => debug!(endpoint = "ok", thread = true, error = %e),
        }
        match &results.1 {
            Ok(tick) => {
                info!(endpoint = "tick_size", thread = true, tick_size = ?tick.minimum_tick_size);
            }
            Err(e) => debug!(endpoint = "tick_size", thread = true, error = %e),
        }
        match &results.2 {
            Ok(neg) => info!(
                endpoint = "neg_risk",
                thread = true,
                neg_risk = neg.neg_risk
            ),
            Err(e) => debug!(endpoint = "neg_risk", thread = true, error = %e),
        }

        anyhow::Ok(())
    });

    // Run requests in main task
    match client.ok().await {
        Ok(ok) => info!(endpoint = "ok", result = %ok),
        Err(e) => debug!(endpoint = "ok", error = %e),
    }

    match client.tick_size(&token_id).await {
        Ok(tick) => info!(endpoint = "tick_size", tick_size = ?tick.minimum_tick_size),
        Err(e) => debug!(endpoint = "tick_size", error = %e),
    }

    match client.neg_risk(&token_id).await {
        Ok(neg) => info!(endpoint = "neg_risk", neg_risk = neg.neg_risk),
        Err(e) => debug!(endpoint = "neg_risk", error = %e),
    }

    thread.await?
}

async fn authenticated(token_ids: &[String]) -> anyhow::Result<()> {
    let Ok(private_key) = std::env::var(PRIVATE_KEY_VAR) else {
        warn!("POLYMARKET_PRIVATE_KEY not set, skipping authenticated example");
        return Ok(());
    };

    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    info!(
        endpoint = "authenticate",
        "Authenticated CLOB client created"
    );

    let client_clone = client.clone();
    let token_id = token_ids.first().map_or_else(String::new, Clone::clone);

    // Spawn concurrent requests in a separate task
    let thread = tokio::spawn(async move {
        let results = join!(client_clone.ok(), client_clone.api_keys());

        match &results.0 {
            Ok(ok) => info!(endpoint = "ok", thread = true, authenticated = true, result = %ok),
            Err(e) => debug!(endpoint = "ok", thread = true, authenticated = true, error = %e),
        }
        match &results.1 {
            Ok(keys) => info!(endpoint = "api_keys", thread = true, keys = ?keys),
            Err(e) => debug!(endpoint = "api_keys", thread = true, error = %e),
        }

        anyhow::Ok(())
    });

    // Run requests in main task
    match client.ok().await {
        Ok(ok) => info!(endpoint = "ok", authenticated = true, result = %ok),
        Err(e) => debug!(endpoint = "ok", authenticated = true, error = %e),
    }

    match client.api_keys().await {
        Ok(keys) => info!(endpoint = "api_keys", keys = ?keys),
        Err(e) => debug!(endpoint = "api_keys", error = %e),
    }

    // Test with token ID if available
    if !token_id.is_empty() {
        match client.tick_size(&token_id).await {
            Ok(tick) => {
                info!(endpoint = "tick_size", authenticated = true, tick_size = ?tick.minimum_tick_size);
            }
            Err(e) => debug!(endpoint = "tick_size", authenticated = true, error = %e),
        }
    }

    thread.await?
}
