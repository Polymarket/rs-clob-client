#![cfg(all(feature = "clob", feature = "rfq"))]
#![allow(
    clippy::unwrap_used,
    reason = "Do not need additional syntax for setting up tests"
)]

mod common;

use alloy::primitives::Address;
use httpmock::MockServer;
use polymarket_client_sdk::clob::types::{
    AcceptRfqQuoteRequest, ApproveRfqOrderRequest, CancelRfqQuoteRequest, CancelRfqRequestRequest,
    CreateRfqQuoteRequest, CreateRfqRequestRequest, RfqQuotesRequest, RfqRequestsRequest, Side,
    SignatureType,
};
use reqwest::StatusCode;
use rust_decimal_macros::dec;
use serde_json::json;
use uuid::Uuid;

use crate::common::{POLY_ADDRESS, create_authenticated};

mod request {
    use std::str::FromStr as _;

    use polymarket_client_sdk::clob::types::request::Asset;
    use polymarket_client_sdk::types::U256;

    use super::*;

    #[tokio::test]
    async fn rfq_create_request_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/rfq/request")
                .header_exists(POLY_ADDRESS)
                .json_body(json!({
                    "assetIn": "12345",
                    "assetOut": "0",
                    "amountIn": "50000000",
                    "amountOut": "3000000",
                    "userType": 0
                }));
            then.status(StatusCode::OK).json_body(json!({
                "requestId": "0196464a-a1fa-75e6-821e-31aa0794f7ad",
                "expiry": 1_744_936_318
            }));
        });

        let request = CreateRfqRequestRequest::builder()
            .asset_in(Asset::Asset(U256::from_str("12345")?))
            .asset_out(Asset::Usdc)
            .amount_in(dec!(50000000))
            .amount_out(dec!(3000000))
            .user_type(SignatureType::Eoa)
            .build();

        let response = client.create_request(&request).await?;

        assert_eq!(response.request_id, "0196464a-a1fa-75e6-821e-31aa0794f7ad");
        assert_eq!(response.expiry, 1_744_936_318);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn rfq_cancel_request_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::DELETE)
                .path("/rfq/request")
                .header_exists(POLY_ADDRESS)
                .json_body(json!({
                    "requestId": "0196464a-a1fa-75e6-821e-31aa0794f7ad"
                }));
            then.status(StatusCode::OK).body("OK");
        });

        let request = CancelRfqRequestRequest::builder()
            .request_id("0196464a-a1fa-75e6-821e-31aa0794f7ad")
            .build();

        client.cancel_request(&request).await?;
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn rfq_requests_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/rfq/data/requests")
                .header_exists(POLY_ADDRESS);
            then.status(StatusCode::OK).json_body(json!({
                "data": [{
                    "requestId": "01968f1e-1182-71c4-9d40-172db9be82af",
                    "userAddress": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
                    "proxyAddress": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
                    "condition": "0x37a6a2dd9f3469495d9ec2467b0a764c5905371a294ce544bc3b2c944eb3e84a",
                    "token": "34097058504275310827233323421517291090691602969494795225921954353603704046623",
                    "complement": "32868290514114487320702931554221558599637733115139769311383916145370132125101",
                    "side": "BUY",
                    "sizeIn": 100,
                    "sizeOut": 50,
                    "price": 0.5,
                    "expiry": 1_746_159_634
                }],
                "next_cursor": "LTE=",
                "limit": 100,
                "count": 1
            }));
        });

        let request = RfqRequestsRequest::default();
        let response = client.requests(&request, None).await?;

        assert_eq!(response.count, 1);
        assert_eq!(response.data.len(), 1);
        assert_eq!(
            response.data[0].request_id,
            "01968f1e-1182-71c4-9d40-172db9be82af"
        );
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn rfq_requests_with_cursor_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/rfq/data/requests")
                .query_param("next_cursor", "abc123")
                .header_exists(POLY_ADDRESS);
            then.status(StatusCode::OK).json_body(json!({
                "data": [],
                "next_cursor": "",
                "limit": 100,
                "count": 0
            }));
        });

        let request = RfqRequestsRequest::default();
        let response = client.requests(&request, Some("abc123")).await?;

        assert_eq!(response.count, 0);
        mock.assert();

        Ok(())
    }
}

mod quote {
    use std::str::FromStr as _;

    use polymarket_client_sdk::clob::types::request::Asset;
    use polymarket_client_sdk::types::U256;

    use super::*;

