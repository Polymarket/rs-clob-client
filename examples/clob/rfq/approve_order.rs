//! Demonstrates approving an RFQ order (quoter-side last-look approval).
//!
//! Like `rfq_accept_quote`, this endpoint requires a fully-specified, signed order payload.
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example rfq_approve_order --features clob,rfq,tracing
//! ```
//!
//! Requires `POLY_PRIVATE_KEY` environment variable to be set.
//!
//! Required environment variables:
//! - `RFQ_REQUEST_ID`
//! - `RFQ_QUOTE_ID`
//! - `RFQ_MAKER_AMOUNT` (base units, integer string, 6 decimals)
//! - `RFQ_TAKER_AMOUNT` (base units, integer string, 6 decimals)
//! - `RFQ_TOKEN_ID` (integer string)
//! - `RFQ_MAKER` (0x...)
//! - `RFQ_SIGNER` (0x...)
//! - `RFQ_TAKER` (0x...)
//! - `RFQ_NONCE` (u64)
//! - `RFQ_EXPIRATION` (unix seconds, i64)
//! - `RFQ_SIDE` (`BUY` or `SELL`)
//! - `RFQ_FEE_RATE_BPS` (u64)
//! - `RFQ_SIGNATURE_TYPE` (`0` = EOA, `1` = Proxy, `2` = GnosisSafe)
//! - `RFQ_SIGNATURE` (0x...)
//! - `RFQ_SALT` (u64)
//! - `RFQ_OWNER_API_KEY` (uuid)
//!
//! Optional environment variables:
//! - `HOST` (default: <https://clob.polymarket.com>)
//! - `POLY_CHAIN_ID` (default: 137)

#![cfg(feature = "rfq")]

use std::str::FromStr as _;

use alloy::primitives::U256;
use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::auth::ApiKey;
use polymarket_client_sdk::clob::types::request::ApproveRfqOrderRequest;
use polymarket_client_sdk::clob::types::{Side, SignatureType};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::types::Address;
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use rust_decimal::Decimal;
use tracing::{error, info};

fn side_from_env() -> anyhow::Result<Side> {
    let raw = std::env::var("RFQ_SIDE").expect("Need RFQ_SIDE");
    match raw.trim().to_ascii_uppercase().as_str() {
        "BUY" => Ok(Side::Buy),
        "SELL" => Ok(Side::Sell),
        _ => anyhow::bail!("RFQ_SIDE must be BUY or SELL (got {raw})"),
    }
}

fn sig_type_from_env() -> anyhow::Result<SignatureType> {
    let raw = std::env::var("RFQ_SIGNATURE_TYPE").expect("Need RFQ_SIGNATURE_TYPE");
    match raw.as_str() {
        "0" => Ok(SignatureType::Eoa),
        "1" => Ok(SignatureType::Proxy),
        "2" => Ok(SignatureType::GnosisSafe),
        _ => anyhow::bail!("RFQ_SIGNATURE_TYPE must be 0, 1, or 2 (got {raw})"),
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

    let request = ApproveRfqOrderRequest::builder()
        .request_id(std::env::var("RFQ_REQUEST_ID").expect("Need RFQ_REQUEST_ID"))
        .quote_id(std::env::var("RFQ_QUOTE_ID").expect("Need RFQ_QUOTE_ID"))
        .maker_amount(Decimal::from_str(
            &std::env::var("RFQ_MAKER_AMOUNT").expect("Need RFQ_MAKER_AMOUNT"),
        )?)
        .taker_amount(Decimal::from_str(
            &std::env::var("RFQ_TAKER_AMOUNT").expect("Need RFQ_TAKER_AMOUNT"),
        )?)
        .token_id(U256::from_str(&std::env::var("RFQ_TOKEN_ID").expect("Need RFQ_TOKEN_ID"))?)
        .maker(Address::from_str(&std::env::var("RFQ_MAKER").expect("Need RFQ_MAKER"))?)
        .signer(Address::from_str(&std::env::var("RFQ_SIGNER").expect("Need RFQ_SIGNER"))?)
        .taker(Address::from_str(&std::env::var("RFQ_TAKER").expect("Need RFQ_TAKER"))?)
        .nonce(
            std::env::var("RFQ_NONCE")
                .expect("Need RFQ_NONCE")
                .parse::<u64>()?,
        )
        .expiration(
            std::env::var("RFQ_EXPIRATION")
                .expect("Need RFQ_EXPIRATION")
                .parse::<i64>()?,
        )
        .side(side_from_env()?)
        .fee_rate_bps(
            std::env::var("RFQ_FEE_RATE_BPS")
                .expect("Need RFQ_FEE_RATE_BPS")
                .parse::<u64>()?,
        )
        .signature_type(sig_type_from_env()?)
        .signature(std::env::var("RFQ_SIGNATURE").expect("Need RFQ_SIGNATURE"))
        .salt(std::env::var("RFQ_SALT").expect("Need RFQ_SALT").parse::<u64>()?)
        .owner(ApiKey::parse_str(
            &std::env::var("RFQ_OWNER_API_KEY").expect("Need RFQ_OWNER_API_KEY"),
        )?)
        .build();

    match client.approve_order(&request).await {
        Ok(resp) => info!(endpoint = "approve_order", trade_ids = ?resp.trade_ids),
        Err(e) => error!(endpoint = "approve_order", error = %e),
    }

    Ok(())
}
