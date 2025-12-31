//! Example demonstrating WebSocket subscribe/unsubscribe functionality.
//!
//! Run with tracing enabled to see subscribe/unsubscribe messages:
//! ```
//! RUST_LOG=debug cargo run --example websocket_unsubscribe --features ws,tracing
//! ```
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]
#![allow(clippy::print_stderr, reason = "Examples are okay to print to stderr")]
#![allow(clippy::unwrap_used, reason = "Examples use unwrap for brevity")]

use std::time::Duration;

use futures::StreamExt as _;
use polymarket_client_sdk::clob::ws::Client;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber if tracing feature is enabled
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("polymarket_client_sdk=debug".parse().unwrap()),
        )
        .init();

    // Create WebSocket client for CLOB endpoint
    let client = Client::default();

    println!("Connected to CLOB WebSocket API");
    println!("(Run with RUST_LOG=debug and --features tracing to see wire messages)\n");

    // Asset IDs to subscribe to
    let asset_ids = vec![
        "92703761682322480664976766247614127878023988651992837287050266308961660624165".to_owned(),
    ];

    // === FIRST SUBSCRIPTION ===
    println!("=== [1] First subscription ===");
    println!("Subscribing to asset (should send 'subscribe' to server)...");
    let stream1 = client.subscribe_orderbook(asset_ids.clone())?;
    let mut stream1 = Box::pin(stream1);

    // Wait for one update
    match timeout(Duration::from_secs(10), stream1.next()).await {
        Ok(Some(Ok(book))) => {
            println!(
                "Got orderbook update: {} bids, {} asks",
                book.bids.len(),
                book.asks.len()
            );
        }
        Ok(Some(Err(e))) => eprintln!("Error: {e}"),
        Ok(None) => println!("Stream ended"),
        Err(_) => println!("Timeout"),
    }

    // === SECOND SUBSCRIPTION (same asset - should multiplex) ===
    println!("\n=== [2] Second subscription (same asset) ===");
    println!("Subscribing again (should NOT send message - multiplexing)...");
    let stream2 = client.subscribe_orderbook(asset_ids.clone())?;
    let mut stream2 = Box::pin(stream2);

    // Wait for one update on stream2
    match timeout(Duration::from_secs(10), stream2.next()).await {
        Ok(Some(Ok(book))) => {
            println!(
                "Stream2 got orderbook update: {} bids, {} asks",
                book.bids.len(),
                book.asks.len()
            );
        }
        Ok(Some(Err(e))) => eprintln!("Error: {e}"),
        Ok(None) => println!("Stream ended"),
        Err(_) => println!("Timeout"),
    }

    // === FIRST UNSUBSCRIBE ===
    println!("\n=== [3] First unsubscribe ===");
    println!("Unsubscribing stream1 (should NOT send message - refcount still 1)...");
    client.unsubscribe_orderbook(&asset_ids)?;
    drop(stream1);
    println!("Stream1 unsubscribed and dropped");

    // stream2 should still work
    println!("Stream2 should still receive updates...");
    match timeout(Duration::from_secs(10), stream2.next()).await {
        Ok(Some(Ok(book))) => {
            println!(
                "Stream2 still works: {} bids, {} asks",
                book.bids.len(),
                book.asks.len()
            );
        }
        Ok(Some(Err(e))) => eprintln!("Error: {e}"),
        Ok(None) => println!("Stream ended"),
        Err(_) => println!("Timeout"),
    }

    // === SECOND UNSUBSCRIBE ===
    println!("\n=== [4] Second unsubscribe ===");
    println!("Unsubscribing stream2 (should send 'unsubscribe' - refcount now 0)...");
    client.unsubscribe_orderbook(&asset_ids)?;
    drop(stream2);
    println!("Stream2 unsubscribed and dropped");

    // === RE-SUBSCRIBE (proves unsubscribe worked) ===
    println!("\n=== [5] Re-subscribe (proves unsubscribe worked) ===");
    println!("Subscribing again (should send 'subscribe' since we fully unsubscribed)...");
    let stream3 = client.subscribe_orderbook(asset_ids)?;
    let mut stream3 = Box::pin(stream3);

    match timeout(Duration::from_secs(10), stream3.next()).await {
        Ok(Some(Ok(book))) => {
            println!(
                "Stream3 works: {} bids, {} asks",
                book.bids.len(),
                book.asks.len()
            );
        }
        Ok(Some(Err(e))) => eprintln!("Error: {e}"),
        Ok(None) => println!("Stream ended"),
        Err(_) => println!("Timeout"),
    }

    println!("\n=== Example complete ===");
    println!("With tracing enabled, you should see:");
    println!("  [1] -> 'Subscribing to new market assets'");
    println!("  [2] -> 'All requested assets already subscribed, multiplexing'");
    println!("  [3] -> (no unsubscribe message - refcount > 0)");
    println!("  [4] -> 'Unsubscribing from market assets'");
    println!("  [5] -> 'Subscribing to new market assets'");

    Ok(())
}
