#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use futures::{StreamExt as _, future};
use polymarket_client_sdk::clob::types::TradesRequestBuilder;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use tokio::join;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (unauthenticated, authenticated) = join!(unauthenticated(), authenticated());
    unauthenticated?;
    authenticated
}

async fn unauthenticated() -> anyhow::Result<()> {
    let client = Client::new("https://clob.polymarket.com", Config::default())?;

    let mut stream = client
        .stream_data(Client::sampling_markets)
        .filter_map(|d| future::ready(d.ok()))
        .boxed();

    while let Some(resp) = stream.next().await {
        println!("sampling_markets -- {resp:?}");
    }

    Ok(())
}

async fn authenticated() -> anyhow::Result<()> {
    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need a private key");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    let request = TradesRequestBuilder::default().build()?;
    let mut stream = client
        .stream_data(|c, cursor| c.trades(&request, cursor))
        .boxed();

    while let Some(resp) = stream.next().await {
        println!("trades -- {resp:?}");
    }

    Ok(())
}
