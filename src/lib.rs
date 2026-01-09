#![cfg_attr(doc, doc = include_str!("../README.md"))]

pub mod auth;
#[cfg(feature = "bridge")]
pub mod bridge;
#[cfg(feature = "clob")]
pub mod clob;
#[cfg(feature = "ctf")]
pub mod ctf;
#[cfg(feature = "data")]
pub mod data;
pub mod error;
#[cfg(feature = "gamma")]
pub mod gamma;
#[cfg(feature = "rtds")]
pub mod rtds;
pub(crate) mod serde_helpers;
pub mod types;
#[cfg(any(feature = "ws", feature = "rtds"))]
pub mod ws;

use std::fmt::Write as _;

use alloy::primitives::ChainId;
use alloy::primitives::{B256, b256, keccak256};
use phf::phf_map;
#[cfg(any(
    feature = "bridge",
    feature = "clob",
    feature = "data",
    feature = "gamma"
))]
use reqwest::{Request, StatusCode, header::HeaderMap};
use serde::Serialize;
#[cfg(any(
    feature = "bridge",
    feature = "clob",
    feature = "data",
    feature = "gamma"
))]
use serde::de::DeserializeOwned;

use crate::error::Error;
use crate::types::{Address, address};

pub type Result<T> = std::result::Result<T, Error>;

/// [`ChainId`] for Polygon mainnet
pub const POLYGON: ChainId = 137;

/// [`ChainId`] for Polygon testnet <https://polygon.technology/blog/introducing-the-amoy-testnet-for-polygon-pos>
pub const AMOY: ChainId = 80002;

pub const PRIVATE_KEY_VAR: &str = "POLYMARKET_PRIVATE_KEY";

/// Timestamp in seconds since [`std::time::UNIX_EPOCH`]
pub(crate) type Timestamp = i64;

static CONFIG: phf::Map<ChainId, ContractConfig> = phf_map! {
    137_u64 => ContractConfig {
        exchange: address!("0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"),
        collateral: address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"),
        conditional_tokens: address!("0x4D97DCd97eC945f40cF65F87097ACe5EA0476045"),
        neg_risk_adapter: None,
    },
    80002_u64 => ContractConfig {
        exchange: address!("0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40"),
        collateral: address!("0x9c4e1703476e875070ee25b56a58b008cfb8fa78"),
        conditional_tokens: address!("0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB"),
        neg_risk_adapter: None,
    },
};

static NEG_RISK_CONFIG: phf::Map<ChainId, ContractConfig> = phf_map! {
    137_u64 => ContractConfig {
        exchange: address!("0xC5d563A36AE78145C45a50134d48A1215220f80a"),
        collateral: address!("0x2791bca1f2de4661ed88a30c99a7a9449aa84174"),
        conditional_tokens: address!("0x4D97DCd97eC945f40cF65F87097ACe5EA0476045"),
        neg_risk_adapter: Some(address!("0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296")),
    },
    80002_u64 => ContractConfig {
        exchange: address!("0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296"),
        collateral: address!("0x9c4e1703476e875070ee25b56a58b008cfb8fa78"),
        conditional_tokens: address!("0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB"),
        neg_risk_adapter: Some(address!("0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296")),
    },
};

// Wallet contract configurations for CREATE2 address derivation
// Source: https://github.com/Polymarket/builder-relayer-client
static WALLET_CONFIG: phf::Map<ChainId, WalletContractConfig> = phf_map! {
    137_u64 => WalletContractConfig {
        proxy_factory: Some(address!("0xaB45c5A4B0c941a2F231C04C3f49182e1A254052")),
        safe_factory: address!("0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b"),
    },
    80002_u64 => WalletContractConfig {
        // Proxy factory unsupported on Amoy testnet
        proxy_factory: None,
        safe_factory: address!("0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b"),
    },
};

/// Init code hash for Polymarket Proxy wallets (EIP-1167 minimal proxy)
const PROXY_INIT_CODE_HASH: B256 =
    b256!("0xd21df8dc65880a8606f09fe0ce3df9b8869287ab0b058be05aa9e8af6330a00b");

/// Init code hash for Gnosis Safe wallets
const SAFE_INIT_CODE_HASH: B256 =
    b256!("0x2bce2127ff07fb632d16c8347c4ebf501f4841168bed00d9e6ef715ddb6fcecf");

