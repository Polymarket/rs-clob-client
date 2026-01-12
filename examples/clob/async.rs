//! Demonstrates async concurrency patterns with the CLOB client.
//!
//! This example shows how to:
//! 1. Run multiple unauthenticated API calls concurrently
//! 2. Run multiple authenticated API calls concurrently
//! 3. Spawn background tasks that share the client
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example async --features clob,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=async.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example async --features clob,tracing
//! ```
//!
//! For authenticated endpoints, set the `POLY_PRIVATE_KEY` environment variable.

use std::fs::File;
use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::types::U256;
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use tokio::join;
use tracing::{error, info};
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
    let client_clone = client.clone();

    let token_id = U256::from_str(
        "42334954850219754195241248003172889699504912694714162671145392673031415571339",
    )?;

    let thread = tokio::spawn(async move {
        let (ok_result, tick_result, neg_risk_result) = join!(
            client_clone.ok(),
            client_clone.tick_size(token_id),
            client_clone.neg_risk(token_id)
        );

        match ok_result {
            Ok(s) => info!(endpoint = "ok", thread = true, result = %s),
            Err(e) => error!(endpoint = "ok", thread = true, error = %e),
        }

        match tick_result {
            Ok(t) => info!(endpoint = "tick_size", thread = true, tick_size = ?t.minimum_tick_size),
            Err(e) => error!(endpoint = "tick_size", thread = true, error = %e),
        }

        match neg_risk_result {
            Ok(n) => info!(endpoint = "neg_risk", thread = true, neg_risk = n.neg_risk),
            Err(e) => error!(endpoint = "neg_risk", thread = true, error = %e),
        }

        anyhow::Ok(())
    });

    match client.ok().await {
        Ok(s) => info!(endpoint = "ok", result = %s),
        Err(e) => error!(endpoint = "ok", error = %e),
    }

    match client.tick_size(token_id).await {
        Ok(t) => {
            info!(endpoint = "tick_size", token_id = %token_id, tick_size = ?t.minimum_tick_size);
        }
        Err(e) => error!(endpoint = "tick_size", token_id = %token_id, error = %e),
    }

    match client.neg_risk(token_id).await {
        Ok(n) => info!(endpoint = "neg_risk", token_id = %token_id, neg_risk = n.neg_risk),
        Err(e) => error!(endpoint = "neg_risk", token_id = %token_id, error = %e),
    }

    thread.await?
}

async fn authenticated() -> anyhow::Result<()> {
    let Ok(private_key) = std::env::var(PRIVATE_KEY_VAR) else {
        info!(
            endpoint = "authenticated",
            "skipped - POLY_PRIVATE_KEY not set"
        );
        return Ok(());
    };
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;
    let client_clone = client.clone();

    let thread = tokio::spawn(async move {
        let (ok_result, api_keys_result) = join!(client_clone.ok(), client_clone.api_keys());

        match ok_result {
            Ok(s) => info!(endpoint = "ok", thread = true, authenticated = true, result = %s),
            Err(e) => error!(endpoint = "ok", thread = true, authenticated = true, error = %e),
        }

        match api_keys_result {
            Ok(keys) => info!(endpoint = "api_keys", thread = true, result = ?keys),
            Err(e) => error!(endpoint = "api_keys", thread = true, error = %e),
        }

        anyhow::Ok(())
    });

    match client.ok().await {
        Ok(s) => info!(endpoint = "ok", authenticated = true, result = %s),
        Err(e) => error!(endpoint = "ok", authenticated = true, error = %e),
    }

    match client.api_keys().await {
        Ok(keys) => info!(endpoint = "api_keys", result = ?keys),
        Err(e) => error!(endpoint = "api_keys", error = %e),
    }

    thread.await?
}
