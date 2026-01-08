//! CTF (Conditional Token Framework) API client.
//!
//! **Feature flag:** `ctf` (required to use this module)
//!
//! The Conditional Token Framework is Gnosis's smart contract system that tokenizes
//! all Polymarket outcomes as binary ERC1155 tokens on Polygon. Each market has two
//! outcome tokens ("YES" and "NO") backed by USDC collateral.
//!
//! # Features
//!
//! - **ID Calculation**: Compute condition IDs, collection IDs, and position IDs
//! - **Splitting**: Convert USDC collateral into outcome token pairs (YES/NO)
//! - **Merging**: Combine outcome token pairs back into USDC
//! - **Redemption**: Redeem winning outcome tokens after market resolution
//!
//! # Example
//!
//! ```ignore
//! use polymarket_client_sdk::ctf::{Client, types::*};
//! use polymarket_client_sdk::types::address;
//! use polymarket_client_sdk::POLYGON;
//! use alloy::providers::ProviderBuilder;
//! use alloy::primitives::{B256, U256};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a provider (requires a wallet for state-changing operations)
//! let provider = ProviderBuilder::new()
//!     .connect("https://polygon-rpc.com")
//!     .await?;
//!
//! let client = Client::new(provider, POLYGON)?;
//!
//! let condition_id_req = ConditionIdRequest::builder()
//!     .oracle(address!("<oracle_address>"))
//!     .question_id(B256::default())
//!     .outcome_slot_count(U256::from(2))
//!     .build();
//!
//! let condition_id = client.condition_id(&condition_id_req).await?;
//! println!("Condition ID: {}", condition_id.condition_id);
//!
//! // Split USDC into outcome tokens
//! let split_req = SplitPositionRequest::builder()
//!     .collateral_token(address!("<collateral_token_address>"))
//!     .condition_id(condition_id.condition_id)
//!     .partition(vec![U256::from(1), U256::from(2)])
//!     .amount(U256::from(1_000_000)) // 1 USDC (6 decimals)
//!     .build();
//!
//! let result = client.split_position(&split_req).await?;
//! println!("Split tx: {}", result.transaction_hash);
//! # Ok(())
//! # }
//! ```
//!
//! # Contract Address
//!
//! - Polygon Mainnet: `0x4D97DCd97eC945f40cF65F87097ACe5EA0476045`
//! - Polygon Amoy Testnet: Available in contract configuration
//!
//! # Resources
//!
//! - [CTF Documentation](https://docs.polymarket.com/developers/CTF/overview)
//! - [Gnosis CTF Source Code](https://github.com/gnosis/conditional-tokens-contracts)

pub mod client;
mod error;
pub mod types;

pub use client::Client;
