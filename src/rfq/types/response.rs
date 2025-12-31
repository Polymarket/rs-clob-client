//! RFQ response types

#![allow(
    clippy::module_name_repetitions,
    reason = "Response suffix is intentional for clarity"
)]

use bon::Builder;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::clob::types::Side;
use crate::types::Address;

/// Response from creating an RFQ request.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CreateRfqRequestResponse {
    /// Unique identifier for the created request.
    pub request_id: String,
    /// Unix timestamp when the request expires.
    pub expiry: i64,
}

/// Response from creating an RFQ quote.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CreateQuoteResponse {
    /// Unique identifier for the created quote.
    pub quote_id: String,
}

/// Response from accepting an RFQ quote.
///
/// Returns "OK" as text, represented as unit type for deserialization.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptQuoteResponse;

/// Response from approving an RFQ order.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct ApproveOrderResponse {
    /// Trade IDs for the executed order.
    pub trade_ids: Vec<String>,
}

/// An RFQ request in the system.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct RfqRequest {
    /// Unique request identifier.
    pub request_id: String,
    /// User's address.
    pub user: Address,
    /// Proxy address (may be same as user).
    pub proxy: Address,
    /// Market condition ID.
    pub market: String,
    /// Token ID for the outcome token.
    pub token: String,
    /// Complement token ID.
    pub complement: String,
    /// Order side (BUY or SELL).
    pub side: Side,
    /// Size of tokens to receive.
    pub size_in: Decimal,
    /// Size of tokens to give.
    pub size_out: Decimal,
    /// Price for the request.
    pub price: Decimal,
    /// Unix timestamp when the request expires.
    pub expiry: i64,
}

/// An RFQ quote in the system.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct Quote {
    /// Unique quote identifier.
    pub quote_id: String,
    /// Request ID this quote is for.
    pub request_id: String,
    /// Quoter's address.
    pub user: Address,
    /// Proxy address (may be same as user).
    pub proxy: Address,
    /// Market condition ID.
    pub market: String,
    /// Token ID for the outcome token.
    pub token: String,
    /// Complement token ID.
    pub complement: String,
    /// Order side (BUY or SELL).
    pub side: Side,
    /// Size of tokens to receive.
    pub size_in: Decimal,
    /// Size of tokens to give.
    pub size_out: Decimal,
    /// Quoted price.
    pub price: Decimal,
}

/// Paginated response wrapper for RFQ queries.
///
/// This is compatible with the existing `Page<T>` structure but allows
/// for RFQ-specific field naming if needed.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Builder, PartialEq)]
#[builder(on(String, into))]
pub struct RfqPage<T> {
    /// List of items in this page.
    pub data: Vec<T>,
    /// Cursor for the next page (base64 encoded).
    pub next_cursor: String,
    /// Maximum items per page.
    pub limit: u64,
    /// Number of items in this page.
    pub count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_create_rfq_request_response() {
        let json = r#"{"requestId":"0196464a-a1fa-75e6-821e-31aa0794f7ad","expiry":1744936318}"#;
        let response: CreateRfqRequestResponse =
            serde_json::from_str(json).expect("deserialize should succeed");

        assert_eq!(response.request_id, "0196464a-a1fa-75e6-821e-31aa0794f7ad");
        assert_eq!(response.expiry, 1_744_936_318);
    }

    #[test]
    fn deserialize_create_quote_response() {
        let json = r#"{"quoteId":"0196464a-a1fa-75e6-821e-31aa0794f7ad"}"#;
        let response: CreateQuoteResponse =
            serde_json::from_str(json).expect("deserialize should succeed");

        assert_eq!(response.quote_id, "0196464a-a1fa-75e6-821e-31aa0794f7ad");
    }

    #[test]
    fn deserialize_approve_order_response() {
        let json = r#"{"tradeIds":["019af0f7-eb77-764f-b40f-6de8a3562e12"]}"#;
        let response: ApproveOrderResponse =
            serde_json::from_str(json).expect("deserialize should succeed");

        assert_eq!(response.trade_ids.len(), 1);
        assert_eq!(
            response.trade_ids[0],
            "019af0f7-eb77-764f-b40f-6de8a3562e12"
        );
    }

    #[test]
    fn deserialize_rfq_request() {
        let json = r#"{
            "requestId": "01968f1e-1182-71c4-9d40-172db9be82af",
            "user": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
            "proxy": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
            "market": "0x37a6a2dd9f3469495d9ec2467b0a764c5905371a294ce544bc3b2c944eb3e84a",
            "token": "34097058504275310827233323421517291090691602969494795225921954353603704046623",
            "complement": "32868290514114487320702931554221558599637733115139769311383916145370132125101",
            "side": "BUY",
            "sizeIn": 100,
            "sizeOut": 50,
            "price": 0.5,
            "expiry": 1746159634
        }"#;
        let request: RfqRequest = serde_json::from_str(json).expect("deserialize should succeed");

        assert_eq!(request.request_id, "01968f1e-1182-71c4-9d40-172db9be82af");
        assert_eq!(request.side, Side::Buy);
        assert_eq!(request.price, Decimal::new(5, 1));
    }

    #[test]
    fn deserialize_quote() {
        let json = r#"{
            "quoteId": "0196f484-9fbd-74c1-bfc1-75ac21c1cf84",
            "requestId": "01968f1e-1182-71c4-9d40-172db9be82af",
            "user": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
            "proxy": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
            "market": "0x37a6a2dd9f3469495d9ec2467b0a764c5905371a294ce544bc3b2c944eb3e84a",
            "token": "34097058504275310827233323421517291090691602969494795225921954353603704046623",
            "complement": "32868290514114487320702931554221558599637733115139769311383916145370132125101",
            "side": "BUY",
            "sizeIn": 100,
            "sizeOut": 50,
            "price": 0.5
        }"#;
        let quote: Quote = serde_json::from_str(json).expect("deserialize should succeed");

        assert_eq!(quote.quote_id, "0196f484-9fbd-74c1-bfc1-75ac21c1cf84");
        assert_eq!(quote.request_id, "01968f1e-1182-71c4-9d40-172db9be82af");
    }

    #[test]
    fn deserialize_rfq_page() {
        let json = r#"{
            "data": [
                {
                    "requestId": "01968f1e-1182-71c4-9d40-172db9be82af",
                    "user": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
                    "proxy": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
                    "market": "0x37a6a2dd9f3469495d9ec2467b0a764c5905371a294ce544bc3b2c944eb3e84a",
                    "token": "34097058504275310827233323421517291090691602969494795225921954353603704046623",
                    "complement": "32868290514114487320702931554221558599637733115139769311383916145370132125101",
                    "side": "BUY",
                    "sizeIn": 100,
                    "sizeOut": 50,
                    "price": 0.5,
                    "expiry": 1746159634
                }
            ],
            "next_cursor": "LTE=",
            "limit": 100,
            "count": 1
        }"#;
        let page: RfqPage<RfqRequest> =
            serde_json::from_str(json).expect("deserialize should succeed");

        assert_eq!(page.count, 1);
        assert_eq!(page.limit, 100);
        assert_eq!(page.next_cursor, "LTE=");
        assert_eq!(page.data.len(), 1);
    }
}
