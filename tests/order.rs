#![allow(
    clippy::unwrap_used,
    reason = "Do not need additional syntax for setting up tests, and https://github.com/rust-lang/rust-clippy/issues/13981"
)]

mod common;

use std::str::FromStr as _;

use alloy::primitives::{Address, U256};
use chrono::{DateTime, Utc};
use httpmock::MockServer;
use polymarket_client_sdk::clob::types::{
    Amount, OrderSummary, OrderType, Side, SignatureType, TickSize,
};
use reqwest::StatusCode;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::common::{
    TOKEN_1, USDC_DECIMALS, create_authenticated, ensure_requirements, to_decimal,
};

/// Tests for the lifecycle of a [`Client`] as it moves from [`Unauthenticated`] to [`Authenticated`]
mod lifecycle {
    use alloy::primitives::address;
    use alloy::signers::Signer as _;
    use alloy::signers::local::LocalSigner;
    use polymarket_client_sdk::POLYGON;
    use polymarket_client_sdk::clob::{Client, Config};
    use polymarket_client_sdk::error::Validation;
    use serde_json::json;

    use super::*;
    use crate::common::{API_KEY, PASSPHRASE, POLY_ADDRESS, PRIVATE_KEY, SECRET};

    #[tokio::test]
    async fn order_parameters_should_reset_on_new_order() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        ensure_requirements(&server, "1", TickSize::Tenth);
        ensure_requirements(&server, "2", TickSize::Thousandth);

        let signable_order = client
            .limit_order()
            .token_id("1")
            .size(Decimal::ONE_HUNDRED)
            .price(dec!(0.1))
            .nonce(1)
            .side(Side::Buy)
            .build()
            .await?;

        let signable_order_2 = client
            .limit_order()
            .token_id("2")
            .price(dec!(0.512))
            .size(Decimal::ONE_HUNDRED)
            .side(Side::Buy)
            .build()
            .await?;

        assert_eq!(signable_order.order.nonce, U256::from(1));
        assert_eq!(signable_order_2.order.nonce, U256::ZERO);
        assert_ne!(signable_order, signable_order_2);

