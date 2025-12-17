#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use alloy::signers::Signer as _;
use alloy::signers::aws::AwsSigner;
use aws_config::BehaviorVersion;
use polymarket_client_sdk::POLYGON;
use polymarket_client_sdk::clob::{Client, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_kms::Client::new(&config);

    let key_id = "<your key ID>".to_owned();
    let alloy_signer = AwsSigner::new(client, key_id, Some(POLYGON))
        .await?
        .with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&alloy_signer)
        .authenticate()
        .await?;

    println!("api_keys -- {:?}", client.api_keys().await?);

    Ok(())
}
