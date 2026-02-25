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

use alloy::primitives::U256;
use polymarket_client_sdk::bridge::Client;
use polymarket_client_sdk::bridge::types::{
    DepositRequest, QuoteRequest, StatusRequest, WithdrawRequest,
};
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

    let status_request = StatusRequest::builder()
        .address("bc1qs82vw5pczv9uj44n4npscldkdjgfjqu7x9mlna")
        .build();

    match client.status(&status_request).await {
        Ok(response) => {
            info!(endpoint = "status", count = response.transactions.len());
        }
        Err(e) => debug!(endpoint = "status", error = %e),
    }

    let quote_request = QuoteRequest::builder()
        .from_amount_base_unit(U256::from(100_000_000))
        .from_chain_id(1)
        .from_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
        .recipient_address("0x0000000000000000000000000000000000000000")
        .to_chain_id(10)
        .to_token_address("0x7F5c764cBc14f9669B88837ca1490cCa17c31607")
        .build();

    match client.quote(&quote_request).await {
        Ok(response) => {
            info!(
                endpoint = "quote",
                quote_id = %response.quote_id,
                input_usd = response.est_input_usd,
                output_usd = response.est_output_usd,
                checkout_time_ms = response.est_checkout_time_ms,
            );
        }
        Err(e) => debug!(endpoint = "quote", error = %e),
    }

    let withdraw_request = WithdrawRequest::builder()
        .address(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
        .to_chain_id(1)
        .to_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
        .recipient_addr("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .build();

    match client.withdraw(&withdraw_request).await {
        Ok(response) => {
            info!(
                endpoint = "withdraw",
                evm = %response.address.evm,
                svm = %response.address.svm,
                btc = %response.address.btc,
                note = %response.note,
            );
        }
        Err(e) => debug!(endpoint = "withdraw", error = %e),
    }

    Ok(())
}
