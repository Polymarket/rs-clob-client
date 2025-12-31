//! Subscribe to authenticated (user) WebSocket channels.
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]
#![allow(clippy::print_stderr, reason = "Examples are okay to print to stderr")]

use std::str::FromStr as _;

use futures::StreamExt as _;
use polymarket_client_sdk::auth::Credentials;
use polymarket_client_sdk::clob::ws::{Client, WsMessage};
use polymarket_client_sdk::types::Address;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = Uuid::parse_str(&std::env::var("POLYMARKET_API_KEY")?)?;
    let api_secret = std::env::var("POLYMARKET_API_SECRET")?;
    let api_passphrase = std::env::var("POLYMARKET_API_PASSPHRASE")?;
    let address = Address::from_str(&std::env::var("POLYMARKET_ADDRESS")?)?;

    // Build credentials for the authenticated ws channel.
    let credentials = Credentials::new(api_key, api_secret, api_passphrase);

    // Connect using the base WebSocket endpoint and authenticate.
    let client = Client::default().authenticate(credentials, address)?;
    println!("Authenticated ws client created.");

    // Provide the specific market IDs you care about, or leave empty to receive all events.
    // let markets = vec!["0xe93c89c41d1bb08d3bb40066d8565df301a696563b2542256e6e8bbbb1ec490d".to_owned()];
    let markets: Vec<String> = Vec::new();
    let mut stream = std::pin::pin!(client.subscribe_user_events(markets)?);

    println!("Subscribed to user ws channel.");

    while let Some(event) = stream.next().await {
        match event {
            Ok(WsMessage::Order(order)) => {
                println!("\n--- Order Update ---");
                println!("Order ID: {}", order.id);
                println!("Market: {}", order.market);
                println!("Type: {:?}", order.msg_type);
                println!("Side: {:?} Price: {}", order.side, order.price);
                if let Some(size) = &order.original_size {
                    println!("Original Size: {size}");
                }
                if let Some(matched) = &order.size_matched {
                    println!("Size Matched: {matched}");
                }
            }
            Ok(WsMessage::Trade(trade)) => {
                println!("\n--- Trade ---");
                println!("Trade ID: {}", trade.id);
                println!("Market: {}", trade.market);
                println!("Status: {}", trade.status);
                println!(
                    "Side: {:?} Size: {} Price: {}",
                    trade.side, trade.size, trade.price
                );
                if let Some(trader_side) = &trade.trader_side {
                    println!("Trader Side: {trader_side:?}");
                }
            }
            Ok(other) => {
                println!("Other event: {other:?}");
            }
            Err(e) => {
                eprintln!("Stream error: {e}");
                break;
            }
        }
    }

    Ok(())
}