        Ok(())
    }

    #[tokio::test]
    async fn client_order_fields_should_persist_new_order() -> anyhow::Result<()> {
        let server = MockServer::start();
        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/auth/derive-api-key")
                .header(POLY_ADDRESS, signer.address().to_string().to_lowercase());
            then.status(StatusCode::OK).json_body(json!({
                "apiKey": API_KEY.to_string(),
                "passphrase": PASSPHRASE,
                "secret": SECRET
            }));
        });

        let client = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .salt_generator(|| 1)
            .authenticate()
            .await?;

        ensure_requirements(&server, "1", TickSize::Tenth);
        ensure_requirements(&server, "2", TickSize::Thousandth);

        let signable_order = client
            .limit_order()
            .token_id("1")
            .size(Decimal::ONE_HUNDRED)
            .price(dec!(0.1))
            .nonce(1)
            .side(Side::Buy)
            .build()
            .await?;

        let signable_order_2 = client
            .limit_order()
            .token_id("2")
            .price(dec!(0.512))
            .size(Decimal::ONE_HUNDRED)
            .side(Side::Buy)
            .build()
            .await?;

        assert_eq!(signable_order.order.salt, U256::from(1));
        assert_eq!(signable_order_2.order.salt, U256::from(1));
        assert_ne!(signable_order, signable_order_2);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn client_order_fields_should_reset_on_deauthenticate() -> anyhow::Result<()> {
        let server = MockServer::start();
        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/auth/derive-api-key")
                .header(POLY_ADDRESS, signer.address().to_string().to_lowercase());
            then.status(StatusCode::OK).json_body(json!({
                "apiKey": API_KEY.to_string(),
                "passphrase": PASSPHRASE,
                "secret": SECRET
            }));
        });

        let client = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .salt_generator(|| 1)
            .funder(address!("0xd1615A7B6146cDbA40a559eC876A3bcca4050890"))
            .signature_type(SignatureType::GnosisSafe)
            .authenticate()
            .await?;

        ensure_requirements(&server, "1", TickSize::Tenth);

        let signable_order = client
            .limit_order()
            .token_id("1")
            .size(Decimal::ONE_HUNDRED)
            .price(dec!(0.1))
            .nonce(1)
            .side(Side::Buy)
            .build()
            .await?;

        assert_eq!(signable_order.order.salt, U256::from(1));
        assert_eq!(
            signable_order.order.signatureType,
            SignatureType::GnosisSafe as u8
        );

        let client = client
            .deauthenticate()?
            .authentication_builder(&signer)
            .salt_generator(|| 123)
            .authenticate()
            .await?;

        let signable_order = client
            .limit_order()
            .token_id("1")
            .size(Decimal::ONE_HUNDRED)
            .price(dec!(0.1))
            .nonce(1)
            .side(Side::Buy)
            .build()
            .await?;

        assert_eq!(signable_order.order.salt, U256::from(123));
        assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);
        assert_eq!(signable_order.order.maker, signer.address());

        mock.assert_calls(2);

        Ok(())
    }

    #[tokio::test]
    async fn client_with_funder_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();

        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/auth/derive-api-key")
                .header(POLY_ADDRESS, signer.address().to_string().to_lowercase());
            then.status(StatusCode::OK).json_body(json!({
                "apiKey": API_KEY.to_string(),
                "passphrase": PASSPHRASE,
                "secret": SECRET
            }));
        });

        let funder = address!("0xaDEFf2158d668f64308C62ef227C5CcaCAAf976D");
        let client = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .funder(funder)
            .signature_type(SignatureType::Proxy)
            .authenticate()
            .await?;

        mock.assert();

        ensure_requirements(&server, "1", TickSize::Tenth);

        let signable_order = client
            .limit_order()
            .token_id("1")
            .size(Decimal::ONE_HUNDRED)
            .price(dec!(0.1))
            .nonce(1)
            .side(Side::Buy)
            .build()
            .await?;

        assert_eq!(signable_order.order.maker, funder);
        assert_eq!(
            signable_order.order.signatureType,
            SignatureType::Proxy as u8
        );
        assert_eq!(signable_order.order.nonce, U256::from(1));
        assert_eq!(signable_order.order.side, Side::Buy as u8);
        assert_ne!(signable_order.order.maker, signable_order.order.signer);

        ensure_requirements(&server, "2", TickSize::Tenth);

        let signable_order = client
            .limit_order()
            .token_id("2")
            .size(Decimal::TEN)
            .price(dec!(0.2))
            .nonce(2)
            .side(Side::Sell)
            .build()
            .await?;

        // Funder and signature type propagate from setting on the auth builder
        assert_eq!(signable_order.order.maker, funder);
        assert_eq!(
            signable_order.order.signatureType,
            SignatureType::Proxy as u8
        );
        assert_eq!(signable_order.order.nonce, U256::from(2));
        assert_eq!(signable_order.order.side, Side::Sell as u8);
        assert_ne!(signable_order.order.maker, signable_order.order.signer);

        Ok(())
    }

    #[tokio::test]
    async fn client_logged_in_then_out_should_reset_funder_and_signature_type() -> anyhow::Result<()>
    {
        let server = MockServer::start();

        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/auth/derive-api-key")
                .header(POLY_ADDRESS, signer.address().to_string().to_lowercase());
            then.status(StatusCode::OK).json_body(json!({
                "apiKey": API_KEY.to_string(),
                "passphrase": PASSPHRASE,
                "secret": SECRET
            }));
        });

        let funder = address!("0xaDEFf2158d668f64308C62ef227C5CcaCAAf976D");
        let client = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .funder(funder)
            .signature_type(SignatureType::Proxy)
            .authenticate()
            .await?;

        mock.assert();

        ensure_requirements(&server, "1", TickSize::Tenth);

        let signable_order = client
            .limit_order()
            .token_id("1")
            .size(Decimal::ONE_HUNDRED)
            .price(dec!(0.1))
            .nonce(1)
            .side(Side::Buy)
            .build()
            .await?;

        assert_eq!(signable_order.order.maker, funder);
        assert_eq!(
            signable_order.order.signatureType,
            SignatureType::Proxy as u8
        );
        assert_eq!(signable_order.order.nonce, U256::from(1));
        assert_eq!(signable_order.order.side, Side::Buy as u8);
        assert_ne!(signable_order.order.maker, signable_order.order.signer);

        ensure_requirements(&server, "2", TickSize::Tenth);

        client.deauthenticate()?;
        let client = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .authenticate()
            .await?;

        let signable_order = client
            .limit_order()
            .token_id("2")
            .size(Decimal::TEN)
            .price(dec!(0.2))
            .nonce(2)
            .side(Side::Sell)
            .build()
            .await?;

        // Funder and signature type propagate from setting on the auth builder
        assert_eq!(signable_order.order.maker, signer.address());
        assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);
        assert_eq!(signable_order.order.nonce, U256::from(2));
        assert_eq!(signable_order.order.side, Side::Sell as u8);
        assert_eq!(signable_order.order.maker, signable_order.order.signer);

        Ok(())
    }

    #[tokio::test]
    async fn incompatible_funder_and_signature_types_should_fail() -> anyhow::Result<()> {
        let server = MockServer::start();

        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));

        let funder = address!("0xaDEFf2158d668f64308C62ef227C5CcaCAAf976D");
        let err = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .funder(funder)
            .signature_type(SignatureType::Eoa)
            .authenticate()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(
            msg,
            "Cannot have a funder address with a Eoa signature type"
        );

        let err = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .signature_type(SignatureType::GnosisSafe)
            .authenticate()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(
            msg,
            "Must have a funder address with a GnosisSafe signature type"
        );

        let err = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .funder(Address::ZERO)
            .signature_type(SignatureType::GnosisSafe)
            .authenticate()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(
            msg,
            "Cannot have a zero funder address with a GnosisSafe signature type"
        );

        Ok(())
    }

    #[tokio::test]
    async fn signer_with_no_chain_id_should_fail() -> anyhow::Result<()> {
        let server = MockServer::start();

        let signer = LocalSigner::from_str(PRIVATE_KEY)?;

        let err = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .authenticate()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(
            msg,
            "Chain id not set, be sure to provide one on the signer"
        );

        Ok(())
    }

    #[tokio::test]
    async fn signer_with_unsupported_chain_id_should_fail() -> anyhow::Result<()> {
        let server = MockServer::start();

        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(1));

        let err = Client::new(&server.base_url(), Config::default())?
            .authentication_builder(&signer)
            .authenticate()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(msg, "Only Polygon and AMOY are supported, got 1");

        Ok(())
    }
}

