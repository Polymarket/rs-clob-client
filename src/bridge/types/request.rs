use bon::Builder;
use serde::Serialize;

use crate::types::Address;

/// Request to create deposit addresses for a Polymarket wallet.
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::types::address;
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