/// Helper struct to group the relevant deployed contract addresses
#[non_exhaustive]
#[derive(Debug)]
pub struct ContractConfig {
    pub exchange: Address,
    pub collateral: Address,
    pub conditional_tokens: Address,
    /// The Neg Risk Adapter contract address. Only present for neg-risk market configs.
    /// Users must approve this contract for token transfers to trade in neg-risk markets.
    pub neg_risk_adapter: Option<Address>,
}

/// Wallet contract configuration for CREATE2 address derivation
#[non_exhaustive]
#[derive(Debug)]
pub struct WalletContractConfig {
    /// Factory contract for Polymarket Proxy wallets (Magic/email wallets).
    /// Not available on all networks (e.g., Amoy testnet).
    pub proxy_factory: Option<Address>,
    /// Factory contract for Gnosis Safe wallets.
    pub safe_factory: Address,
}

/// Given a `chain_id` and `is_neg_risk`, return the relevant [`ContractConfig`]
#[must_use]
pub fn contract_config(chain_id: ChainId, is_neg_risk: bool) -> Option<&'static ContractConfig> {
    if is_neg_risk {
        NEG_RISK_CONFIG.get(&chain_id)
    } else {
        CONFIG.get(&chain_id)
    }
}

/// Returns the wallet contract configuration for the given chain ID.
#[must_use]
pub fn wallet_contract_config(chain_id: ChainId) -> Option<&'static WalletContractConfig> {
    WALLET_CONFIG.get(&chain_id)
}

/// Derives the Polymarket Proxy wallet address for an EOA using CREATE2.
///
/// This is the deterministic address of the EIP-1167 minimal proxy wallet
/// that Polymarket deploys for Magic/email wallet users.
///
/// # Arguments
/// * `eoa_address` - The externally owned account (EOA) address
/// * `chain_id` - The chain ID (e.g., 137 for Polygon mainnet)
///
/// # Returns
/// * `Some(Address)` - The derived proxy wallet address
/// * `None` - If the chain doesn't support proxy wallets or config is missing
#[must_use]
pub fn derive_proxy_wallet(eoa_address: Address, chain_id: ChainId) -> Option<Address> {
    let config = wallet_contract_config(chain_id)?;
    let factory = config.proxy_factory?;

    // Salt is keccak256(encodePacked(address)) - address is 20 bytes, no padding
    let salt = keccak256(eoa_address);

    Some(factory.create2(salt, PROXY_INIT_CODE_HASH))
}

/// Derives the Gnosis Safe wallet address for an EOA using CREATE2.
///
/// This is the deterministic address of the 1-of-1 Gnosis Safe multisig
/// that Polymarket deploys for browser wallet users.
///
/// # Arguments
/// * `eoa_address` - The externally owned account (EOA) address
/// * `chain_id` - The chain ID (e.g., 137 for Polygon mainnet)
///
/// # Returns
/// * `Some(Address)` - The derived Safe wallet address
/// * `None` - If the chain config is missing
#[must_use]
pub fn derive_safe_wallet(eoa_address: Address, chain_id: ChainId) -> Option<Address> {
    let config = wallet_contract_config(chain_id)?;
    let factory = config.safe_factory;

    // Salt is keccak256(encodeAbiParameters(address)) - address padded to 32 bytes
    // ABI encoding pads address to 32 bytes (left-padded with zeros)
    let mut padded = [0_u8; 32];
    padded[12..].copy_from_slice(eoa_address.as_slice());
    let salt = keccak256(padded);

    Some(factory.create2(salt, SAFE_INIT_CODE_HASH))
}

/// Trait for converting request types to URL query parameters.
///
/// This trait is automatically implemented for all types that implement [`Serialize`].
/// It uses [`serde_html_form`] to serialize the struct fields into a query string.
/// Arrays are serialized as repeated keys (`key=val1&key=val2`).
pub trait ToQueryParams: Serialize {
    /// Converts the request to a URL query string.
    ///
    /// Returns an empty string if no parameters are set, otherwise returns
    /// a string starting with `?` followed by URL-encoded key-value pairs.
    /// Also uses an optional cursor as a parameter, if provided.
    fn query_params(&self, next_cursor: Option<&str>) -> String {
        let mut params = serde_html_form::to_string(self)
            .inspect_err(|e| {
                #[cfg(feature = "tracing")]
                tracing::error!("Unable to convert to URL-encoded string {e:?}");
                #[cfg(not(feature = "tracing"))]
                let _: &serde_html_form::ser::Error = e;
            })
            .unwrap_or_default();

        if let Some(cursor) = next_cursor {
            if !params.is_empty() {
                params.push('&');
            }
            let _ = write!(params, "next_cursor={cursor}");
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("?{params}")
        }
    }
}

