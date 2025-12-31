//! RFQ (Request for Quote) API client and types.
//!
//! This module provides functionality for interacting with the Polymarket RFQ system,
//! which allows users to create requests for quotes on outcome tokens and receive
//! quotes from market makers.
//!
//! # Overview
//!
//! The RFQ system consists of:
//! - **Requests**: Users create requests to buy or sell outcome tokens
//! - **Quotes**: Market makers respond with quotes for those requests
//! - **Execution**: Users can accept quotes and market makers can approve orders
//!
//! All endpoints require L2 authentication via credentials.
//!
//! # Example
//!
//! ```rust,no_run
//! use polymarket_client_sdk::rfq::Client;
//! use polymarket_client_sdk::rfq::types::{CreateRfqRequestRequest, UserType};
//! use polymarket_client_sdk::auth::Credentials;
//! use alloy::primitives::Address;
//! use uuid::Uuid;
//! use std::str::FromStr;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create credentials (obtained via CLOB API key creation)
//! let address = Address::from_str("0x...")?;
//! let credentials = Credentials::new(
//!     Uuid::new_v4(),
//!     "your-secret".to_string(),
//!     "your-passphrase".to_string(),
//! );
//!
//! // Create an RFQ client
//! let rfq_client = Client::new("https://clob.polymarket.com", address, credentials)?;
//!
//! // Create an RFQ request
//! let request = CreateRfqRequestRequest::builder()
//!     .asset_in("12345")  // Token ID to receive
//!     .asset_out("0")      // USDC to give
//!     .amount_in("50000000")
//!     .amount_out("3000000")
//!     .user_type(UserType::Eoa)
//!     .build();
//!
//! let response = rfq_client.create_request(&request).await?;
//! println!("Created RFQ request: {}", response.request_id);
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod types;

pub use client::Client;