mod limit {
    use polymarket_client_sdk::error::Validation;

    use super::*;

    #[tokio::test]
    async fn should_fail_on_expiration_for_gtc() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        ensure_requirements(&server, TOKEN_1, TickSize::Tenth);

        let err = client
            .limit_order()
            .token_id(TOKEN_1)
            .price(dec!(0.5))
            .size(dec!(21.04))
            .side(Side::Buy)
            .nonce(123)
            .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(msg, "Only GTD orders may have a non-zero expiration");

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_on_missing_fields() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        ensure_requirements(&server, TOKEN_1, TickSize::Tenth);

        let err = client
            .limit_order()
            .token_id(TOKEN_1)
            .size(dec!(21.04))
            .side(Side::Buy)
            .nonce(123)
            .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(msg, "Unable to build Order due to missing price");

        let err = client
            .limit_order()
            .token_id(TOKEN_1)
            .price(dec!(0.5))
            .side(Side::Buy)
            .nonce(123)
            .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(msg, "Unable to build Order due to missing size");

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_on_too_granular_of_a_price() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

        let err = client
            .limit_order()
            .token_id(TOKEN_1)
            .price(dec!(0.005))
            .size(dec!(21.04))
            .side(Side::Buy)
            .nonce(123)
            .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(
            msg,
            "Unable to build Order: Price 0.005 has 3 decimal places. Minimum tick size 0.01 has 2 decimal places. Price decimal places <= minimum tick size decimal places"
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_on_negative_price_and_size() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        ensure_requirements(&server, TOKEN_1, TickSize::Tenth);

        let err = client
            .limit_order()
            .token_id(TOKEN_1)
            .price(dec!(-0.5))
            .size(dec!(21.04))
            .side(Side::Buy)
            .nonce(123)
            .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(msg, "Unable to build Order due to negative price -0.5");

        let err = client
            .limit_order()
            .token_id(TOKEN_1)
            .price(dec!(0.5))
            .size(dec!(-21.04))
            .side(Side::Buy)
            .nonce(123)
            .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(msg, "Unable to build Order due to negative size -21.04");

        Ok(())
    }

    mod buy {
        use super::*;