impl<T: Serialize> ToQueryParams for T {}

#[cfg(any(
    feature = "bridge",
    feature = "clob",
    feature = "data",
    feature = "gamma"
))]
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        level = "debug",
        skip(client, request, headers),
        fields(
            method = %request.method(),
            path = request.url().path(),
            status_code
        )
    )
)]
async fn request<Response: DeserializeOwned>(
    client: &reqwest::Client,
    mut request: Request,
    headers: Option<HeaderMap>,
) -> Result<Response> {
    let method = request.method().clone();
    let path = request.url().path().to_owned();

    if let Some(h) = headers {
        *request.headers_mut() = h;
    }

    let response = client.execute(request).await?;
    let status_code = response.status();

    #[cfg(feature = "tracing")]
    tracing::Span::current().record("status_code", status_code.as_u16());

    if !status_code.is_success() {
        let message = response.text().await.unwrap_or_default();

        #[cfg(feature = "tracing")]
        tracing::warn!(
            status = %status_code,
            method = %method,
            path = %path,
            message = %message,
            "API request failed"
        );

        return Err(Error::status(status_code, method, path, message));
    }

    let json_value = response.json::<serde_json::Value>().await?;
    let response_data: Option<Response> = serde_helpers::deserialize_with_warnings(json_value)?;

    if let Some(response) = response_data {
        Ok(response)
    } else {
        #[cfg(feature = "tracing")]
        tracing::warn!(method = %method, path = %path, "API resource not found");
        Err(Error::status(
            StatusCode::NOT_FOUND,
            method,
            path,
            "Unable to find requested resource",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_contains_80002() {
        let cfg = contract_config(AMOY, false).expect("missing config");
        assert_eq!(
            cfg.exchange,
            address!("0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40")
        );
    }

    #[test]
    fn config_contains_80002_neg() {
        let cfg = contract_config(AMOY, true).expect("missing config");
        assert_eq!(
            cfg.exchange,
            address!("0xd91e80cf2e7be2e162c6513ced06f1dd0da35296")
        );
    }

    #[test]
    fn wallet_contract_config_polygon() {
        let cfg = wallet_contract_config(POLYGON).expect("missing config");
        assert_eq!(
            cfg.proxy_factory,
            Some(address!("0xaB45c5A4B0c941a2F231C04C3f49182e1A254052"))
        );
        assert_eq!(
            cfg.safe_factory,
            address!("0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b")
        );
    }

    #[test]
    fn wallet_contract_config_amoy() {
        let cfg = wallet_contract_config(AMOY).expect("missing config");
        // Proxy factory not supported on Amoy
        assert_eq!(cfg.proxy_factory, None);
        assert_eq!(
            cfg.safe_factory,
            address!("0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b")
        );
    }

    #[test]
    fn derive_safe_wallet_polygon() {
        // Test address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (Foundry/Anvil test key)
        let eoa = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let safe_addr = derive_safe_wallet(eoa, POLYGON).expect("derivation failed");

        // This is the deterministic Safe address for this EOA on Polygon
        assert_eq!(
            safe_addr,
            address!("0xd93b25Cb943D14d0d34FBAf01fc93a0F8b5f6e47")
        );
    }

    #[test]
    fn derive_proxy_wallet_polygon() {
        // Test address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (Foundry/Anvil test key)
        let eoa = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let proxy_addr = derive_proxy_wallet(eoa, POLYGON).expect("derivation failed");

        // This is the deterministic Proxy address for this EOA on Polygon
        assert_eq!(
            proxy_addr,
            address!("0x365f0cA36ae1F641E02Fe3b7743673DA42A13a70")
        );
    }

    #[test]
    fn derive_proxy_wallet_amoy_not_supported() {
        let eoa = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        // Proxy wallet derivation should fail on Amoy (no proxy factory)
        assert!(derive_proxy_wallet(eoa, AMOY).is_none());
    }

    #[test]
    fn derive_safe_wallet_amoy() {
        let eoa = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        // Safe wallet derivation should work on Amoy
        let safe_addr = derive_safe_wallet(eoa, AMOY).expect("derivation failed");

        // Same Safe factory on both networks, so same derived address
        assert_eq!(
            safe_addr,
            address!("0xd93b25Cb943D14d0d34FBAf01fc93a0F8b5f6e47")
        );
    }

    #[test]
    fn derive_wallet_unsupported_chain() {
        let eoa = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        // Unsupported chain should return None
        assert!(derive_proxy_wallet(eoa, 1).is_none());
        assert!(derive_safe_wallet(eoa, 1).is_none());
    }
}
