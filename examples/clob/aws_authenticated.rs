//! Demonstrates AWS KMS-based authentication with the CLOB client.
//!
//! This example shows how to:
//! 1. Configure AWS SDK and KMS client
//! 2. Create an `AwsSigner` using a KMS key
//! 3. Authenticate with the CLOB API using the AWS signer
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example aws_authenticated --features clob,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=aws_authenticated.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example aws_authenticated --features clob,tracing
//! ```
//!
//! Requires AWS credentials configured and a valid KMS key ID.

use std::fs::File;

use alloy::signers::Signer as _;
use alloy::signers::aws::AwsSigner;
use aws_config::BehaviorVersion;
use polymarket_client_sdk::POLYGON;
use polymarket_client_sdk::clob::{Client, Config};
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

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let kms_client = aws_sdk_kms::Client::new(&config);

    let key_id = "<your key ID>".to_owned();
    info!(endpoint = "aws_signer", key_id = %key_id, "creating AWS KMS signer");

    let alloy_signer = AwsSigner::new(kms_client, key_id, Some(POLYGON))
        .await?
        .with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&alloy_signer)
        .authenticate()
        .await?;

    match client.api_keys().await {
        Ok(keys) => info!(endpoint = "api_keys", result = ?keys),
        Err(e) => error!(endpoint = "api_keys", error = %e),
    }

    Ok(())
}