        #[tokio::test]
        async fn should_succeed_0_1() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Tenth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.5))
                .size(dec!(21.04))
                .side(Side::Buy)
                .order_type(OrderType::GTD)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(maker_amount) / to_decimal(taker_amount);
            assert_eq!(price, dec!(0.50));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(10_520_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(21_040_000));
            assert_eq!(signable_order.order.expiration, U256::from(50000));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Buy as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_01() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.56))
                .size(dec!(21.04))
                .side(Side::Buy)
                .order_type(OrderType::GTD)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(maker_amount) / to_decimal(taker_amount);
            assert_eq!(price, dec!(0.56));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(11_782_400));
            assert_eq!(signable_order.order.takerAmount, U256::from(21_040_000));
            assert_eq!(signable_order.order.expiration, U256::from(50000));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Buy as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_001() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Thousandth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.056))
                .size(dec!(21.04))
                .side(Side::Buy)
                .order_type(OrderType::GTD)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(maker_amount) / to_decimal(taker_amount);
            assert_eq!(price, dec!(0.056));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(1_178_240));
            assert_eq!(signable_order.order.takerAmount, U256::from(21_040_000));
            assert_eq!(signable_order.order.expiration, U256::from(50000));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Buy as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_0001() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::TenThousandth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.0056))
                .size(dec!(21.04))
                .side(Side::Buy)
                .order_type(OrderType::GTD)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(maker_amount) / to_decimal(taker_amount);
            assert_eq!(price, dec!(0.0056));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(117_824));
            assert_eq!(signable_order.order.takerAmount, U256::from(21_040_000));
            assert_eq!(signable_order.order.expiration, U256::from(50000));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Buy as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn buy_should_succeed_decimal_accuracy() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.24))
                .size(dec!(15))
                .side(Side::Buy)
                .build()
                .await?;

            assert_eq!(signable_order.order.makerAmount, U256::from(3_600_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(15_000_000));

            Ok(())
        }

        #[tokio::test]
        async fn buy_should_succeed_decimal_accuracy_2() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.82))
                .size(dec!(101))
                .side(Side::Buy)
                .build()
                .await?;

            assert_eq!(signable_order.order.makerAmount, U256::from(82_820_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(101_000_000));

            Ok(())
        }

        #[tokio::test]
        async fn buy_should_fail_on_too_granular_of_lot_size() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let err = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.78))
                .size(dec!(12.8205))
                .side(Side::Buy)
                .build()
                .await
                .unwrap_err();
            let validation_err = err.downcast_ref::<Validation>().unwrap();

            assert_eq!(
                validation_err.reason,
                "Unable to build Order: Size 12.8205 has 4 decimal places. Maximum lot size is 2"
            );

            Ok(())
        }

        #[tokio::test]
        async fn buy_should_succeed_decimal_accuracy_4() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.58))
                .size(dec!(18233.33))
                .side(Side::Buy)
                .build()
                .await?;

            assert_eq!(
                signable_order.order.makerAmount,
                U256::from(10_575_331_400_u64)
            );
            assert_eq!(
                signable_order.order.takerAmount,
                U256::from(18_233_330_000_u64)
            );

            Ok(())
        }
    }

    mod sell {
        use super::*;

        #[tokio::test]
        async fn should_succeed_0_1() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Tenth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.5))
                .size(dec!(21.04))
                .side(Side::Sell)
                .order_type(OrderType::GTD)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(taker_amount) / to_decimal(maker_amount);
            assert_eq!(price, dec!(0.50));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(21_040_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(10_520_000));
            assert_eq!(signable_order.order.expiration, U256::from(50000));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Sell as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_01() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.56))
                .size(dec!(21.04))
                .side(Side::Sell)
                .order_type(OrderType::GTD)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(taker_amount) / to_decimal(maker_amount);
            assert_eq!(price, dec!(0.56));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(21_040_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(11_782_400));
            assert_eq!(signable_order.order.expiration, U256::from(50000));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Sell as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_001() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Thousandth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.056))
                .size(dec!(21.04))
                .side(Side::Sell)
                .order_type(OrderType::GTD)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(taker_amount) / to_decimal(maker_amount);
            assert_eq!(price, dec!(0.056));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(21_040_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(1_178_240));
            assert_eq!(signable_order.order.expiration, U256::from(50000));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Sell as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_0001() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::TenThousandth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.0056))
                .size(dec!(21.04))
                .side(Side::Sell)
                .order_type(OrderType::GTD)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(taker_amount) / to_decimal(maker_amount);
            assert_eq!(price, dec!(0.0056));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(21_040_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(117_824));
            assert_eq!(signable_order.order.expiration, U256::from(50000));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Sell as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn sell_should_succeed_decimal_accuracy() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.24))
                .size(dec!(15))
                .side(Side::Sell)
                .build()
                .await?;

            assert_eq!(signable_order.order.makerAmount, U256::from(15_000_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(3_600_000));

            Ok(())
        }

        #[tokio::test]
        async fn sell_should_succeed_decimal_accuracy_2() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.82))
                .size(dec!(101))
                .side(Side::Sell)
                .build()
                .await?;

            assert_eq!(signable_order.order.makerAmount, U256::from(101_000_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(82_820_000));

            Ok(())
        }

        #[tokio::test]
        async fn sell_should_succeed_decimal_accuracy_3() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let err = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.78))
                .size(dec!(12.8205))
                .side(Side::Sell)
                .build()
                .await
                .unwrap_err();

            let validation_err = err.downcast_ref::<Validation>().unwrap();

            assert_eq!(
                validation_err.reason,
                "Unable to build Order: Size 12.8205 has 4 decimal places. Maximum lot size is 2"
            );

            Ok(())
        }

        #[tokio::test]
        async fn sell_should_succeed_decimal_accuracy_4() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.39))
                .size(dec!(2435.89))
                .side(Side::Sell)
                .build()
                .await?;

            assert_eq!(
                signable_order.order.makerAmount,
                U256::from(2_435_890_000_u64)
            );
            assert_eq!(signable_order.order.takerAmount, U256::from(949_997_100));

            Ok(())
        }

        #[tokio::test]
        async fn sell_should_succeed_decimal_accuracy_5() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);

            let signable_order = client
                .limit_order()
                .token_id(TOKEN_1)
                .price(dec!(0.43))
                .size(dec!(19.1))
                .side(Side::Sell)
                .build()
                .await?;

            assert_eq!(signable_order.order.makerAmount, U256::from(19_100_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(8_213_000));

            Ok(())
        }
    }

    #[tokio::test]
    async fn should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let token_id = "123".to_owned();
        let token_id_2 = "456".to_owned();

        ensure_requirements(&server, &token_id, TickSize::Thousandth);
        ensure_requirements(&server, &token_id_2, TickSize::Hundredth);

        assert_eq!(
            client.tick_size(&token_id).await?.minimum_tick_size,
            TickSize::Thousandth
        );

        let signable_order = client
            .limit_order()
            .token_id(token_id.clone())
            .price(dec!(0.512))
            .size(Decimal::ONE_HUNDRED)
            .side(Side::Buy)
            .build()
            .await?;

        assert_eq!(signable_order.order.maker, client.address());
        assert_eq!(signable_order.order.taker, Address::ZERO);
        assert_eq!(signable_order.order.tokenId.to_string(), token_id);
        assert_eq!(signable_order.order.makerAmount, U256::from(51_200_000));
        assert_eq!(signable_order.order.takerAmount, U256::from(100_000_000));
        assert_eq!(signable_order.order.expiration, U256::ZERO);
        assert_eq!(signable_order.order.nonce, U256::ZERO);
        assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
        assert_eq!(signable_order.order.side, Side::Buy as u8);
        assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

        let signable_order = client
            .limit_order()
            .token_id(token_id_2.clone())
            .price(dec!(0.78))
            .size(dec!(12.82))
            .side(Side::Buy)
            .build()
            .await?;

        assert_eq!(signable_order.order.maker, client.address());
        assert_eq!(signable_order.order.taker, Address::ZERO);
        assert_eq!(signable_order.order.tokenId.to_string(), token_id_2);
        assert_eq!(signable_order.order.makerAmount, U256::from(9_999_600));
        assert_eq!(signable_order.order.takerAmount, U256::from(12_820_000));
        assert_eq!(signable_order.order.expiration, U256::ZERO);
        assert_eq!(signable_order.order.nonce, U256::ZERO);
        assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
        assert_eq!(signable_order.order.side, Side::Buy as u8);
        assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

        let _order = client
            .limit_order()
            .token_id(token_id_2.clone())
            .order_type(OrderType::GTC)
            .price(dec!(0.78))
            .size(dec!(12.82))
            .side(Side::Sell)
            .build()
            .await?;

        Ok(())
    }
}

