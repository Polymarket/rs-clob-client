//! RFQ request types

#![allow(
    clippy::module_name_repetitions,
    reason = "Request suffix is intentional for clarity"
)]

use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use strum_macros::Display;

use crate::auth::ApiKey;
use crate::clob::types::Side;
use crate::types::Address;

/// User type for RFQ participants.
///
/// Indicates the type of wallet/account being used.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Display, Eq, PartialEq, Serialize_repr, Deserialize)]
#[repr(u8)]
pub enum UserType {
    /// Externally Owned Account (standard wallet)
    #[default]
    Eoa = 0,
    /// Polymarket Proxy account
    PolyProxy = 1,
    /// Gnosis Safe multisig
    PolyGnosisSafe = 2,
}

/// RFQ state filter for queries.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Display, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum RfqState {
    /// Active requests/quotes
    #[default]
    Active,
    /// Inactive requests/quotes
    Inactive,
}

/// Sort field for RFQ queries.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Display, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum RfqSortBy {
    /// Sort by price
    Price,
    /// Sort by expiry
    Expiry,
    /// Sort by size
    Size,
    /// Sort by creation time (default)
    #[default]
    Created,
}

/// Sort direction for RFQ queries.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Display, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SortDir {
    /// Ascending order (default)
    #[default]
    Asc,
    /// Descending order
    Desc,
}

/// Request body for creating an RFQ request.
///
/// Creates an RFQ Request to buy or sell outcome tokens.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CreateRfqRequestRequest {
    /// Token ID the Requester wants to receive. "0" indicates USDC.
    pub asset_in: String,
    /// Token ID the Requester wants to give. "0" indicates USDC.
    pub asset_out: String,
    /// Amount of asset to receive (in base units).
    pub amount_in: String,
    /// Amount of asset to give (in base units).
    pub amount_out: String,
    /// User type (`EOA`, `POLY_PROXY`, or `POLY_GNOSIS_SAFE`).
    pub user_type: UserType,
}

/// Request body for canceling an RFQ request.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CancelRfqRequestRequest {
    /// ID of the request to cancel.
    pub request_id: String,
}

/// Query parameters for getting RFQ requests.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct GetRfqRequestsRequest {
    /// Cursor offset for pagination (base64 encoded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    /// Max requests to return. Defaults to 50, max 1000.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Filter by state (active or inactive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<RfqState>,
    /// Filter by request IDs.
    #[serde(rename = "requestIds", skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub request_ids: Vec<String>,
    /// Filter by condition IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub markets: Vec<String>,
    /// Minimum size in tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_min: Option<f64>,
    /// Maximum size in tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_max: Option<f64>,
    /// Minimum size in USDC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_usdc_min: Option<f64>,
    /// Maximum size in USDC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_usdc_max: Option<f64>,
    /// Minimum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_min: Option<f64>,
    /// Maximum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_max: Option<f64>,
    /// Sort field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<RfqSortBy>,
    /// Sort direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_dir: Option<SortDir>,
}

/// Request body for creating an RFQ quote.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CreateQuoteRequest {
    /// ID of the Request to quote.
    pub request_id: String,
    /// Token ID the Quoter wants to receive. "0" indicates USDC.
    pub asset_in: String,
    /// Token ID the Quoter wants to give. "0" indicates USDC.
    pub asset_out: String,
    /// Amount of asset to receive (in base units).
    pub amount_in: String,
    /// Amount of asset to give (in base units).
    pub amount_out: String,
    /// User type (`EOA`, `POLY_PROXY`, or `POLY_GNOSIS_SAFE`).
    pub user_type: UserType,
}

/// Request body for canceling an RFQ quote.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CancelQuoteRequest {
    /// ID of the quote to cancel.
    pub quote_id: String,
}

/// Query parameters for getting RFQ quotes.
#[non_exhaustive]
#[derive(Debug, Clone, Default, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct GetQuotesRequest {
    /// Cursor offset for pagination (base64 encoded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    /// Max quotes to return. Defaults to 50, max 1000.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Filter by state (active or inactive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<RfqState>,
    /// Filter by quote IDs.
    #[serde(rename = "quoteIds", skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub quote_ids: Vec<String>,
    /// Filter by request IDs.
    #[serde(rename = "requestIds", skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub request_ids: Vec<String>,
    /// Filter by condition IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub markets: Vec<String>,
    /// Minimum size in tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_min: Option<f64>,
    /// Maximum size in tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_max: Option<f64>,
    /// Minimum size in USDC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_usdc_min: Option<f64>,
    /// Maximum size in USDC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_usdc_max: Option<f64>,
    /// Minimum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_min: Option<f64>,
    /// Maximum price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_max: Option<f64>,
    /// Sort field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<RfqSortBy>,
    /// Sort direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_dir: Option<SortDir>,
}

