//! Example demonstrating user WebSocket usage for authenticated user events
//!
//! This example shows how to connect to the user WebSocket to receive real-time
//! notifications about trade executions, order updates, and other user-specific events.
//!
//! Run with:
//! ```bash
//! POLYMARKET_PRIVATE_KEY=<your_private_key> cargo run --example websocket_user
//! ```

use std::str::FromStr;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::types::{
    MSG_CANCELLATION, MSG_PLACEMENT, MSG_UPDATE, STATUS_CONFIRMED, STATUS_MATCHED, STATUS_MINED,
    UserWebSocketMessage,
};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load private key from environment
    let private_key = std::env::var(PRIVATE_KEY_VAR)
        .expect("POLYMARKET_PRIVATE_KEY environment variable not set");

    // Create signer with Polygon mainnet
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    println!("Authenticating with address: {}", signer.address());

    // Create and authenticate client
    let client = Client::new("https://clob.polymarket.com", Config::default())?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    println!("Authenticated successfully");

    // Connect to user WebSocket
    let mut ws = client.user_websocket().await?;

    println!("Connected to user WebSocket");
    println!("Listening for user events...\n");

    // Receive and process messages
    let mut message_count = 0;
    while let Some(result) = ws.next().await {
        match result {
            Ok(message) => {
                message_count += 1;

                match message {
                    UserWebSocketMessage::Trade(trade) => {
                        println!("[{}] Trade Event:", message_count);
                        println!("  Trade ID: {}", trade.id);
                        println!("  Order ID: {}", trade.order_id);
                        println!("  Market: {}", trade.market);
                        println!("  Asset ID: {}", trade.asset_id);
                        println!("  Side: {}", trade.side);
                        println!("  Size: {}", trade.size);
                        println!("  Price: {}", trade.price);
                        println!("  Status: {}", trade.status);
                        println!("  Fee Rate (bps): {}", trade.fee_rate_bps);
                        println!("  Timestamp: {}", trade.timestamp);

                        // Handle different trade statuses
                        match trade.status.as_str() {
                            STATUS_MATCHED => {
                                println!("  ✓ Trade matched!");
                                if let Some(match_time) = trade.match_time {
                                    println!("    Match time: {}", match_time);
                                }
                            }
                            STATUS_MINED => {
                                println!("  ⛏ Trade mined on blockchain");
                            }
                            STATUS_CONFIRMED => {
                                println!("  ✓ Trade confirmed on blockchain");
                            }
                            _ => {}
                        }

                        if let Some(trader_side) = trade.trader_side {
                            println!("  Trader Side: {}", trader_side);
                        }

                        println!();
                    }
                    UserWebSocketMessage::Order(order) => {
                        println!("[{}] Order Event:", message_count);
                        println!("  Order ID: {}", order.id);
                        println!("  Message Type: {}", order.msg_type);
                        println!("  Market: {}", order.market);
                        println!("  Asset ID: {}", order.asset_id);
                        println!("  Side: {}", order.side);
                        println!("  Price: {}", order.price);
                        println!("  Original Size: {}", order.original_size);

                        if let Some(size_matched) = &order.size_matched {
                            println!("  Size Matched: {}", size_matched);
                        }

                        if let Some(order_type) = &order.order_type {
                            println!("  Order Type: {}", order_type);
                        }

                        println!("  Timestamp: {}", order.timestamp);

                        // Handle different order message types
                        match order.msg_type.as_str() {
                            MSG_PLACEMENT => {
                                println!("  📝 New order placed");
                            }
                            MSG_UPDATE => {
                                println!("  📝 Order updated");
                            }
                            MSG_CANCELLATION => {
                                println!("  ❌ Order cancelled");
                            }
                            _ => {}
                        }

                        println!();
                    }
                    UserWebSocketMessage::Unknown => {
                        println!("[{}] Unknown message type", message_count);
                        println!();
                    }
                    _ => {
                        println!("[{}] Unhandled message type", message_count);
                        println!();
                    }
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