mod market {
    use polymarket_client_sdk::clob::types::OrderSummaryBuilder;
    use polymarket_client_sdk::error::Validation;
    use serde_json::json;

    use super::*;

    fn ensure_requirements_for_market_price(
        server: &MockServer,
        token_id: &str,
        bids: &[OrderSummary],
        asks: &[OrderSummary],
    ) {
        let minimum_tick_size = TickSize::Tenth;

        server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/book")
                .query_param("token_id", token_id);
            then.status(StatusCode::OK).json_body(json!({
                "market": "market",
                "asset_id": token_id,
                "timestamp": "1000",
                "bids": bids,
                "asks": asks,
                "min_order_size": "5",
                "neg_risk": false,
                "tick_size": minimum_tick_size.as_decimal(),
            }));
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/tick-size")
                .query_param("token_id", token_id);
            then.status(StatusCode::OK).json_body(json!({
                "minimum_tick_size": minimum_tick_size.as_decimal(),
            }));
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/fee-rate")
                .query_param("token_id", token_id);
            then.status(StatusCode::OK)
                .json_body(json!({ "base_fee": 0 }));
        });
    }

    mod buy {
        use super::*;

        mod fok {
            use polymarket_client_sdk::error::Validation;

            use super::*;

            #[tokio::test]
            async fn should_fail_on_no_asks() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(&server, TOKEN_1, &[], &[]);

                let err = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .order_type(OrderType::FOK)
                    .build()
                    .await
                    .unwrap_err();
                let msg = &err.downcast_ref::<Validation>().unwrap().reason;

                assert_eq!(
                    msg,
                    "No opposing orders for 1 which means there is no market price"
                );

                Ok(())
            }

            #[tokio::test]
            async fn should_fail_on_insufficient_liquidity() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let err = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .order_type(OrderType::FOK)
                    .build()
                    .await
                    .unwrap_err();
                let msg = &err.downcast_ref::<Validation>().unwrap().reason;

                assert_eq!(msg, "Insufficient liquidity to fill order for 1 at 100");

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .order_type(OrderType::FOK)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(maker_amount) / to_decimal(taker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(signable_order.order.maker, client.address());
                assert_eq!(signable_order.order.signer, client.address());
                assert_eq!(signable_order.order.taker, Address::ZERO);
                assert_eq!(signable_order.order.tokenId, U256::ONE);
                assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(signable_order.order.takerAmount, U256::from(200_000_000)); // 200 `TOKEN_1` tokens
                assert_eq!(signable_order.order.expiration, U256::ZERO);
                assert_eq!(signable_order.order.nonce, U256::ZERO);
                assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
                assert_eq!(signable_order.order.side, Side::Buy as u8);
                assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed2() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(dec!(200))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .order_type(OrderType::FOK)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(maker_amount) / to_decimal(taker_amount);
                assert_eq!(price, dec!(0.4));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(taker_amount, U256::from(250_000_000)); // 250 `TOKEN_1` tokens

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_3() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(dec!(120))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.2))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .order_type(OrderType::FOK)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(maker_amount) / to_decimal(taker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(taker_amount, U256::from(200_000_000)); // 200 `TOKEN_1` tokens

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_4() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(dec!(200))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .order_type(OrderType::FOK)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(maker_amount) / to_decimal(taker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(taker_amount, U256::from(200_000_000)); // 200 `TOKEN_1` tokens

                Ok(())
            }
        }

        mod fak {
            use super::*;

            #[tokio::test]
            async fn should_fail_on_no_asks() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(&server, TOKEN_1, &[], &[]);

                let err = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .build()
                    .await
                    .unwrap_err();
                let msg = &err.downcast_ref::<Validation>().unwrap().reason;

                assert_eq!(
                    msg,
                    "No opposing orders for 1 which means there is no market price"
                );

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(maker_amount) / to_decimal(taker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(signable_order.order.maker, client.address());
                assert_eq!(signable_order.order.signer, client.address());
                assert_eq!(signable_order.order.taker, Address::ZERO);
                assert_eq!(signable_order.order.tokenId, U256::ONE);
                assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(signable_order.order.takerAmount, U256::from(200_000_000)); // 200 `TOKEN_1` tokens
                assert_eq!(signable_order.order.expiration, U256::ZERO);
                assert_eq!(signable_order.order.nonce, U256::ZERO);
                assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
                assert_eq!(signable_order.order.side, Side::Buy as u8);
                assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_2() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(maker_amount) / to_decimal(taker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(taker_amount, U256::from(200_000_000)); // 200 `TOKEN_1` tokens

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_3() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(dec!(200))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(maker_amount) / to_decimal(taker_amount);
                assert_eq!(price, dec!(0.4));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(taker_amount, U256::from(250_000_000)); // 250 `TOKEN_1` tokens

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_4() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(dec!(120))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(maker_amount) / to_decimal(taker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(taker_amount, U256::from(200_000_000)); // 200 `TOKEN_1` tokens

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_5() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[],
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(dec!(200))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                    .side(Side::Buy)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(maker_amount) / to_decimal(taker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(taker_amount, U256::from(200_000_000)); // 200 `TOKEN_1` tokens

                Ok(())
            }
        }

        #[tokio::test]
        async fn should_succeed_0_1() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Tenth);
            // Always gives a market price of 0.5 for 100
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[],
                &[OrderSummaryBuilder::default()
                    .price(dec!(0.5))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?],
            );

            let signable_order = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                .side(Side::Buy)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(maker_amount) / to_decimal(taker_amount);
            assert_eq!(price, dec!(0.50));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(200_000_000));
            assert_eq!(signable_order.order.expiration, U256::from(0));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Buy as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_01() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);
            // Always gives a market price of 0.56 for 100
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[],
                &[OrderSummaryBuilder::default()
                    .price(dec!(0.56))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?],
            );

            let signable_order = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                .side(Side::Buy)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = (to_decimal(maker_amount) / to_decimal(taker_amount))
                .trunc_with_scale(USDC_DECIMALS);
            assert_eq!(price, dec!(0.56));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(178_571_400));
            assert_eq!(signable_order.order.expiration, U256::from(0));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Buy as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_001() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Thousandth);
            // Always gives a market price of 0.056 for 100
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[],
                &[OrderSummaryBuilder::default()
                    .price(dec!(0.056))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?],
            );

            let signable_order = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                .side(Side::Buy)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = (to_decimal(maker_amount) / to_decimal(taker_amount))
                .trunc_with_scale(USDC_DECIMALS);
            assert_eq!(price, dec!(0.056));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(1_785_714_280));
            assert_eq!(signable_order.order.expiration, U256::from(0));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Buy as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_0001() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::TenThousandth);
            // Always gives a market price of 0.0056 for 100
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[],
                &[OrderSummaryBuilder::default()
                    .price(dec!(0.0056))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?],
            );

            let signable_order = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
                .side(Side::Buy)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = (to_decimal(maker_amount) / to_decimal(taker_amount))
                .trunc_with_scale(USDC_DECIMALS);
            assert_eq!(price, dec!(0.0056));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000));
            assert_eq!(
                signable_order.order.takerAmount,
                U256::from(17_857_142_857_u64)
            );
            assert_eq!(signable_order.order.expiration, U256::from(0));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Buy as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn market_buy_with_shares_fok_should_fail_on_no_asks() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements_for_market_price(&server, TOKEN_1, &[], &[]);

            let err = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                .side(Side::Buy)
                .order_type(OrderType::FOK)
                .build()
                .await
                .unwrap_err();

            let msg = &err
                .downcast_ref::<polymarket_client_sdk::error::Validation>()
                .unwrap()
                .reason;
            assert_eq!(
                msg,
                "No opposing orders for 1 which means there is no market price"
            );
            Ok(())
        }

        #[tokio::test]
        async fn market_buy_with_shares_fok_should_fail_on_insufficient_liquidity()
        -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            // only 50 shares available on asks
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[],
                &[OrderSummaryBuilder::default()
                    .price(dec!(0.5))
                    .size(dec!(50))
                    .build()?],
            );

            let err = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                .side(Side::Buy)
                .order_type(OrderType::FOK)
                .build()
                .await
                .unwrap_err();

            let msg = &err
                .downcast_ref::<polymarket_client_sdk::error::Validation>()
                .unwrap()
                .reason;
            assert_eq!(msg, "Insufficient liquidity to fill order for 1 at 100");
            Ok(())
        }

        #[tokio::test]
        async fn market_buy_with_shares_should_succeed_and_encode_maker_as_usdc()
        -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            // cutoff price should end at 0.4 for 250 shares
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[],
                &[
                    OrderSummaryBuilder::default()
                        .price(dec!(0.5))
                        .size(dec!(100))
                        .build()?,
                    OrderSummaryBuilder::default()
                        .price(dec!(0.4))
                        .size(dec!(300))
                        .build()?,
                ],
            );

            let signable_order = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::shares(dec!(250))?)
                .side(Side::Buy)
                .order_type(OrderType::FOK)
                .build()
                .await?;

            // maker = USDC, taker = shares
            assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000)); // 250 * 0.4 = 100
            assert_eq!(signable_order.order.takerAmount, U256::from(250_000_000));
            Ok(())
        }
    }

    mod sell {
        use super::*;

        mod fok {
            use super::*;

            #[tokio::test]
            async fn should_fail_on_no_bids() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(&server, TOKEN_1, &[], &[]);

                let err = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                    .side(Side::Sell)
                    .order_type(OrderType::FOK)
                    .build()
                    .await
                    .unwrap_err();
                let msg = &err.downcast_ref::<Validation>().unwrap().reason;

                assert_eq!(
                    msg,
                    "No opposing orders for 1 which means there is no market price"
                );

                Ok(())
            }

            #[tokio::test]
            async fn should_fail_on_insufficient_liquidity() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::TEN)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::TEN)
                            .build()?,
                    ],
                    &[],
                );

                let err = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                    .side(Side::Sell)
                    .order_type(OrderType::FOK)
                    .build()
                    .await
                    .unwrap_err();
                let msg = &err.downcast_ref::<Validation>().unwrap().reason;

                assert_eq!(msg, "Insufficient liquidity to fill order for 1 at 100");

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                    .side(Side::Sell)
                    .order_type(OrderType::FOK)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(signable_order.order.maker, client.address());
                assert_eq!(signable_order.order.signer, client.address());
                assert_eq!(signable_order.order.taker, Address::ZERO);
                assert_eq!(signable_order.order.tokenId, U256::ONE);
                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(50_000_000)); // 50 USDC
                assert_eq!(signable_order.order.expiration, U256::ZERO);
                assert_eq!(signable_order.order.nonce, U256::ZERO);
                assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
                assert_eq!(signable_order.order.side, Side::Sell as u8);
                assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_2() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(dec!(300))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::TEN)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                    .side(Side::Sell)
                    .order_type(OrderType::FOK)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.4));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(40_000_000)); // 40 USDC

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_3() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(dec!(200))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::TEN)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(dec!(200))?)
                    .side(Side::Sell)
                    .order_type(OrderType::FOK)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.4));

                assert_eq!(maker_amount, U256::from(200_000_000)); // 200 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(80_000_000)); // 80 USDC

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_4() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(dec!(300))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(dec!(300))?)
                    .side(Side::Sell)
                    .order_type(OrderType::FOK)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.3));

                assert_eq!(maker_amount, U256::from(300_000_000)); // 300 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(90_000_000)); // 90 USDC

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_5() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(dec!(334))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(dec!(300))?)
                    .side(Side::Sell)
                    .order_type(OrderType::FOK)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.3));

                assert_eq!(maker_amount, U256::from(300_000_000)); // 300 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(90_000_000)); // 90 USDC

                Ok(())
            }
        }

        mod fak {
            use super::*;

            #[tokio::test]
            async fn should_fail_on_no_bids() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(&server, TOKEN_1, &[], &[]);

                let err = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                    .side(Side::Sell)
                    .build()
                    .await
                    .unwrap_err();
                let msg = &err.downcast_ref::<Validation>().unwrap().reason;

                assert_eq!(
                    msg,
                    "No opposing orders for 1 which means there is no market price"
                );

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::TEN)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::TEN)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                    .side(Side::Sell)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.4));

                assert_eq!(signable_order.order.maker, client.address());
                assert_eq!(signable_order.order.signer, client.address());
                assert_eq!(signable_order.order.taker, Address::ZERO);
                assert_eq!(signable_order.order.tokenId, U256::ONE);
                assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000)); // 100 USDC
                assert_eq!(signable_order.order.takerAmount, U256::from(40_000_000)); // 40 `TOKEN_1` tokens
                assert_eq!(signable_order.order.expiration, U256::ZERO);
                assert_eq!(signable_order.order.nonce, U256::ZERO);
                assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
                assert_eq!(signable_order.order.side, Side::Sell as u8);
                assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_2() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                    .side(Side::Sell)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(50_000_000)); // 50 USDC

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_3() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(dec!(300))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::TEN)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                    .side(Side::Sell)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.4));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(40_000_000)); // 40 USDC

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_4() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(dec!(200))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::TEN)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(dec!(200))?)
                    .side(Side::Sell)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.4));

                assert_eq!(maker_amount, U256::from(200_000_000)); // 200 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(80_000_000)); // 80 USDC

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_5() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(dec!(300))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                    .side(Side::Sell)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.5));

                assert_eq!(maker_amount, U256::from(100_000_000)); // 100 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(50_000_000)); // 50 USDC

                Ok(())
            }

            #[tokio::test]
            async fn should_succeed_6() -> anyhow::Result<()> {
                let server = MockServer::start();
                let client = create_authenticated(&server).await?;

                ensure_requirements_for_market_price(
                    &server,
                    TOKEN_1,
                    &[
                        OrderSummaryBuilder::default()
                            .price(dec!(0.3))
                            .size(dec!(334))
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.4))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                        OrderSummaryBuilder::default()
                            .price(dec!(0.5))
                            .size(Decimal::ONE_HUNDRED)
                            .build()?,
                    ],
                    &[],
                );

                let signable_order = client
                    .market_order()
                    .token_id(TOKEN_1)
                    .amount(Amount::shares(dec!(300))?)
                    .side(Side::Sell)
                    .build()
                    .await?;

                let maker_amount = signable_order.order.makerAmount;
                let taker_amount = signable_order.order.takerAmount;

                let price = to_decimal(taker_amount) / to_decimal(maker_amount);
                assert_eq!(price, dec!(0.3));

                assert_eq!(maker_amount, U256::from(300_000_000)); // 300 `TOKEN_1` tokens
                assert_eq!(taker_amount, U256::from(90_000_000)); // 90 USDC

                Ok(())
            }
        }

        #[tokio::test]
        async fn should_succeed_0_1() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Tenth);
            // Always gives a market price of 0.5 for 100
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[OrderSummaryBuilder::default()
                    .price(dec!(0.5))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?],
                &[],
            );

            let signable_order = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                .side(Side::Sell)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = to_decimal(taker_amount) / to_decimal(maker_amount);
            assert_eq!(price, dec!(0.50));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(50_000_000));
            assert_eq!(signable_order.order.expiration, U256::from(0));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Sell as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_01() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Hundredth);
            // Always gives a market price of 0.56 for 100
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[OrderSummaryBuilder::default()
                    .price(dec!(0.56))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?],
                &[],
            );

            let signable_order = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                .side(Side::Sell)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = (to_decimal(taker_amount) / to_decimal(maker_amount))
                .trunc_with_scale(USDC_DECIMALS);
            assert_eq!(price, dec!(0.56));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(56_000_000));
            assert_eq!(signable_order.order.expiration, U256::from(0));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Sell as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_001() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::Thousandth);
            // Always gives a market price of 0.056 for 100
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[OrderSummaryBuilder::default()
                    .price(dec!(0.056))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?],
                &[],
            );

            let signable_order = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                .side(Side::Sell)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = (to_decimal(taker_amount) / to_decimal(maker_amount))
                .trunc_with_scale(USDC_DECIMALS);
            assert_eq!(price, dec!(0.056));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(5_600_000));
            assert_eq!(signable_order.order.expiration, U256::from(0));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Sell as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }

        #[tokio::test]
        async fn should_succeed_0_0001() -> anyhow::Result<()> {
            let server = MockServer::start();
            let client = create_authenticated(&server).await?;

            ensure_requirements(&server, TOKEN_1, TickSize::TenThousandth);
            // Always gives a market price of 0.0056 for 100
            ensure_requirements_for_market_price(
                &server,
                TOKEN_1,
                &[OrderSummaryBuilder::default()
                    .price(dec!(0.0056))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?],
                &[],
            );

            let signable_order = client
                .market_order()
                .token_id(TOKEN_1)
                .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
                .side(Side::Sell)
                .nonce(123)
                .expiration(DateTime::<Utc>::from_str("1970-01-01T13:53:20Z").unwrap())
                .build()
                .await?;

            let maker_amount = signable_order.order.makerAmount;
            let taker_amount = signable_order.order.takerAmount;

            let price = (to_decimal(taker_amount) / to_decimal(maker_amount))
                .trunc_with_scale(USDC_DECIMALS);
            assert_eq!(price, dec!(0.0056));

            assert_eq!(signable_order.order.maker, client.address());
            assert_eq!(signable_order.order.signer, client.address());
            assert_eq!(signable_order.order.taker, Address::ZERO);
            assert_eq!(signable_order.order.tokenId.to_string(), TOKEN_1);
            assert_eq!(signable_order.order.makerAmount, U256::from(100_000_000));
            assert_eq!(signable_order.order.takerAmount, U256::from(560_000));
            assert_eq!(signable_order.order.expiration, U256::from(0));
            assert_eq!(signable_order.order.nonce, U256::from(123));
            assert_eq!(signable_order.order.feeRateBps, U256::ZERO);
            assert_eq!(signable_order.order.side, Side::Sell as u8);
            assert_eq!(signable_order.order.signatureType, SignatureType::Eoa as u8);

            Ok(())
        }
    }

    #[tokio::test]
    async fn should_fail_on_missing_required_fields() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let err = client
            .market_order()
            .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
            .side(Side::Buy)
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(msg, "Unable to build Order due to missing token ID");

        let err = client
            .market_order()
            .token_id(TOKEN_1)
            .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(msg, "Unable to build Order due to missing token side");

        let err = client
            .market_order()
            .token_id(TOKEN_1)
            .side(Side::Buy)
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(msg, "Unable to build Order due to missing amount");

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_on_gtc() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        ensure_requirements_for_market_price(&server, TOKEN_1, &[], &[]);

        let err = client
            .market_order()
            .token_id(TOKEN_1)
            .amount(Amount::shares(Decimal::ONE_HUNDRED)?)
            .side(Side::Sell)
            .order_type(OrderType::GTC)
            .build()
            .await
            .unwrap_err();
        let msg = &err.downcast_ref::<Validation>().unwrap().reason;

        assert_eq!(
            msg,
            "Cannot set an order type other than FAK/FOK for a market order"
        );

        Ok(())
    }

    #[tokio::test]
    async fn market_sell_with_usdc_should_fail() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        ensure_requirements_for_market_price(&server, TOKEN_1, &[], &[]);

        let err = client
            .market_order()
            .token_id(TOKEN_1)
            .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
            .side(Side::Sell)
            .build()
            .await
            .unwrap_err();
        let msg = &err
            .downcast_ref::<polymarket_client_sdk::error::Validation>()
            .unwrap()
            .reason;

        assert_eq!(msg, "Sell Orders must specify their `amount`s in shares");
        Ok(())
    }
}
