//! Re-exported types from external crates for convenience.
//!
//! These types are commonly used in this SDK and are re-exported here
//! so users don't need to add these dependencies to their `Cargo.toml`.

/// Ethereum address type and the [`address!`] macro for compile-time address literals.
/// [`ChainId`] is a type alias for `u64` representing EVM chain IDs.
/// [`Signature`] represents cryptographic signatures for signed orders.
/// [`B256`] is a 256-bit fixed-size byte array type used for condition IDs and hashes.
/// [`U256`] is a 256-bit integer
pub use alloy::primitives::{Address, B256, ChainId, Signature, U256, address, b256};
/// Date and time types for timestamps in API responses and order expiration.
pub use chrono::{DateTime, NaiveDate, Utc};
/// Arbitrary precision decimal type for prices, sizes, and amounts.
pub use rust_decimal::Decimal;
/// Macro for creating [`Decimal`] literals at compile time.
///
/// # Example
/// ```
/// use polymarket_client_sdk::types::dec;
/// let price = dec!(0.55);
/// ```
pub use rust_decimal_macros::dec;
