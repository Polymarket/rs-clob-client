//! Polymarket Data API client and types.
//!
//! **Feature flag:** `data` (required to use this module)
//!
//! This module provides a client for interacting with the Polymarket Data API,
//! which offers HTTP endpoints for querying user positions, trades, activity,
//! market holders, open interest, volume data, and leaderboards.
//!
//! # Overview
//!
//! The Data API is a read-only HTTP API that provides access to Polymarket data.
//! It is separate from the CLOB (Central Limit Order Book) API which handles trading.
//!
//! ## Available Endpoints
//!
//! | Endpoint | Description |
//! |----------|-------------|
//! | `/` | Health check |
//! | `/positions` | Get current positions for a user |
//! | `/trades` | Get trades for a user or markets |
//! | `/activity` | Get on-chain activity for a user |
//! | `/holders` | Get top holders for markets |
//! | `/value` | Get total value of a user's positions |
//! | `/closed-positions` | Get closed positions for a user |
//! | `/traded` | Get total markets a user has traded |
//! | `/oi` | Get open interest for markets |
//! | `/live-volume` | Get live volume for an event |
//! | `/v1/leaderboard` | Get trader leaderboard rankings |
//! | `/v1/builders/leaderboard` | Get builder leaderboard rankings |
//! | `/v1/builders/volume` | Get daily builder volume time-series |
//!
//! # Example
//!
//! ```no_run
//! use polymarket_client_sdk::types::address;
//! use polymarket_client_sdk::data::{Client, types::request::PositionsRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client with the default endpoint
//! let client = Client::default();
//!
//! // Build a request for user positions
//! let request = PositionsRequest::builder()
//!     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
//!     .build();
//!
//! // Fetch positions
//! let positions = client.positions(&request).await?;
//!
//! for position in positions {
//!     println!("{}: {} tokens at ${:.2}",
//!         position.title,
//!         position.size,
//!         position.current_value
//!     );
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # API Base URL
//!
//! The default API endpoint is `https://data-api.polymarket.com`.

pub mod client;
pub mod types;

pub use client::Client;
