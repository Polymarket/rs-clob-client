#![expect(
    clippy::module_name_repetitions,
    reason = "Re-exported names intentionally match their modules for API clarity"
)]

//! WebSocket client for real-time market data and user updates.
//!
//! This module provides WebSocket connectivity to the CLOB API, enabling
//! real-time subscriptions to orderbook updates, price changes, and user-specific events.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_client_sdk::ws::{WebSocketClient, WebSocketConfig};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = WebSocketClient::new(
//!         "wss://ws-subscriptions-clob.polymarket.com",
//!         WebSocketConfig::default()
//!     )?;
//!
//!     let mut stream = client
//!         .subscribe_orderbook(vec!["asset_id".to_owned()])
//!         .await?;
//!
//!     while let Some(book) = stream.next().await {
//!         println!("Orderbook update: {:?}", book?);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod connection;
pub mod error;
pub mod messages;
pub mod subscription;

// Re-export commonly used types
pub use client::WebSocketClient;
pub use config::{ReconnectConfig, WebSocketConfig};
pub use error::WsError;
pub use messages::{
    AuthPayload, BookUpdate, LastTradePrice, OrderMessage, OrderStatus, PriceChange,
    SubscriptionRequest, TickSizeChange, TradeMessage, WsMessage,
};
