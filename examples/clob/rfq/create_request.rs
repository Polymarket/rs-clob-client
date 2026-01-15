//! Demonstrates creating an RFQ request.
//!
//! Note: This SDK's RFQ create endpoints take *assets + base-unit amounts* (6 decimals),
//! not `{ tokenID, side, size, price }` like the TS examples.
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example rfq_create_request --features clob,rfq,tracing
//! ```
//!
//! Requires `POLY_PRIVATE_KEY` environment variable to be set.
//!
//! Required environment variables:
//! - `RFQ_TOKEN_ID`: outcome token id (integer string)
//! - `RFQ_SIDE`: `BUY` or `SELL`
//! - `RFQ_SIZE`: token size in human units (e.g. `40`)
//! - `RFQ_PRICE`: price (e.g. `0.50`)
//!
//! Optional environment variables:
//! - `HOST` (default: <https://clob.polymarket.com>)
//! - `POLY_CHAIN_ID` (default: 137)

#![cfg(feature = "rfq")]

use std::str::FromStr as _;

use alloy::primitives::U256;
use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::types::request::{Asset, CreateRfqRequestRequest};
use polymarket_client_sdk::clob::types::{Side, SignatureType};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use rust_decimal::Decimal;
use tracing::{error, info};

const BASE_DECIMALS: u64 = 1_000_000;

fn to_base_units(amount: Decimal) -> Decimal {
    // The API expects integer base-units represented as a decimal string.
    (amount * Decimal::from(BASE_DECIMALS)).trunc()
}

fn side_from_env() -> anyhow::Result<Side> {
    let raw = std::env::var("RFQ_SIDE").expect("Need RFQ_SIDE");
    match raw.trim().to_ascii_uppercase().as_str() {
        "BUY" => Ok(Side::Buy),
        "SELL" => Ok(Side::Sell),
        _ => anyhow::bail!("RFQ_SIDE must be BUY or SELL (got {raw})"),
    }
}

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

    let token_id = U256::from_str(&std::env::var("RFQ_TOKEN_ID").expect("Need RFQ_TOKEN_ID"))?;
    let side = side_from_env()?;
    let size = Decimal::from_str(&std::env::var("RFQ_SIZE").expect("Need RFQ_SIZE"))?;
    let price = Decimal::from_str(&std::env::var("RFQ_PRICE").expect("Need RFQ_PRICE"))?;

    // BUY: receive tokens, give USDC
    // SELL: receive USDC, give tokens
    let (asset_in, asset_out, amount_in, amount_out) = match side {
        Side::Buy => (
            Asset::Asset(token_id),
            Asset::Usdc,
            to_base_units(size),
            to_base_units(size * price),
        ),
        Side::Sell => (
            Asset::Usdc,
            Asset::Asset(token_id),
            to_base_units(size * price),
            to_base_units(size),
        ),
        other => anyhow::bail!("RFQ_SIDE must be BUY or SELL, got {other:?}"),
    };

    let request = CreateRfqRequestRequest::builder()
        .asset_in(asset_in)
        .asset_out(asset_out)
        .amount_in(amount_in)
        .amount_out(amount_out)
        .user_type(SignatureType::Eoa)
        .build();

    match client.create_request(&request).await {
        Ok(resp) => info!(endpoint = "create_request", response = ?resp),
        Err(e) => error!(endpoint = "create_request", error = %e),
    }

    Ok(())
}
