#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use polymarket_client_sdk::gamma::GammaClient;
use polymarket_client_sdk::gamma::types::{ListTeamsRequest, ListTeamsRequestBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = GammaClient::default();

    println!(
        "teams default -- {:?}",
        client.teams(&ListTeamsRequest::default()).await
    );

    let filtered_request = ListTeamsRequestBuilder::default()
        .limit(5_u32)
        .offset(10_u32)
        .build()?;
    println!(
        "teams custom -- {:?}",
        client.teams(&filtered_request).await
    );

    println!("sports -- {:?}", client.sports().await);

    println!(
        "sports_market_types -- {:?}",
        client.sports_market_types().await
    );

    Ok(())
}
