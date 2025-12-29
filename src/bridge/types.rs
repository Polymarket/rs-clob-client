use alloy::primitives::Address;
use bon::Builder;
use serde::{Deserialize, Serialize};

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

/// Response containing deposit addresses for different blockchain networks.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq, Builder)]
pub struct DepositResponse {
    /// Deposit addresses for different blockchain networks.
    pub address: DepositAddresses,
    /// Additional information about supported chains.
    pub note: Option<String>,
}

/// Deposit addresses for different blockchain networks.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq, Builder)]
#[builder(on(String, into))]
pub struct DepositAddresses {
    /// EVM-compatible deposit address (Ethereum, Polygon, Arbitrum, Base, etc.).
    pub evm: String,
    /// Solana Virtual Machine deposit address.
    pub svm: String,
    /// Bitcoin deposit address.
    pub btc: String,
}

/// Response containing all supported assets for deposits.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq, Builder)]
#[serde(rename_all = "camelCase")]
pub struct SupportedAssetsResponse {
    /// List of supported assets with minimum deposit amounts.
    pub supported_assets: Vec<SupportedAsset>,
}

/// A supported asset with chain and token information.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq, Builder)]
#[builder(on(String, into))]
#[serde(rename_all = "camelCase")]
pub struct SupportedAsset {
    /// Chain ID.
    pub chain_id: String,
    /// Human-readable chain name.
    pub chain_name: String,
    /// Token information.
    pub token: Token,
    /// Minimum deposit amount in USD.
    pub min_checkout_usd: f64,
}

/// Token information for a supported asset.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, PartialEq, Builder)]
#[builder(on(String, into))]
pub struct Token {
    /// Full token name.
    pub name: String,
    /// Token symbol.
    pub symbol: String,
    /// Token contract address.
    pub address: String,
    /// Token decimals.
    pub decimals: u8,
}
