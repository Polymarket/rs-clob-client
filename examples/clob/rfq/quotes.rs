#![cfg(feature = "rfq")]
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::types::{RfqQuotesRequest, RfqSortBy, RfqSortDir, RfqState};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need a private key");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    let request = RfqQuotesRequest::builder()
        .state(RfqState::Active)
        .limit(10)
        .offset("MA==")
        .sort_by(RfqSortBy::Price)
        .sort_dir(RfqSortDir::Asc)
        .build();

    let quotes = client.quotes(&request, None).await?;
    println!(
        "count: {}, next_cursor: {}",
        quotes.count, quotes.next_cursor
    );
    println!("{:#?}", quotes.data);

    Ok(())
}
