#![cfg(feature = "rfq")]
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::types::{RfqRequestsRequest, RfqSortBy, RfqSortDir, RfqState};
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

    let request = RfqRequestsRequest::builder()
        .state(RfqState::Active)
        .limit(10)
        .offset("MA==")
        .sort_by(RfqSortBy::Created)
        .sort_dir(RfqSortDir::Desc)
        .build();

    let requests = client.requests(&request, None).await?;
    println!(
        "count: {}, next_cursor: {}",
        requests.count, requests.next_cursor
    );
    println!("{:#?}", requests.data);

    Ok(())
}
