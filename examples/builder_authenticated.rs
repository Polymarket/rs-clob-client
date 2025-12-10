#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::auth::builder::Config as BuilderConfig;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::types::TradesRequestBuilder;
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need a private key");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .builder_authentication_builder(signer, BuilderConfig::local())
        .authenticate()
        .await?;

    let request = TradesRequestBuilder::default().asset_id("asset").build()?;
    println!(
        "builder_trades -- {:?}",
        client.builder_trades(&request, None).await?
    );

    Ok(())
}
