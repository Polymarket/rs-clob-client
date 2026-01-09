#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::auth::builder::Config as BuilderConfig;
use polymarket_client_sdk::clob::types::request::TradesRequest;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::types::U256;
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need a private key");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    // Save these credentials for subsequent calls with the builder client
    let builder_credentials = client.create_builder_api_key().await?;
    let config = BuilderConfig::local(builder_credentials);

    let client = client.promote_to_builder(config).await?;

    let keys = client.builder_api_keys().await?;
    println!("{keys:#?}");

    let token_id = U256::from_str(
        "15871154585880608648532107628464183779895785213830018178010423617714102767076",
    )?;
    let request = TradesRequest::builder().asset_id(token_id).build();
    println!(
        "builder_trades -- {:?}",
        client.builder_trades(&request, None).await?
    );

    Ok(())
}
