#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use polymarket_client_sdk::bridge::Client;
use polymarket_client_sdk::bridge::types::DepositRequest;
use polymarket_client_sdk::types::address;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    // Get supported assets
    println!("=== Supported Assets ===");
    let response = client.supported_assets().await?;
    for asset in response.supported_assets {
        println!(
            "{} ({}) on {} [chain_id: {}] - min: ${:.2}",
            asset.token.name,
            asset.token.symbol,
            asset.chain_name,
            asset.chain_id,
            asset.min_checkout_usd
        );
    }

    // Get deposit addresses
    println!("\n=== Deposit Addresses ===");
    let request = DepositRequest::builder()
        .address(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
        .build();

    let response = client.deposit(&request).await?;
    println!("EVM Address: {}", response.address.evm);
    println!("SVM Address: {}", response.address.svm);
    println!("BTC Address: {}", response.address.btc);
    if let Some(note) = response.note {
        println!("Note: {note}");
    }

    Ok(())
}
