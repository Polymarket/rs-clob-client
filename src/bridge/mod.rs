//! Polymarket Bridge API client and types.
//!
//! **Feature flag:** `bridge` (required to use this module)
//!
//! This module provides a client for interacting with the Polymarket Bridge API,
//! which enables bridging assets from various chains (EVM, Solana, Bitcoin) to
//! USDC.e on Polygon for trading on Polymarket.
//!
//! # Overview
//!
//! The Bridge API is a read/write HTTP API that provides:
//! - Deposit address generation for multi-chain asset bridging
//! - Supported asset and chain information
//!
//! ## Available Endpoints
//!
//! | Endpoint | Method | Description |
//! |----------|--------|-------------|
//! | `/deposit` | POST | Create deposit addresses for a wallet |
//! | `/supported-assets` | GET | Get supported chains and tokens |
//!
//! # Example
//!
//! ```no_run
//! use polymarket_client_sdk::types::address;
//! use polymarket_client_sdk::bridge::{Client, types::DepositRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client with the default endpoint
//! let client = Client::default();
//!
//! // Get deposit addresses for a wallet
//! let request = DepositRequest::builder()
//!     .address(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
//!     .build();
//!
//! let response = client.deposit(&request).await?;
//! println!("EVM: {}", response.address.evm);
//! println!("SVM: {}", response.address.svm);
//! println!("BTC: {}", response.address.btc);
//! # Ok(())
//! # }
//! ```
//!
//! # API Base URL
//!
//! The default API endpoint is `https://bridge.polymarket.com`.

pub mod client;
pub mod types;

pub use client::Client;