/// Request body for accepting an RFQ quote.
///
/// This creates an Order that the Requester must sign.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct AcceptQuoteRequest {
    /// ID of the Request.
    pub request_id: String,
    /// ID of the Quote being accepted.
    pub quote_id: String,
    /// Maker's amount in base units.
    pub maker_amount: String,
    /// Taker's amount in base units.
    pub taker_amount: String,
    /// Outcome token ID.
    pub token_id: String,
    /// Maker's address.
    pub maker: Address,
    /// Signer's address.
    pub signer: Address,
    /// Taker's address.
    pub taker: Address,
    /// Order nonce.
    pub nonce: String,
    /// Unix timestamp for order expiration.
    pub expiration: i64,
    /// Order side (BUY or SELL).
    pub side: Side,
    /// Fee rate in basis points.
    pub fee_rate_bps: String,
    /// EIP-712 signature.
    pub signature: String,
    /// Random salt for order uniqueness.
    pub salt: String,
    /// Owner identifier.
    pub owner: ApiKey,
}

/// Request body for approving an RFQ order.
///
/// Quoter approves an RFQ order during the last look window.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct ApproveOrderRequest {
    /// ID of the Request.
    pub request_id: String,
    /// ID of the Quote being approved.
    pub quote_id: String,
    /// Maker's amount in base units.
    pub maker_amount: String,
    /// Taker's amount in base units.
    pub taker_amount: String,
    /// Outcome token ID.
    pub token_id: String,
    /// Maker's address.
    pub maker: Address,
    /// Signer's address.
    pub signer: Address,
    /// Taker's address.
    pub taker: Address,
    /// Order nonce.
    pub nonce: String,
    /// Unix timestamp for order expiration.
    pub expiration: i64,
    /// Order side (BUY or SELL).
    pub side: Side,
    /// Fee rate in basis points.
    pub fee_rate_bps: String,
    /// EIP-712 signature.
    pub signature: String,
    /// Random salt for order uniqueness.
    pub salt: String,
    /// Owner identifier.
    pub owner: ApiKey,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToQueryParams as _;

    #[test]
    fn create_rfq_request_serializes_correctly() {
        let request = CreateRfqRequestRequest::builder()
            .asset_in("12345")
            .asset_out("0")
            .amount_in("50000000")
            .amount_out("3000000")
            .user_type(UserType::PolyProxy)
            .build();

        let json = serde_json::to_string(&request).expect("serialization should succeed");
        assert!(json.contains("\"assetIn\":\"12345\""));
        assert!(json.contains("\"assetOut\":\"0\""));
        assert!(json.contains("\"userType\":1"));
    }

    #[test]
    fn get_rfq_requests_query_params() {
        let request = GetRfqRequestsRequest::builder()
            .limit(100)
            .state(RfqState::Active)
            .sort_by(RfqSortBy::Price)
            .sort_dir(SortDir::Desc)
            .build();

        let params = request.query_params(None);
        assert!(params.contains("limit=100"));
        assert!(params.contains("state=active"));
        assert!(params.contains("sortBy=price"));
        assert!(params.contains("sortDir=desc"));
    }

    #[test]
    fn get_quotes_request_query_params() {
        let request = GetQuotesRequest::builder()
            .limit(50)
            .price_min(0.1)
            .price_max(0.9)
            .build();

        let params = request.query_params(None);
        assert!(params.contains("limit=50"));
        assert!(params.contains("priceMin=0.1"));
        assert!(params.contains("priceMax=0.9"));
    }

    #[test]
    fn user_type_serializes_as_integer() {
        assert_eq!(
            serde_json::to_string(&UserType::Eoa).expect("serialize"),
            "0"
        );
        assert_eq!(
            serde_json::to_string(&UserType::PolyProxy).expect("serialize"),
            "1"
        );
        assert_eq!(
            serde_json::to_string(&UserType::PolyGnosisSafe).expect("serialize"),
            "2"
        );
    }
}
