#![expect(
    clippy::module_name_repetitions,
    reason = "Re-exported names intentionally match their modules for API clarity"
)]

//! Real-Time Data Socket (RTDS) client for streaming Polymarket data.
//!
//! This module provides a WebSocket-based client for subscribing to real-time
//! data streams from Polymarket's RTDS service.
//!
//! # Available Streams
//!
//! - **Crypto Prices (Binance)**: Real-time cryptocurrency price data from Binance
//! - **Crypto Prices (Chainlink)**: Price data from Chainlink oracle networks
//! - **Comments**: Comment events including creations, removals, and reactions
//!
//! # Example
//!
//! ```rust, no_run
//! use polymarket_client_sdk::rtds::Client;
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = Client::default();
//!
//!     // Subscribe to BTC prices
//!     let stream = client.subscribe_crypto_prices(Some(vec!["btcusdt".to_owned()]))?;
//!     let mut stream = Box::pin(stream);
//!
//!     while let Some(price) = stream.next().await {
//!         println!("BTC Price: {:?}", price?);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod connection;
pub mod error;
pub mod subscription;
pub mod types;

// Re-export commonly used types
pub use client::Client;
pub use config::{Config, ReconnectConfig};
pub use connection::ConnectionState;
pub use error::RtdsError;
pub use subscription::{SubscriptionInfo, TopicType};
pub use types::request::{Subscription, SubscriptionAction, SubscriptionRequest};
pub use types::response::{
    ChainlinkPrice, Comment, CommentProfile, CommentType, CryptoPrice, RtdsMessage,
};
