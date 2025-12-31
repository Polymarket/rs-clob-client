use bon::Builder;
use serde::Deserialize;

use crate::types::Decimal;

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
    pub min_checkout_usd: Decimal,
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