    #[tokio::test]
    async fn rfq_create_quote_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/rfq/quote")
                .header_exists(POLY_ADDRESS)
                .json_body(json!({
                    "requestId": "01968f1e-1182-71c4-9d40-172db9be82af",
                    "assetIn": "0",
                    "assetOut": "12345",
                    "amountIn": "3000000",
                    "amountOut": "50000000",
                    "userType": 0
                }));
            then.status(StatusCode::OK).json_body(json!({
                "quoteId": "0196f484-9fbd-74c1-bfc1-75ac21c1cf84"
            }));
        });

        let request = CreateRfqQuoteRequest::builder()
            .request_id("01968f1e-1182-71c4-9d40-172db9be82af")
            .asset_in(Asset::Usdc)
            .asset_out(Asset::Asset(U256::from_str("12345")?))
            .amount_in(dec!(3000000))
            .amount_out(dec!(50000000))
            .user_type(SignatureType::Eoa)
            .build();

        let response = client.create_quote(&request).await?;

        assert_eq!(response.quote_id, "0196f484-9fbd-74c1-bfc1-75ac21c1cf84");
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn rfq_cancel_quote_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::DELETE)
                .path("/rfq/quote")
                .header_exists(POLY_ADDRESS)
                .json_body(json!({
                    "quoteId": "0196f484-9fbd-74c1-bfc1-75ac21c1cf84"
                }));
            then.status(StatusCode::OK).body("OK");
        });

        let request = CancelRfqQuoteRequest::builder()
            .quote_id("0196f484-9fbd-74c1-bfc1-75ac21c1cf84")
            .build();

        client.cancel_quote(&request).await?;
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn rfq_quotes_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/rfq/data/quotes")
                .header_exists(POLY_ADDRESS);
            then.status(StatusCode::OK).json_body(json!({
                "data": [{
                    "quoteId": "0196f484-9fbd-74c1-bfc1-75ac21c1cf84",
                    "requestId": "01968f1e-1182-71c4-9d40-172db9be82af",
                    "userAddress": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
                    "proxyAddress": "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5",
                    "condition": "0x37a6a2dd9f3469495d9ec2467b0a764c5905371a294ce544bc3b2c944eb3e84a",
                    "token": "34097058504275310827233323421517291090691602969494795225921954353603704046623",
                    "complement": "32868290514114487320702931554221558599637733115139769311383916145370132125101",
                    "side": "BUY",
                    "sizeIn": 100,
                    "sizeOut": 50,
                    "price": 0.5
                }],
                "next_cursor": "LTE=",
                "limit": 100,
                "count": 1
            }));
        });

        let request = RfqQuotesRequest::default();
        let response = client.quotes(&request, None).await?;

        assert_eq!(response.count, 1);
        assert_eq!(response.data.len(), 1);
        assert_eq!(
            response.data[0].quote_id,
            "0196f484-9fbd-74c1-bfc1-75ac21c1cf84"
        );
        mock.assert();

        Ok(())
    }
}

mod execution {
    use super::*;
    use crate::common::token_1;

    #[tokio::test]
    async fn rfq_accept_quote_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let maker: Address = "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5".parse()?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/rfq/request/accept")
                .header_exists(POLY_ADDRESS);
            then.status(StatusCode::OK).body("OK");
        });

        let request = AcceptRfqQuoteRequest::builder()
            .request_id("01968f1e-1182-71c4-9d40-172db9be82af")
            .quote_id("0196f484-9fbd-74c1-bfc1-75ac21c1cf84")
            .maker_amount(dec!(50000000))
            .taker_amount(dec!(3000000))
            .token_id(token_1())
            .maker(maker)
            .signer(maker)
            .taker(Address::ZERO)
            .nonce(0)
            .expiration(0)
            .side(Side::Buy)
            .fee_rate_bps(0)
            .signature("0x1234")
            .salt("123")
            .owner(Uuid::nil())
            .build();

        client.accept_quote(&request).await?;
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn rfq_approve_order_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let maker: Address = "0x6e0c80c90ea6c15917308f820eac91ce2724b5b5".parse()?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/rfq/quote/approve")
                .header_exists(POLY_ADDRESS);
            then.status(StatusCode::OK).json_body(json!({
                "tradeIds": ["019af0f7-eb77-764f-b40f-6de8a3562e12"]
            }));
        });

        let request = ApproveRfqOrderRequest::builder()
            .request_id("01968f1e-1182-71c4-9d40-172db9be82af")
            .quote_id("0196f484-9fbd-74c1-bfc1-75ac21c1cf84")
            .maker_amount(dec!(50000000))
            .taker_amount(dec!(3000000))
            .token_id(token_1())
            .maker(maker)
            .signer(maker)
            .taker(Address::ZERO)
            .nonce(0)
            .expiration(0)
            .side(Side::Buy)
            .fee_rate_bps(0)
            .signature("0x1234")
            .salt("123")
            .owner(Uuid::nil())
            .build();

        let response = client.approve_order(&request).await?;

        assert_eq!(response.trade_ids.len(), 1);
        assert_eq!(
            response.trade_ids[0],
            "019af0f7-eb77-764f-b40f-6de8a3562e12"
        );
        mock.assert();

        Ok(())
    }
}

mod error_handling {
    use std::str::FromStr as _;

    use polymarket_client_sdk::clob::types::request::Asset;
    use polymarket_client_sdk::error::Kind;
    use polymarket_client_sdk::types::U256;

    use super::*;

    #[tokio::test]
    async fn rfq_create_request_error_should_return_status() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST).path("/rfq/request");
            then.status(StatusCode::BAD_REQUEST)
                .body("Invalid request parameters");
        });

        let request = CreateRfqRequestRequest::builder()
            .asset_in(Asset::Asset(U256::from_str("12345")?))
            .asset_out(Asset::Usdc)
            .amount_in(dec!(50000000))
            .amount_out(dec!(3000000))
            .user_type(SignatureType::Eoa)
            .build();

        let result = client.create_request(&request).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), Kind::Status);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn rfq_cancel_request_error_should_return_status() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::DELETE).path("/rfq/request");
            then.status(StatusCode::NOT_FOUND).body("Request not found");
        });

        let request = CancelRfqRequestRequest::builder()
            .request_id("nonexistent")
            .build();

        let result = client.cancel_request(&request).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), Kind::Status);
        mock.assert();

        Ok(())
    }
}
