//! Example demonstrating market WebSocket usage for real-time order book updates
//!
//! This example shows how to connect to the market WebSocket and subscribe to
//! asset updates for real-time price changes and order book data.
//!
//! Run with:
//! ```bash
//! cargo run --example websocket_market
//! ```

use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::types::MarketWebSocketMessage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create an unauthenticated client
    let client = Client::new("https://clob.polymarket.com", Config::default())?;

    // Connect to market WebSocket
    let mut ws = client.market_websocket().await?;

    println!("Connected to market WebSocket");

    // Example asset IDs - replace with actual asset IDs you want to monitor
    let asset_ids = vec![
        "51338236787729560681434534660841415073585974762690814047670810862722808070955",
        // "18289842382539867639079362738467334752951741961393928566628307174343542320349",
    ];

    // Subscribe to market updates
    let asset_refs: Vec<&str> = asset_ids.iter().map(|s| s.as_ref()).collect();
    ws.subscribe(&asset_refs).await?;

    println!("Subscribed to {} assets", asset_ids.len());
    println!("Listening for market updates...\n");

    // Receive and process messages
    let mut message_count = 0;
    while let Some(result) = ws.next().await {
        match result {
            Ok(messages) => {
                for message in messages {
                    message_count += 1;

                    match message {
                        MarketWebSocketMessage::Book(book) => {
                            println!("[{}] Book Update:", message_count);
                            println!("  Asset ID: {}", book.asset_id);
                            println!("  Market: {}", book.market);
                            println!("  Timestamp: {}", book.timestamp);
                            println!("  Bids: {} levels", book.bids.len());
                            println!("  Asks: {} levels", book.asks.len());

                            if let (Some(best_bid), Some(best_ask)) =
                                (book.bids.first(), book.asks.first())
                            {
                                println!("  Best Bid: {} @ {}", best_bid.size, best_bid.price);
                                println!("  Best Ask: {} @ {}", best_ask.size, best_ask.price);
                            }
                            println!();
                        }
                        MarketWebSocketMessage::PriceChange(price_change) => {
                            println!("[{}] Price Change:", message_count);
                            println!("  Timestamp: {}", price_change.timestamp);
                            println!("  Changes: {}", price_change.price_changes.len());
                            println!("   Market: {}", price_change.market);

                            for change in &price_change.price_changes {
                                println!("    Asset: {}", change.asset_id);
                                println!("      Best Bid: {}", change.best_bid);
                                println!("      Best Ask: {}", change.best_ask);
                            }
                            println!();
                        }
                        MarketWebSocketMessage::LastTradePrice(trade) => {
                            println!("[{}] Last Trade Price:", message_count);
                            println!("  Asset ID: {}", trade.asset_id);
                            println!("  Market: {}", trade.market);
                            println!("  Timestamp: {}", trade.timestamp);
                            println!();
                        }
                        MarketWebSocketMessage::Unknown => {
                            println!("[{}] Unknown message type", message_count);
                            println!();
                        }
                        _ => {
                            println!("[{}] Unhandled message type", message_count);
                            println!();
                        }
                    }

                    // Optionally limit the number of messages to process
                    // if message_count >= 10 {
                    //     println!("Received {} messages, closing connection...", message_count);
                    //     ws.close().await?;
                    //     println!("WebSocket connection closed");
                    //     return Ok(());
                    // }
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
            }
        }
    }

    // Gracefully close the WebSocket
    ws.close().await?;
    println!("WebSocket connection closed");

    Ok(())
}
