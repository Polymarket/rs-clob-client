//! Example of subscribing to real-time orderbook updates via WebSocket.
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]
#![allow(clippy::print_stderr, reason = "Examples are okay to print to stderr")]

use futures::StreamExt as _;
use polymarket_client_sdk::clob::ws::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create WebSocket client for CLOB endpoint
    let client = Client::default();

    println!("Connected to CLOB WebSocket API");

    // Subscribe to orderbook updates
    let asset_ids = vec![
        "92703761682322480664976766247614127878023988651992837287050266308961660624165".to_owned(),
        "34551606549875928972193520396544368029176529083448203019529657908155427866742".to_owned(),
    ];

    let stream = client.subscribe_orderbook(asset_ids)?;
    let mut stream = Box::pin(stream);

    // Process orderbook updates
    while let Some(book_result) = stream.next().await {
        match book_result {
            Ok(book) => {
                println!("\n--- Orderbook Update ---");
                println!("Asset ID: {}", book.asset_id);
                println!("Market: {}", book.market);
                println!("Timestamp: {}", book.timestamp);

                if !book.bids.is_empty() {
                    println!("\nTop 5 Bids:");
                    for (i, bid) in book.bids.iter().take(5).enumerate() {
                        println!("  {}: {} @ {}", i + 1, bid.size, bid.price);
                    }
                }

                if !book.asks.is_empty() {
                    println!("\nTop 5 Asks:");
                    for (i, ask) in book.asks.iter().take(5).enumerate() {
                        println!("  {}: {} @ {}", i + 1, ask.size, ask.price);
                    }
                }

                if let Some(hash) = &book.hash {
                    println!("\nBook Hash: {hash}");
                }
            }
            Err(e) => {
                eprintln!("Error receiving orderbook update: {e}");
            }
        }
    }

    Ok(())
}
