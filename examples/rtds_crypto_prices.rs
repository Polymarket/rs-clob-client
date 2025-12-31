//! Example: Subscribe to cryptocurrency prices via RTDS
//!
//! This example demonstrates how to use the RTDS client to subscribe to
//! real-time cryptocurrency price updates from Binance.
//!
//! Run with: `cargo run --example rtds_crypto_prices --features rtds`

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "Example binary uses println for output"
)]

use futures::StreamExt as _;
use polymarket_client_sdk::rtds::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a new RTDS client with default configuration
    let client = Client::default();

    println!("Subscribing to crypto prices from Binance...\n");

    // Subscribe to all crypto prices
    let stream = client.subscribe_crypto_prices(None)?;
    let mut stream = Box::pin(stream);

    // Process incoming price updates
    let mut count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(price) => {
                println!(
                    "{} @ {} (timestamp: {})",
                    price.symbol.to_uppercase(),
                    price.value,
                    price.timestamp
                );
                count += 1;

                // Stop after receiving 10 price updates for this example
                if count >= 10 {
                    println!("\nReceived 10 price updates. Stopping...");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error receiving price: {e}");
            }
        }
    }

    Ok(())
}
