#![cfg_attr(doc, doc = include_str!("../README.md"))]

pub mod auth;
pub mod clob;
pub mod error;
#[cfg(feature = "gamma")]
pub mod gamma;

use alloy::primitives::{Address, ChainId, address};
use phf::phf_map;

pub type Result<T> = std::result::Result<T, error::Error>;

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
        conditional_tokens: address!("0x4D97DCd97eC945f40cF65F87097ACe5EA0476045")
    },
    80002_u64 => ContractConfig {
        exchange: address!("0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40"),
        collateral: address!("0x9c4e1703476e875070ee25b56a58b008cfb8fa78"),
        conditional_tokens: address!("0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB"),
    },
};

static NEG_RISK_CONFIG: phf::Map<ChainId, ContractConfig> = phf_map! {
    137_u64 => ContractConfig {
        exchange: address!("0xC5d563A36AE78145C45a50134d48A1215220f80a"),
        collateral: address!("0x2791bca1f2de4661ed88a30c99a7a9449aa84174"),
        conditional_tokens: address!("0x4D97DCd97eC945f40cF65F87097ACe5EA0476045")
    },
    80002_u64 => ContractConfig {
        exchange: address!("0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296"),
        collateral: address!("0x9c4e1703476e875070ee25b56a58b008cfb8fa78"),
        conditional_tokens: address!("0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB"),
    },
};

/// Helper struct to group the relevant deployed contract addresses
#[non_exhaustive]
#[derive(Debug)]
pub struct ContractConfig {
    pub exchange: Address,
    pub collateral: Address,
    pub conditional_tokens: Address,
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
}
