use alloy::primitives::Address;
use bon::Builder;
use serde::Serialize;

/// Request to create deposit addresses for a Polymarket wallet.
///
/// # Example
///
/// ```
/// use alloy::primitives::address;
/// use polymarket_client_sdk::bridge::types::DepositRequest;
///
/// let request = DepositRequest::builder()
///     .address(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
///     .build();
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
pub struct DepositRequest {
    /// The Polymarket wallet address to generate deposit addresses for.
    pub address: Address,
}
