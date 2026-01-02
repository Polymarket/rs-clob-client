#![cfg_attr(doc, doc = include_str!("../README.md"))]

pub mod auth;
#[cfg(feature = "bridge")]
pub mod bridge;
pub mod clob;
#[cfg(feature = "data")]
pub mod data;
pub(crate) mod deser_warn;
pub mod error;
#[cfg(feature = "gamma")]
pub mod gamma;
#[cfg(feature = "rtds")]
pub mod rtds;
pub mod types;

use std::fmt::Write as _;

use alloy::primitives::ChainId;
use phf::phf_map;
use reqwest::header::HeaderMap;
use reqwest::{Request, StatusCode};
use serde::Serialize;
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

/// Given a `chain_id` and `is_neg_risk`, return the relevant [`ContractConfig`]
#[must_use]
pub fn contract_config(chain_id: ChainId, is_neg_risk: bool) -> Option<&'static ContractConfig> {
    if is_neg_risk {
        NEG_RISK_CONFIG.get(&chain_id)
    } else {
        CONFIG.get(&chain_id)
    }
}

/// Trait for converting request types to URL query parameters.
///
/// This trait is automatically implemented for all types that implement [`Serialize`].
/// It uses [`serde_urlencoded`] to serialize the struct fields into a query string.
pub trait ToQueryParams: Serialize {
    /// Converts the request to a URL query string.
    ///
    /// Returns an empty string if no parameters are set, otherwise returns
    /// a string starting with `?` followed by URL-encoded key-value pairs.
    /// Also uses an optional cursor as a parameter, if provided.
    fn query_params(&self, next_cursor: Option<&str>) -> String {
        let mut params = serde_urlencoded::to_string(self)
            .inspect_err(|e| {
                #[cfg(not(feature = "tracing"))]
                let _: &serde_urlencoded::ser::Error = e;

                #[cfg(feature = "tracing")]
                tracing::error!("Unable to convert to URL-encoded string {e:?}");
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

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        level = "debug",
        skip(client, request, headers),
        fields(method, path, status_code)
    )
)]
async fn request<Response: DeserializeOwned>(
    client: &reqwest::Client,
    mut request: Request,
    headers: Option<HeaderMap>,
) -> Result<Response> {
    let method = request.method().clone();
    let path = request.url().path().to_owned();

    #[cfg(feature = "tracing")]
    {
        let span = tracing::Span::current();
        span.record("method", method.as_str());
        span.record("path", path.as_str());
    }

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
    let response_data: Option<Response> = deser_warn::deserialize_with_warnings(json_value)?;

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
}
