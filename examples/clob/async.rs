#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::types::U256;
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
    let client_clone = client.clone();

    let token_id = U256::from_str(
        "42334954850219754195241248003172889699504912694714162671145392673031415571339",
    )?;

    let thread = tokio::spawn(async move {
        let results = join!(
            client_clone.ok(),
            client_clone.tick_size(token_id),
            client_clone.neg_risk(token_id)
        );

        println!("In thread -- {results:?}");
        anyhow::Ok(())
    });

    println!("ok -- {:?}", client.ok().await?);
    println!("tick_size -- {:?}", client.tick_size(token_id).await?);
    println!("neg_risk -- {:?}", client.neg_risk(token_id).await?);

    thread.await?
}

async fn authenticated() -> anyhow::Result<()> {
    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need a private key");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;
    let client_clone = client.clone();

    let thread = tokio::spawn(async move {
        let results = join!(client_clone.ok(), client_clone.api_keys(),);

        println!("In thread -- {results:?}");
        anyhow::Ok(())
    });

    println!("ok -- {:?}", client.ok().await?);
    println!("api_keys -- {:?}", client.api_keys().await?);

    thread.await?
}
