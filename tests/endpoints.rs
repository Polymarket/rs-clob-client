#![allow(
    clippy::unwrap_used,
    reason = "Do not need additional syntax for setting up tests, and https://github.com/rust-lang/rust-clippy/issues/13981"
)]

mod common;

use std::collections::HashMap;
use std::str::FromStr as _;

use alloy::primitives::U256;
use chrono::{DateTime, Utc};
use httpmock::MockServer;
use polymarket_client_sdk::POLYGON;
use polymarket_client_sdk::clob::types::{SignatureType, TickSize};
use polymarket_client_sdk::clob::{Client, Config};
use reqwest::StatusCode;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use uuid::Uuid;

use crate::common::{
    POLY_ADDRESS, POLY_API_KEY, POLY_PASSPHRASE, PRIVATE_KEY, create_authenticated,
    ensure_requirements,
};

mod unauthenticated {

    use chrono::{TimeDelta, TimeZone as _};
    use futures_util::future;
    use futures_util::stream::StreamExt as _;
    use polymarket_client_sdk::clob::types::{
        FeeRateResponseBuilder, LastTradePriceRequestBuilder, LastTradePriceResponseBuilder,
        LastTradesPricesResponseBuilder, MarketResponse, MarketResponseBuilder,
        MidpointRequestBuilder, MidpointResponseBuilder, MidpointsResponseBuilder,
        NegRiskResponseBuilder, OrderBookSummaryRequestBuilder, OrderBookSummaryResponseBuilder,
        OrderSummaryBuilder, PageBuilder, PriceRequestBuilder, PriceResponseBuilder,
        PricesResponseBuilder, RewardsBuilder, Side, SimplifiedMarketResponseBuilder,
        SpreadRequestBuilder, SpreadResponseBuilder, SpreadsResponseBuilder, TickSize,
        TickSizeResponseBuilder, TokenBuilder,
    };
    use polymarket_client_sdk::error::Status;
    use reqwest::Method;

    use super::*;

    #[tokio::test]
    async fn ok_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/");
            then.status(StatusCode::OK).body("\"OK\"");
        });

        let response = client.ok().await?;

        assert_eq!(response, "OK");
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn server_time_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/time");
            then.status(StatusCode::OK).body("1764612536");
        });

        let response = client.server_time().await?;

        assert_eq!(response, 1_764_612_536);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn midpoint_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/midpoint")
                .query_param("token_id", "1");
            then.status(StatusCode::OK)
                .json_body(json!({ "mid": "0.5" }));
        });

        let request = MidpointRequestBuilder::default().token_id("1").build()?;
        let response = client.midpoint(&request).await?;

        let expected = MidpointResponseBuilder::default().mid(dec!(0.5)).build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn midpoints_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/midpoints")
                .json_body(json!([{ "token_id": "1" }]));
            then.status(StatusCode::OK).json_body(json!(
                { "1": 0.5 }
            ));
        });

        let request = MidpointRequestBuilder::default().token_id("1").build()?;
        let response = client.midpoints(&[request]).await?;

        let expected = MidpointsResponseBuilder::default()
            .midpoints(HashMap::from_iter([("1".to_owned(), dec!(0.5))]))
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn price_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/price")
                .query_param("token_id", "1")
                .query_param("side", "BUY");
            then.status(StatusCode::OK)
                .json_body(json!({ "price": "0.5" }));
        });

        let request = PriceRequestBuilder::default()
            .token_id("1")
            .side(Side::Buy)
            .build()?;
        let response = client.price(&request).await?;

        let expected = PriceResponseBuilder::default().price(dec!(0.5)).build()?;

        assert_eq!(response, expected);
        mock.assert();

        let request = PriceRequestBuilder::default()
            .token_id("1")
            .side(Side::Sell)
            .build()?;
        let err = client.price(&request).await.unwrap_err();
        let status_err = err.downcast_ref::<Status>().unwrap();

        assert_eq!(
            status_err.to_string(),
            r#"error(404 Not Found) making GET call to /price with {"message":"Request did not match any route or mock"}"#
        );
        assert_eq!(status_err.status_code, StatusCode::NOT_FOUND);
        assert_eq!(status_err.method, Method::GET);
        assert_eq!(status_err.path, "/price");

        Ok(())
    }

    #[tokio::test]
    async fn prices_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/prices")
                .json_body(json!([{ "token_id": "1", "side": "BUY" }]));
            then.status(StatusCode::OK)
                .json_body(json!({ "1": { "BUY": 0.5 } }));
        });

        let mut price_map = HashMap::new();
        let mut side_map = HashMap::new();
        side_map.insert(Side::Buy, dec!(0.5));
        price_map.insert("1".to_owned(), side_map);

        let request = PriceRequestBuilder::default()
            .token_id("1")
            .side(Side::Buy)
            .build()?;
        let response = client.prices(&[request]).await?;

        let expected = PricesResponseBuilder::default().prices(price_map).build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn spread_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/spread")
                .query_param("token_id", "1");
            then.status(StatusCode::OK)
                .json_body(json!({ "spread": "0.5" }));
        });

        let request = SpreadRequestBuilder::default().token_id("1").build()?;
        let response = client.spread(&request).await?;

        let expected = SpreadResponseBuilder::default().spread(dec!(0.5)).build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn spreads_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/spreads")
                .json_body(json!([{ "token_id": "1" }]));
            then.status(StatusCode::OK)
                .json_body(json!({ "spreads": { "1": 2 } }));
        });

        let mut spread_map = HashMap::new();
        spread_map.insert("1".to_owned(), Decimal::TWO);

        let request = SpreadRequestBuilder::default().token_id("1").build()?;
        let response = client.spreads(&[request]).await?;

        let expected = SpreadsResponseBuilder::default()
            .spreads(spread_map)
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn tick_size_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/tick-size")
                .query_param("token_id", "1");
            then.status(StatusCode::OK)
                .json_body(json!({ "minimum_tick_size": "0.1" }));
        });

        let response = client.tick_size("1").await?;

        let expected = TickSizeResponseBuilder::default()
            .minimum_tick_size(TickSize::Tenth.as_decimal())
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn neg_risk_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/neg-risk")
                .query_param("token_id", "1");
            then.status(StatusCode::OK)
                .json_body(json!({ "neg_risk": true }));
        });

        let response = client.neg_risk("1").await?;

        let expected = NegRiskResponseBuilder::default().neg_risk(true).build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn fee_rate_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/fee-rate")
                .query_param("token_id", "1");
            then.status(StatusCode::OK)
                .json_body(json!({ "base_fee": 0 }));
        });

        let response = client.fee_rate_bps("1").await?;

        let expected = FeeRateResponseBuilder::default().base_fee(0).build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn order_book_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/book")
                .query_param("token_id", "1");
            then.status(StatusCode::OK).json_body(json!({
                "market": "0xaabbcc",
                "asset_id": "100",
                "tick_size": TickSize::Hundredth.as_decimal(),
                "min_order_size": "100",
                "neg_risk": false,
                "timestamp": "123456789",
                "bids": [
                    {
                        "price": "0.3",
                        "size": "100"
                    },
                    {
                        "price": "0.4",
                        "size": "100"
                    }
                ],
                "asks": [
                    {
                        "price": "0.6",
                        "size": "100"
                    },
                    {
                        "price": "0.7",
                        "size": "100"
                    }
                ]
            }));
        });

        let request = OrderBookSummaryRequestBuilder::default()
            .token_id("1")
            .build()?;
        let response = client.order_book(&request).await?;

        let expected = OrderBookSummaryResponseBuilder::default()
            .market("0xaabbcc")
            .neg_risk(false)
            .timestamp(Utc.timestamp_millis_opt(123_456_789).unwrap())
            .min_order_size(Decimal::ONE_HUNDRED)
            .tick_size(TickSize::Hundredth)
            .asset_id("100")
            .bids(vec![
                OrderSummaryBuilder::default()
                    .price(dec!(0.3))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?,
                OrderSummaryBuilder::default()
                    .price(dec!(0.4))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?,
            ])
            .asks(vec![
                OrderSummaryBuilder::default()
                    .price(dec!(0.6))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?,
                OrderSummaryBuilder::default()
                    .price(dec!(0.7))
                    .size(Decimal::ONE_HUNDRED)
                    .build()?,
            ])
            .build()?;

        assert_eq!(response, expected);
        assert_eq!(
            expected.hash()?,
            "98800af6a111791440d0936f5bc0fe1ac15da6eda09cc050227fb82ab8ceca7f"
        );
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn order_books_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/books")
                .json_body(json!([{ "token_id": "1" }]));
            then.status(StatusCode::OK).json_body(json!([{
                "market": "market",
                "asset_id": "asset",
                "tick_size": TickSize::Hundredth.as_decimal(),
                "min_order_size": "5",
                "neg_risk": false,
                "timestamp": "1",
                "asks": [{
                    "price": "2",
                    "size": "1"
                }]
            }]));
        });

        let request = OrderBookSummaryRequestBuilder::default()
            .token_id("1")
            .build()?;
        let response = client.order_books(&[request]).await?;

        let expected = vec![
            OrderBookSummaryResponseBuilder::default()
                .market("market")
                .neg_risk(false)
                .timestamp(DateTime::<Utc>::UNIX_EPOCH + TimeDelta::milliseconds(1))
                .min_order_size(dec!(5))
                .tick_size(TickSize::Hundredth)
                .asset_id("asset")
                .asks(vec![
                    OrderSummaryBuilder::default()
                        .price(Decimal::TWO)
                        .size(Decimal::ONE)
                        .build()?,
                ])
                .build()?,
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn last_trade_price_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/last-trade-price")
                .query_param("token_id", "1");
            then.status(StatusCode::OK)
                .json_body(json!({ "price": 0.12, "side": "BUY" }));
        });

        let request = LastTradePriceRequestBuilder::default()
            .token_id("1".to_owned())
            .build()?;
        let response = client.last_trade_price(&request).await?;

        let expected = LastTradePriceResponseBuilder::default()
            .price(dec!(0.12))
            .side(Side::Buy)
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn last_trades_prices_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/last-trades-prices")
                .json_body(json!([{ "token_id": "1" }]));
            then.status(StatusCode::OK)
                .json_body(json!([{ "token_id": "1", "price": 0.12, "side": "BUY" }]));
        });

        let request = LastTradePriceRequestBuilder::default()
            .token_id("1".to_owned())
            .build()?;
        let response = client.last_trades_prices(&[request]).await?;

        let expected = vec![
            LastTradesPricesResponseBuilder::default()
                .token_id("1".to_owned())
                .price(dec!(0.12))
                .side(Side::Buy)
                .build()?,
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn market_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/markets/1");
            then.status(StatusCode::OK).json_body(json!({
                "enable_order_book": true,
                "active": true,
                "closed": false,
                "archived": false,
                "accepting_orders": true,
                "accepting_order_timestamp": "2024-01-15T12:34:56Z",
                "minimum_order_size": "1",
                "minimum_tick_size": "0.01",
                "condition_id": "1",
                "question_id": "q_67890",
                "question": "Will BTC close above $50k today?",
                "description": "A market about BTC daily close price",
                "market_slug": "btc-close-above-50k",
                "end_date_iso": "2024-02-01T00:00:00Z",
                "game_start_time": null,
                "seconds_delay": 5,
                "fpmm": "fpmm_abc123",
                "maker_base_fee": "0",
                "taker_base_fee": 0.1,
                "notifications_enabled": true,
                "neg_risk": false,
                "neg_risk_market_id": "",
                "neg_risk_request_id": "",
                "icon": "https://example.com/icon.png",
                "image": "https://example.com/image.png",
                "rewards": {
                    "rates": null,
                    "min_size": "10.0",
                    "max_spread": "0.05"
                },
                "is_50_50_outcome": false,
                "tokens": [
                    {
                        "token_id": "YES_TOKEN",
                        "outcome": "YES",
                        "price": "0.55",
                        "winner": false
                    },
                    {
                        "token_id": "NO_TOKEN",
                        "outcome": "NO",
                        "price": "0.45",
                        "winner": false
                    }
                ],
                "tags": [
                    "crypto",
                    "btc",
                    "price"
                ]
            }));
        });

        let response = client.market("1").await?;

        let expected = MarketResponseBuilder::default()
            .enable_order_book(true)
            .active(true)
            .closed(false)
            .archived(false)
            .accepting_orders(true)
            .accepting_order_timestamp("2024-01-15T12:34:56Z".parse::<DateTime<Utc>>().unwrap())
            .minimum_order_size(Decimal::ONE)
            .minimum_tick_size(TickSize::Hundredth.as_decimal())
            .condition_id("1")
            .question_id("q_67890")
            .question("Will BTC close above $50k today?")
            .description("A market about BTC daily close price")
            .market_slug("btc-close-above-50k")
            .end_date_iso("2024-02-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap())
            .seconds_delay(5)
            .fpmm("fpmm_abc123")
            .maker_base_fee(Decimal::ZERO)
            .taker_base_fee(dec!(0.1))
            .notifications_enabled(true)
            .neg_risk(false)
            .neg_risk_market_id("")
            .neg_risk_request_id("")
            .icon("https://example.com/icon.png")
            .image("https://example.com/image.png")
            .rewards(
                RewardsBuilder::default()
                    .min_size(dec!(10.0))
                    .max_spread(dec!(0.05))
                    .build()?,
            )
            .is_50_50_outcome(false)
            .tokens(vec![
                TokenBuilder::default()
                    .token_id("YES_TOKEN")
                    .outcome("YES")
                    .price(dec!(0.55))
                    .winner(false)
                    .build()?,
                TokenBuilder::default()
                    .token_id("NO_TOKEN")
                    .outcome("NO")
                    .price(dec!(0.45))
                    .winner(false)
                    .build()?,
            ])
            .tags(vec!["crypto".into(), "btc".into(), "price".into()])
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn sampling_markets_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/sampling-markets");
            then.status(StatusCode::OK).json_body(json!({
                "data": [
                    {
                        "enable_order_book": true,
                        "active": true,
                        "closed": false,
                        "archived": false,
                        "accepting_orders": true,
                        "accepting_order_timestamp": "2024-01-15T12:34:56Z",
                        "minimum_order_size": "1",
                        "minimum_tick_size": "0.01",
                        "condition_id": "1",
                        "question_id": "q_67890",
                        "question": "Will BTC close above $50k today?",
                        "description": "A market about BTC daily close price",
                        "market_slug": "btc-close-above-50k",
                        "end_date_iso": "2024-02-01T00:00:00Z",
                        "game_start_time": null,
                        "seconds_delay": 5,
                        "fpmm": "fpmm_abc123",
                        "maker_base_fee": "0",
                        "taker_base_fee": "0",
                        "notifications_enabled": true,
                        "neg_risk": false,
                        "neg_risk_market_id": "",
                        "neg_risk_request_id": "",
                        "icon": "https://example.com/icon.png",
                        "image": "https://example.com/image.png",
                        "rewards": {
                            "rates": null,
                            "min_size": "10.0",
                            "max_spread": "0.05"
                        },
                        "is_50_50_outcome": false,
                        "tokens": [
                            {
                                "token_id": "YES_TOKEN",
                                "outcome": "YES",
                                "price": "0.55",
                                "winner": false
                            },
                            {
                                "token_id": "NO_TOKEN",
                                "outcome": "NO",
                                "price": "0.45",
                                "winner": false
                            }
                        ],
                        "tags": [
                            "crypto",
                            "btc",
                            "price"
                        ]
                    }
                ],
                "limit": 1,
                "count": 1,
                "next_cursor": "next"
            }));
        });

        let response = client.sampling_markets(None).await?;

        let market = MarketResponseBuilder::default()
            .enable_order_book(true)
            .active(true)
            .closed(false)
            .archived(false)
            .accepting_orders(true)
            .accepting_order_timestamp("2024-01-15T12:34:56Z".parse::<DateTime<Utc>>().unwrap())
            .minimum_order_size(Decimal::ONE)
            .minimum_tick_size(TickSize::Hundredth.as_decimal())
            .condition_id("1")
            .question_id("q_67890")
            .question("Will BTC close above $50k today?")
            .description("A market about BTC daily close price")
            .market_slug("btc-close-above-50k")
            .end_date_iso("2024-02-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap())
            .seconds_delay(5)
            .fpmm("fpmm_abc123")
            .maker_base_fee(Decimal::ZERO)
            .taker_base_fee(Decimal::ZERO)
            .notifications_enabled(true)
            .neg_risk(false)
            .neg_risk_market_id("")
            .neg_risk_request_id("")
            .icon("https://example.com/icon.png")
            .image("https://example.com/image.png")
            .rewards(
                RewardsBuilder::default()
                    .min_size(dec!(10.0))
                    .max_spread(dec!(0.05))
                    .build()?,
            )
            .is_50_50_outcome(false)
            .tokens(vec![
                TokenBuilder::default()
                    .token_id("YES_TOKEN")
                    .outcome("YES")
                    .price(dec!(0.55))
                    .winner(false)
                    .build()?,
                TokenBuilder::default()
                    .token_id("NO_TOKEN")
                    .outcome("NO")
                    .price(dec!(0.45))
                    .winner(false)
                    .build()?,
            ])
            .tags(vec!["crypto".into(), "btc".into(), "price".into()])
            .build()?;
        let expected = PageBuilder::default()
            .data(vec![market])
            .next_cursor("next")
            .limit(1)
            .count(1)
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn simplified_markets_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/simplified-markets");
            then.status(StatusCode::OK).json_body(json!({
                "data": [
                    {
                        "condition_id": "cond_12345",
                        "tokens": [
                            {
                                "token_id": "YES_TOKEN",
                                "outcome": "YES",
                                "price": "0.55",
                                "winner": false
                            },
                            {
                                "token_id": "NO_TOKEN",
                                "outcome": "NO",
                                "price": "0.45",
                                "winner": false
                            }
                        ],
                        "rewards": {
                            "rates": null,
                            "min_size": "10.0",
                            "max_spread": "0.05"
                        },
                        "archived": false,
                        "accepting_orders": true,
                        "active": true,
                        "closed": false
                    }
                ],
                "limit": 1,
                "count": 1,
                "next_cursor": "next"
            }));
        });

        let response = client.simplified_markets(None).await?;

        let simplified = SimplifiedMarketResponseBuilder::default()
            .condition_id("cond_12345")
            .tokens(vec![
                TokenBuilder::default()
                    .token_id("YES_TOKEN")
                    .outcome("YES")
                    .price(dec!(0.55))
                    .winner(false)
                    .build()
                    .unwrap(),
                TokenBuilder::default()
                    .token_id("NO_TOKEN")
                    .outcome("NO")
                    .price(dec!(0.45))
                    .winner(false)
                    .build()
                    .unwrap(),
            ])
            .rewards(
                RewardsBuilder::default()
                    .min_size(dec!(10.0))
                    .max_spread(dec!(0.05))
                    .build()
                    .unwrap(),
            )
            .archived(false)
            .accepting_orders(true)
            .active(true)
            .closed(false)
            .build()?;
        let expected = PageBuilder::default()
            .data(vec![simplified])
            .next_cursor("next")
            .limit(1)
            .count(1)
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn sampling_simplified_markets_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/sampling-simplified-markets");
            then.status(StatusCode::OK).json_body(json!({
                "data": [
                    {
                        "condition_id": "cond_12345",
                        "tokens": [
                            {
                                "token_id": "YES_TOKEN",
                                "outcome": "YES",
                                "price": "0.55",
                                "winner": false
                            },
                            {
                                "token_id": "NO_TOKEN",
                                "outcome": "NO",
                                "price": "0.45",
                                "winner": false
                            }
                        ],
                        "rewards": {
                            "rates": null,
                            "min_size": "10.0",
                            "max_spread": "0.05"
                        },
                        "archived": false,
                        "accepting_orders": true,
                        "active": true,
                        "closed": false
                    }
                ],
                "limit": 1,
                "count": 1,
                "next_cursor": "next"
            }));
        });

        let response = client.sampling_simplified_markets(None).await?;

        let simplified = SimplifiedMarketResponseBuilder::default()
            .condition_id("cond_12345")
            .tokens(vec![
                TokenBuilder::default()
                    .token_id("YES_TOKEN")
                    .outcome("YES")
                    .price(dec!(0.55))
                    .winner(false)
                    .build()
                    .unwrap(),
                TokenBuilder::default()
                    .token_id("NO_TOKEN")
                    .outcome("NO")
                    .price(dec!(0.45))
                    .winner(false)
                    .build()
                    .unwrap(),
            ])
            .rewards(
                RewardsBuilder::default()
                    .min_size(dec!(10.0))
                    .max_spread(dec!(0.05))
                    .build()
                    .unwrap(),
            )
            .archived(false)
            .accepting_orders(true)
            .active(true)
            .closed(false)
            .build()?;
        let expected = PageBuilder::default()
            .data(vec![simplified])
            .next_cursor("next")
            .limit(1)
            .count(1)
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn stream_markets_should_succeed() -> anyhow::Result<()> {
        const TERMINAL_CURSOR: &str = "LTE="; // base64("-1")

        let server = MockServer::start();
        let client = Client::new(&server.base_url(), Config::default())?;

        let json = json!({
            "data": [
                {
                    "enable_order_book": true,
                    "active": true,
                    "closed": false,
                    "archived": false,
                    "accepting_orders": true,
                    "accepting_order_timestamp": "2024-01-15T12:34:56Z",
                    "minimum_order_size": "1",
                    "minimum_tick_size": "0.01",
                    "condition_id": "1",
                    "question_id": "q_67890",
                    "question": "Will BTC close above $50k today?",
                    "description": "A market about BTC daily close price",
                    "market_slug": "btc-close-above-50k",
                    "end_date_iso": "2024-02-01T00:00:00Z",
                    "game_start_time": null,
                    "seconds_delay": 5,
                    "fpmm": "fpmm_abc123",
                    "maker_base_fee": "0",
                    "taker_base_fee": "0",
                    "notifications_enabled": true,
                    "neg_risk": false,
                    "neg_risk_market_id": "",
                    "neg_risk_request_id": "",
                    "icon": "https://example.com/icon.png",
                    "image": "https://example.com/image.png",
                    "rewards": {
                        "rates": null,
                        "min_size": "10.0",
                        "max_spread": "0.05"
                    },
                    "is_50_50_outcome": false,
                    "tokens": [
                        {
                            "token_id": "YES_TOKEN",
                            "outcome": "YES",
                            "price": "0.55",
                            "winner": false
                        },
                        {
                            "token_id": "NO_TOKEN",
                            "outcome": "NO",
                            "price": "0.45",
                            "winner": false
                        }
                    ],
                    "tags": [
                        "crypto",
                        "btc",
                        "price"
                    ]
                }
            ],
            "limit": 1,
            "count": 1
        });

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/markets")
                .is_true(|req| req.query_params().is_empty());

            let mut json_with_cursor = json.clone();
            if let Some(obj) = json_with_cursor.as_object_mut() {
                obj.insert(
                    "next_cursor".to_owned(),
                    serde_json::Value::String("1".to_owned()),
                );
            }

            then.status(StatusCode::OK).json_body(json_with_cursor);
        });

        let mock2 = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/markets")
                .query_param("next_cursor", "1");

            let mut json_with_cursor = json.clone();
            if let Some(obj) = json_with_cursor.as_object_mut() {
                obj.insert(
                    "next_cursor".to_owned(),
                    serde_json::Value::String(TERMINAL_CURSOR.to_owned()),
                );
            }

            then.status(StatusCode::OK).json_body(json_with_cursor);
        });

        let response: Vec<MarketResponse> = client
            .stream_data(Client::markets)
            .filter_map(|d| future::ready(d.ok()))
            .collect()
            .await;

        let market = MarketResponseBuilder::default()
            .enable_order_book(true)
            .active(true)
            .closed(false)
            .archived(false)
            .accepting_orders(true)
            .accepting_order_timestamp("2024-01-15T12:34:56Z".parse::<DateTime<Utc>>().unwrap())
            .minimum_order_size(Decimal::ONE)
            .minimum_tick_size(TickSize::Hundredth.as_decimal())
            .condition_id("1")
            .question_id("q_67890")
            .question("Will BTC close above $50k today?")
            .description("A market about BTC daily close price")
            .market_slug("btc-close-above-50k")
            .end_date_iso("2024-02-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap())
            .seconds_delay(5)
            .fpmm("fpmm_abc123")
            .maker_base_fee(Decimal::ZERO)
            .taker_base_fee(Decimal::ZERO)
            .notifications_enabled(true)
            .neg_risk(false)
            .neg_risk_market_id("")
            .neg_risk_request_id("")
            .icon("https://example.com/icon.png")
            .image("https://example.com/image.png")
            .rewards(
                RewardsBuilder::default()
                    .min_size(dec!(10.0))
                    .max_spread(dec!(0.05))
                    .build()?,
            )
            .is_50_50_outcome(false)
            .tokens(vec![
                TokenBuilder::default()
                    .token_id("YES_TOKEN")
                    .outcome("YES")
                    .price(dec!(0.55))
                    .winner(false)
                    .build()?,
                TokenBuilder::default()
                    .token_id("NO_TOKEN")
                    .outcome("NO")
                    .price(dec!(0.45))
                    .winner(false)
                    .build()?,
            ])
            .tags(vec!["crypto".into(), "btc".into(), "price".into()])
            .build()?;
        let expected = vec![market.clone(), market];

        assert_eq!(response, expected);
        mock.assert();
        mock2.assert();

        Ok(())
    }
}

mod authenticated {
    use alloy::primitives::{Address, Signature, address};
    use alloy::signers::Signer as _;
    use alloy::signers::local::LocalSigner;
    use chrono::NaiveDate;
    use httpmock::Method::{DELETE, GET, POST};
    use polymarket_client_sdk::clob::ConfigBuilder;
    use polymarket_client_sdk::clob::types::{
        ApiKeysResponseBuilder, AssetType, BalanceAllowanceRequestBuilder,
        BalanceAllowanceResponseBuilder, BanStatusResponseBuilder, CancelMarketOrderRequestBuilder,
        CancelOrdersResponseBuilder, CurrentRewardResponseBuilder,
        DeleteNotificationsRequestBuilder, EarningBuilder, MakerOrderBuilder,
        MarketRewardResponseBuilder, MarketRewardsConfigBuilder, NotificationPayloadBuilder,
        NotificationResponseBuilder, OpenOrderResponseBuilder, OrderScoringResponseBuilder,
        OrderStatusType, OrderType, OrdersRequestBuilder, PageBuilder, PostOrderResponseBuilder,
        RewardsConfigBuilder, Side, SignableOrder, SignedOrderBuilder, TokenBuilder,
        TotalUserEarningResponseBuilder, TradeResponseBuilder, TraderSide, TradesRequestBuilder,
        UserEarningResponseBuilder, UserRewardsEarningRequestBuilder,
        UserRewardsEarningResponseBuilder,
    };

    use super::*;
    use crate::common::{
        API_KEY, PASSPHRASE, POLY_NONCE, POLY_SIGNATURE, POLY_TIMESTAMP, SECRET, SIGNATURE,
        TIMESTAMP,
    };

    #[tokio::test]
    async fn api_keys_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/auth/api-keys")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE);
            then.status(StatusCode::OK)
                .json_body(json!({"apiKeys": [API_KEY]}));
        });

        let response = client.api_keys().await?;

        let expected = ApiKeysResponseBuilder::default()
            .keys(vec![API_KEY])
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn delete_api_keys_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/auth/api-key")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE);
            then.status(StatusCode::OK).body("\"\"");
        });

        client.delete_api_key().await?;

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn closed_only_mode_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/auth/ban-status/closed-only")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE);
            then.status(StatusCode::OK)
                .json_body(json!({"closed_only": true}));
        });

        let response = client.closed_only_mode().await?;

        let expected = BanStatusResponseBuilder::default()
            .closed_only(true)
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    // Also fills in some other, less often used fields like nonce, and salt generator
    #[tokio::test]
    async fn sign_order_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/auth/derive-api-key")
                .header(POLY_ADDRESS, signer.address().to_string().to_lowercase())
                .header(POLY_NONCE, "0")
                .header(POLY_SIGNATURE, SIGNATURE)
                .header(POLY_TIMESTAMP, TIMESTAMP);
            then.status(StatusCode::OK).json_body(json!({
                "apiKey": API_KEY.to_string(),
                "passphrase": PASSPHRASE,
                "secret": SECRET
            }));
        });
        let mock2 = server.mock(|when, then| {
            when.method(GET).path("/time");
            then.status(StatusCode::OK)
                .json_body(TIMESTAMP.parse::<i64>().unwrap());
        });

        let funder = address!("0x995c9b1f779c04e65AF8ea3360F96c43b5e62316");
        let config = ConfigBuilder::default().use_server_time(true).build()?;
        let client = Client::new(&server.base_url(), config)?
            .authentication_builder(&signer)
            .funder(funder)
            .signature_type(SignatureType::Proxy)
            .salt_generator(|| 1) // To ensure determinism
            .authenticate()
            .await?;

        ensure_requirements(&server, "123", TickSize::Thousandth);

        assert_eq!(
            client.tick_size("123").await?.minimum_tick_size,
            TickSize::Thousandth
        );

        let taker = address!("0xf7fB45986800e2D259BAa25B56466bd02dA37a44");
        let signable_order = client
            .limit_order()
            .token_id("123")
            .price(dec!(0.512))
            .size(Decimal::ONE_HUNDRED)
            .side(Side::Buy)
            .taker(taker)
            .nonce(2)
            .build()
            .await?;

        let signed_order = client.sign(&signer, signable_order.clone()).await?;

        let expected = SignedOrderBuilder::default()
            .owner(API_KEY)
            .order(signable_order.order)
            .order_type(OrderType::GTC)
            .signature(Signature::new(
                U256::from_str(
                    "83481775348298569941382281649378839838022156673358335472613408323829135581822",
                )?,
                U256::from_str(
                    "14700784935255652319392527293337942251542995764256618215552578034044377798980",
                )?,
                false,
            ))
            .build()?;

        assert_eq!(signed_order.order.taker, taker);
        assert_eq!(signed_order.order.maker, funder);
        assert_ne!(signed_order.order.maker, client.address());
        assert_eq!(signed_order.order.signatureType, SignatureType::Proxy as u8);
        assert_eq!(signed_order.order.nonce, U256::from(2));
        assert_eq!(signed_order.order.salt, U256::from(1));
        assert_eq!(
            client.address(),
            address!("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266")
        );

        assert_eq!(signed_order, expected);
        mock.assert();
        mock2.assert_calls(2);

        Ok(())
    }

    #[tokio::test]
    async fn post_order_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        ensure_requirements(&server, "1", TickSize::Hundredth);

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/orders")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .json_body(json!([
                    {
                        "order": {
                            "expiration": "0",
                            "feeRateBps": "0",
                            "maker": Address::ZERO,
                            "makerAmount": "0",
                            "nonce": "0",
                            "salt": 0,
                            "side": Side::Buy,
                            "signature": "0x0d18c04a653d89bf7375636adb7db69cffe362755960dc6ce8a0d46b04355b767958fae51c48e0e4b0908347442cb461e811d2f5a751303f7a8c1f75e17b3e701b",
                            "signatureType": 0,
                            "signer": Address::ZERO,
                            "taker": Address::ZERO,
                            "takerAmount": "0",
                            "tokenId": "0"
                        },
                        "orderType": "FOK",
                        "owner": "00000000-0000-0000-0000-000000000000"
                    }
                ]));
            then.status(StatusCode::OK).json_body(json!([
                {
                    "error_msg": "",
                    "makingAmount": "",
                    "orderID": "0x23b457271bce9fa09b4f79125c9ec09e968235a462de82e318ef4eb6fe0ffeb0",
                    "status": "live",
                    "success": true,
                    "takingAmount": ""
                }
            ]));
        });

        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
        let signed_order = client.sign(&signer, SignableOrder::default()).await?;
        let response = client.post_order(signed_order).await?;

        let expected = vec![
            PostOrderResponseBuilder::default()
                .making_amount(Decimal::ZERO)
                .taking_amount(Decimal::ZERO)
                .order_id("0x23b457271bce9fa09b4f79125c9ec09e968235a462de82e318ef4eb6fe0ffeb0")
                .status(OrderStatusType::Live)
                .success(true)
                .build()?,
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn order_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let json = json!({
            "id": "1",
            "status": "LIVE",
            "owner": "ffffffff-ffff-ffff-ffff-ffffffffffff",
            "maker_address": "0x2222222222222222222222222222222222222222",
            "market": "market_abc",
            "asset_id": "asset_xyz",
            "side": "buy",
            "original_size": "10.0",
            "size_matched": "2.5",
            "price": "0.45",
            "associate_trades": [
                "0xtradehash1",
                "0xtradehash2"
            ],
            "outcome": "YES",
            "created_at": 1_705_322_096,
            "expiration": "1705708800",
            "order_type": "gtd"
        });

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/data/order/1")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE);
            then.status(StatusCode::OK).json_body(json);
        });

        let response = client.order("1").await?;

        let expected = OpenOrderResponseBuilder::default()
            .id("1")
            .status(OrderStatusType::Live)
            .owner(Uuid::max())
            .maker_address(address!("0x2222222222222222222222222222222222222222"))
            .market("market_abc")
            .asset_id("asset_xyz")
            .side(Side::Buy)
            .original_size(dec!(10.0))
            .size_matched(dec!(2.5))
            .price(dec!(0.45))
            .associate_trades(vec!["0xtradehash1".into(), "0xtradehash2".into()])
            .outcome("YES")
            .created_at("2024-01-15T12:34:56Z".parse().unwrap())
            .expiration("2024-01-20T00:00:00Z".parse().unwrap())
            .order_type(OrderType::GTD)
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn orders_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let json = json!({
            "data": [
                {
                    "id": "1",
                    "status": "LIVE",
                    "owner": "ffffffff-ffff-ffff-ffff-ffffffffffff",
                    "maker_address": "0x2222222222222222222222222222222222222222",
                    "market": "market_abc",
                    "asset_id": "asset_xyz",
                    "side": "buy",
                    "original_size": "10.0",
                    "size_matched": "2.5",
                    "price": "0.45",
                    "associate_trades": [
                        "0xtradehash1",
                        "0xtradehash2"
                    ],
                    "outcome": "YES",
                    "created_at": 1_705_322_096,
                    "expiration": "1705708800",
                    "order_type": "GTC"
                }
            ],
            "limit": 1,
            "count": 1,
            "next_cursor": "next"
        });

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/data/orders")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("order_id", "1");
            then.status(StatusCode::OK).json_body(json);
        });

        let request = OrdersRequestBuilder::default().order_id("1").build()?;
        let response = client.orders(&request, None).await?;

        let order = OpenOrderResponseBuilder::default()
            .id("1")
            .status(OrderStatusType::Live)
            .owner(Uuid::max())
            .maker_address(address!("0x2222222222222222222222222222222222222222"))
            .market("market_abc")
            .asset_id("asset_xyz")
            .side(Side::Buy)
            .original_size(dec!(10.0))
            .size_matched(dec!(2.5))
            .price(dec!(0.45))
            .associate_trades(vec!["0xtradehash1".into(), "0xtradehash2".into()])
            .outcome("YES")
            .created_at("2024-01-15T12:34:56Z".parse().unwrap())
            .expiration("2024-01-20T00:00:00Z".parse().unwrap())
            .order_type(OrderType::GTC)
            .build()?;
        let expected = PageBuilder::default()
            .data(vec![order])
            .limit(1)
            .count(1)
            .next_cursor("next")
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn cancel_order_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/order")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .json_body(json!({ "orderId": "1" }));
            then.status(StatusCode::OK).json_body(json!({
                    "canceled": [],
                    "notCanceled": {
                        "1": "the order is already canceled"
                    }
                }
            ));
        });

        let response = client.cancel_order("1").await?;

        let expected = CancelOrdersResponseBuilder::default()
            .not_canceled(HashMap::from_iter([(
                "1".to_owned(),
                "the order is already canceled".to_owned(),
            )]))
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn cancel_orders_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/orders")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .json_body(json!(["1"]));
            then.status(StatusCode::OK).json_body(json!({
                    "canceled": ["1"]
                }
            ));
        });

        let response = client.cancel_orders(&["1"]).await?;

        let expected = CancelOrdersResponseBuilder::default()
            .canceled(vec!["1".to_owned()])
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn cancel_all_orders_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(DELETE)
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .path("/cancel-all");
            then.status(StatusCode::OK).json_body(json!({
                    "canceled": ["2"],
                    "notCanceled": {
                        "1": "the order is already canceled"
                    }
                }
            ));
        });

        let response = client.cancel_all_orders().await?;

        let expected = CancelOrdersResponseBuilder::default()
            .canceled(vec!["2".to_owned()])
            .not_canceled(HashMap::from_iter([(
                "1".to_owned(),
                "the order is already canceled".to_owned(),
            )]))
            .build()?;

        assert_eq!(response, expected);

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn cancel_market_orders_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(DELETE)
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .path("/cancel-market-orders");
            then.status(StatusCode::OK).json_body(json!({
                "market": "m",
                "asset_id": "a",
            }));
        });

        let request = CancelMarketOrderRequestBuilder::default()
            .market("m")
            .asset_id("a")
            .build()?;

        client.cancel_market_orders(&request).await?;

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn trades_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/data/trades")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("id", "1")
                .query_param("market", "market");

            then.status(StatusCode::OK).json_body(json!({
                "data": [
                    {
                        "id": "1",
                        "taker_order_id": "taker_123",
                        "market": "market",
                        "asset_id": "asset_xyz",
                        "side": "BUY",
                        "size": "12.5",
                        "fee_rate_bps": "5",
                        "price": "0.42",
                        "status": "MATCHED",
                        "match_time": "1705322096",
                        "last_update": "1705322130",
                        "outcome": "YES",
                        "bucket_index": 2,
                        "owner": "ffffffff-ffff-ffff-ffff-ffffffffffff",
                        "maker_address": "0x2222222222222222222222222222222222222222",
                        "maker_orders": [
                            {
                                "order_id": "maker_001",
                                "owner": "ffffffff-ffff-ffff-ffff-ffffffffffff",
                                "maker_address": "0x4444444444444444444444444444444444444444",
                                "matched_amount": "5.0",
                                "price": "0.42",
                                "fee_rate_bps": "5",
                                "asset_id": "asset_xyz",
                                "outcome": "YES",
                                "side": "SELL"
                            },
                            {
                                "order_id": "maker_002",
                                "owner": "ffffffff-ffff-ffff-ffff-ffffffffffff",
                                "maker_address": "0x6666666666666666666666666666666666666666",
                                "matched_amount": "7.5",
                                "price": "0.42",
                                "fee_rate_bps": "5",
                                "asset_id": "asset_xyz",
                                "outcome": "YES",
                                "side": "SELL"
                            }
                        ],
                        "transaction_hash": "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd",
                        "trader_side": "TAKER"
                    }
                ],
                "limit": 1,
                "count": 1,
                "next_cursor": "next"
            }));
        });

        let request = TradesRequestBuilder::default()
            .id("1")
            .market("market")
            .build()?;
        let response = client.trades(&request, None).await?;

        let trade = TradeResponseBuilder::default()
            .id("1")
            .taker_order_id("taker_123")
            .market("market")
            .asset_id("asset_xyz")
            .side(Side::Buy)
            .size(dec!(12.5))
            .fee_rate_bps(dec!(5))
            .price(dec!(0.42))
            .status(OrderStatusType::Matched)
            .match_time("2024-01-15T12:34:56Z".parse().unwrap())
            .last_update("2024-01-15T12:35:30Z".parse().unwrap())
            .outcome("YES")
            .bucket_index(2)
            .owner(Uuid::max())
            .maker_address(address!("0x2222222222222222222222222222222222222222"))
            .maker_orders(vec![
                MakerOrderBuilder::default()
                    .order_id("maker_001")
                    .owner(Uuid::max())
                    .maker_address(address!("0x4444444444444444444444444444444444444444"))
                    .matched_amount(dec!(5.0))
                    .price(dec!(0.42))
                    .fee_rate_bps(dec!(5))
                    .asset_id("asset_xyz")
                    .outcome("YES")
                    .side(Side::Sell)
                    .build()?,
                MakerOrderBuilder::default()
                    .order_id("maker_002")
                    .owner(Uuid::max())
                    .maker_address(address!("0x6666666666666666666666666666666666666666"))
                    .matched_amount(dec!(7.5))
                    .price(dec!(0.42))
                    .fee_rate_bps(dec!(5))
                    .asset_id("asset_xyz")
                    .outcome("YES")
                    .side(Side::Sell)
                    .build()?,
            ])
            .transaction_hash("0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .trader_side(TraderSide::Taker)
            .build()?;
        let expected = PageBuilder::default()
            .limit(1)
            .count(1)
            .data(vec![trade])
            .next_cursor("next")
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn notifications_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/notifications")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("signature_type", (SignatureType::Eoa as u8).to_string());
            then.status(StatusCode::OK).json_body(json!([
                {
                    "type": 1,
                    "owner": API_KEY,
                    "payload": {
                        "asset_id": "71321045679252212594626385532706912750332728571942532289631379312455583992563",
                        "condition_id": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
                        "eventSlug": "will-trump-win-the-2024-iowa-caucus",
                        "icon": "https://polymarket-upload.s3.us-east-2.amazonaws.com/trump1+copy.png",
                        "image": "https://polymarket-upload.s3.us-east-2.amazonaws.com/trump1+copy.png",
                        "market": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
                        "market_slug": "will-trump-win-the-2024-iowa-caucus",
                        "matched_size": "20",
                        "order_id": "0x2ae21876d2702d8b71308d0999062db9625a691ce4593c5f10230eeeff945e70",
                        "original_size": "2.4",
                        "outcome": "YES",
                        "outcome_index": 0,
                        "owner": "b349bff6-7af8-0470-ed25-22a2a5e1c154",
                        "price": "0.12",
                        "question": "Will Trump win the 2024 Iowa Caucus?",
                        "remaining_size": "0",
                        "seriesSlug": "",
                        "side": "buy",
                        "trade_id": "565a5035-d70e-4493-9215-8cae52d26efe",
                        "transaction_hash": "0x3bc57dcae83a930df64fce8fdc46a8fca9b98af92a7b83a8a2f2c657446c2a71",
                        "type": ""
                    }
                }
            ]));
        });

        let response = client.notifications().await?;

        let expected = vec![
            NotificationResponseBuilder::default()
                .r#type(1)
                .owner(API_KEY)
                .payload(NotificationPayloadBuilder::default()
                    .asset_id("71321045679252212594626385532706912750332728571942532289631379312455583992563")
                    .condition_id("0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1")
                    .event_slug("will-trump-win-the-2024-iowa-caucus")
                    .icon("https://polymarket-upload.s3.us-east-2.amazonaws.com/trump1+copy.png")
                    .image("https://polymarket-upload.s3.us-east-2.amazonaws.com/trump1+copy.png")
                    .market("0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1")
                    .market_slug("will-trump-win-the-2024-iowa-caucus")
                    .matched_size(dec!(20))
                    .order_id("0x2ae21876d2702d8b71308d0999062db9625a691ce4593c5f10230eeeff945e70")
                    .original_size(dec!(2.4))
                    .outcome("YES")
                    .outcome_index(0)
                    .owner(Uuid::from_str("b349bff6-7af8-0470-ed25-22a2a5e1c154").unwrap())
                    .price(dec!(0.12))
                    .question("Will Trump win the 2024 Iowa Caucus?")
                    .remaining_size(Decimal::ZERO)
                    .series_slug("")
                    .side(Side::Buy)
                    .trade_id("565a5035-d70e-4493-9215-8cae52d26efe")
                    .transaction_hash(
                        "0x3bc57dcae83a930df64fce8fdc46a8fca9b98af92a7b83a8a2f2c657446c2a71",
                    )
                    .order_type(OrderType::Unknown)
                    .build()?
                )
                .build()?,
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn delete_notifications_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/notifications")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("ids", "1,2");
            then.status(StatusCode::OK).json_body(json!(null));
        });

        let request = DeleteNotificationsRequestBuilder::default()
            .notification_ids(vec!["1".to_owned(), "2".to_owned()])
            .build()?;
        client.delete_notifications(&request).await?;

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn balance_allowance_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/balance-allowance")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("asset_type", "COLLATERAL")
                .query_param("token_id", "1")
                .query_param("signature_type", "0");
            // Trying different Decimal deserialization routes
            then.status(StatusCode::OK).json_body(json!({
                "balance": 0,
                "allowances": { Address::ZERO.to_string(): "1" }
            }));
        });

        let request = BalanceAllowanceRequestBuilder::default()
            .asset_type(AssetType::Collateral)
            .token_id("1")
            .build()?;
        let response = client.balance_allowance(&request).await?;

        let expected = BalanceAllowanceResponseBuilder::default()
            .balance(Decimal::ZERO)
            .allowances(HashMap::from_iter([(Address::ZERO, "1".to_owned())]))
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn update_balance_allowance_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/balance-allowance/update")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("asset_type", "COLLATERAL")
                .query_param("token_id", "1")
                .query_param("signature_type", "0");
            then.status(StatusCode::OK).json_body(json!(null));
        });

        let request = BalanceAllowanceRequestBuilder::default()
            .asset_type(AssetType::Collateral)
            .token_id("1")
            .build()?;
        client.update_balance_allowance(&request).await?;

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn is_order_scoring_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/order-scoring")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("order_id", "1");
            then.status(StatusCode::OK).json_body(json!({
                "scoring": true,
            }));
        });

        let response = client.is_order_scoring("1").await?;

        let expected = OrderScoringResponseBuilder::default()
            .scoring(true)
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn are_orders_scoring_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/orders-scoring")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .json_body(json!(["1"]));
            then.status(StatusCode::OK).json_body(json!(
                { "1": true }
            ));
        });

        let response = client.are_orders_scoring(&["1"]).await?;

        let expected = HashMap::from_iter(vec![("1".to_owned(), true)]);

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn earnings_for_user_for_day_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let date = NaiveDate::from_ymd_opt(2025, 12, 8).unwrap();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/rewards/user")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("date", date.to_string())
                .query_param("signature_type", (SignatureType::Eoa as u8).to_string());
            then.status(StatusCode::OK).json_body(json!({
                "data": [{
                    "date": "2025-12-08",
                    "condition_id": "1",
                    "asset_address": "0x0000000000000000000000000000000000000001",
                    "maker_address": "0x0000000000000000000000000000000000000002",
                    "earnings": 1,
                    "asset_rate": "0.1"
                }],
                "limit": 1,
                "count": 1,
                "next_cursor": "next"
            }));
        });

        let expected = PageBuilder::default()
            .limit(1)
            .count(1)
            .next_cursor("next")
            .data(vec![
                UserEarningResponseBuilder::default()
                    .date(date)
                    .condition_id("1")
                    .asset_address(address!("0x0000000000000000000000000000000000000001"))
                    .maker_address(address!("0x0000000000000000000000000000000000000002"))
                    .earnings(Decimal::ONE)
                    .asset_rate(dec!(0.1))
                    .build()?,
            ])
            .build()?;

        let response = client.earnings_for_user_for_day(date, None).await?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn total_earnings_for_user_for_day_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let date = NaiveDate::from_ymd_opt(2025, 12, 8).unwrap();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/rewards/user/total")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("date", date.to_string())
                .query_param("signature_type", (SignatureType::Eoa as u8).to_string());
            then.status(StatusCode::OK).json_body(json!([{
                "date": "2025-12-08",
                "asset_address": "0x0000000000000000000000000000000000000001",
                "maker_address": "0x0000000000000000000000000000000000000002",
                "earnings": 1,
                "asset_rate": "0.1"
            }]));
        });

        let response = client.total_earnings_for_user_for_day(date).await?;

        let expected = vec![
            TotalUserEarningResponseBuilder::default()
                .date(date)
                .asset_address(address!("0x0000000000000000000000000000000000000001"))
                .maker_address(address!("0x0000000000000000000000000000000000000002"))
                .earnings(Decimal::ONE)
                .asset_rate(dec!(0.1))
                .build()?,
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn user_earnings_and_markets_config_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let today = Utc::now();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/rewards/user/total")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("date", today.date_naive().to_string())
                .query_param("order_by", "")
                .query_param("position", "")
                .query_param("no_competition", "false")
                .query_param("signature_type", (SignatureType::Eoa as u8).to_string());
            then.status(StatusCode::OK).json_body(json!(
                [
                    {
                        "condition_id": "cond_123",
                        "question": "Will BTC be above $50k on December 31, 2025?",
                        "market_slug": "btc-above-50k-2025-12-31",
                        "event_slug": "btc-above-50k-2025",
                        "image": "https://example.com/markets/btc.png",
                        "rewards_max_spread": "0.05",
                        "rewards_min_size": "10.0",
                        "market_competitiveness": "0.80",
                        "tokens": [
                            {
                                "token_id": "YES_TOKEN",
                                "outcome": "YES",
                                "price": "0.55",
                                "winner": true
                            },
                            {
                                "token_id": "NO_TOKEN",
                                "outcome": "NO",
                                "price": "0.45",
                                "winner": false
                            }
                        ],
                        "rewards_config": [
                            {
                                "asset_address": "0x0000000000000000000000000000000000000001",
                                "start_date": "2024-01-01",
                                "end_date": "2024-12-31",
                                "rate_per_day": "1.5",
                                "total_rewards": "500.0"
                            },
                            {
                                "asset_address": "0x0000000000000000000000000000000000000002",
                                "start_date": "2024-06-01",
                                "end_date": "2024-12-31",
                                "rate_per_day": "0.75",
                                "total_rewards": "250.0"
                            }
                        ],
                        "maker_address": "0x1111111111111111111111111111111111111111",
                        "earning_percentage": "0.25",
                        "earnings": [
                            {
                                "asset_address": "0x0000000000000000000000000000000000000001",
                                "earnings": "125.0",
                                "asset_rate": "1.5"
                            },
                            {
                                "asset_address": "0x0000000000000000000000000000000000000002",
                                "earnings": "62.5",
                                "asset_rate": "0.75"
                            }
                        ]
                    }
                ]
            ));
        });

        let request = UserRewardsEarningRequestBuilder::default()
            .date(today.date_naive())
            .build()?;
        let response = client
            .user_earnings_and_markets_config(&request, None)
            .await?;

        let expected = vec![
            UserRewardsEarningResponseBuilder::default()
                .condition_id("cond_123")
                .question("Will BTC be above $50k on December 31, 2025?")
                .market_slug("btc-above-50k-2025-12-31")
                .event_slug("btc-above-50k-2025")
                .image("https://example.com/markets/btc.png")
                .rewards_max_spread(dec!(0.05))
                .rewards_min_size(dec!(10.0))
                .market_competitiveness(dec!(0.80))
                .tokens(vec![
                    TokenBuilder::default()
                        .token_id("YES_TOKEN")
                        .outcome("YES")
                        .price(dec!(0.55))
                        .winner(true)
                        .build()?,
                    TokenBuilder::default()
                        .token_id("NO_TOKEN")
                        .outcome("NO")
                        .price(dec!(0.45))
                        .winner(false)
                        .build()?,
                ])
                .rewards_config(vec![
                    RewardsConfigBuilder::default()
                        .asset_address(address!("0x0000000000000000000000000000000000000001"))
                        .start_date("2024-01-01".parse().unwrap())
                        .end_date("2024-12-31".parse().unwrap())
                        .rate_per_day(dec!(1.5))
                        .total_rewards(dec!(500.0))
                        .build()?,
                    RewardsConfigBuilder::default()
                        .asset_address(address!("0x0000000000000000000000000000000000000002"))
                        .start_date("2024-06-01".parse().unwrap())
                        .end_date("2024-12-31".parse().unwrap())
                        .rate_per_day(dec!(0.75))
                        .total_rewards(dec!(250.0))
                        .build()?,
                ])
                .maker_address(address!("0x1111111111111111111111111111111111111111"))
                .earning_percentage(dec!(0.25))
                .earnings(vec![
                    EarningBuilder::default()
                        .asset_address(address!("0x0000000000000000000000000000000000000001"))
                        .earnings(dec!(125.0))
                        .asset_rate(dec!(1.5))
                        .build()?,
                    EarningBuilder::default()
                        .asset_address(address!("0x0000000000000000000000000000000000000002"))
                        .earnings(dec!(62.5))
                        .asset_rate(dec!(0.75))
                        .build()?,
                ])
                .build()?,
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn reward_percentages_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/rewards/user/percentages")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("signature_type", "0");
            then.status(StatusCode::OK).json_body(json!({ "1": 2 }));
        });

        let response = client.reward_percentages().await?;

        let expected = HashMap::from_iter(vec![("1".to_owned(), Decimal::TWO)]);

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn current_rewards_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/rewards/markets/current")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE);
            then.status(StatusCode::OK).json_body(json!({
                "data": [
                    {
                        "condition_id": "cond_abc123",
                        "rewards_max_spread": "0.05",
                        "rewards_min_size": "20.0",
                        "rewards_config": [
                            {
                                "asset_address": "0x0000000000000000000000000000000000000001",
                                "start_date": "2024-01-01",
                                "end_date": "2024-12-31",
                                "rate_per_day": "2.0",
                                "total_rewards": "750.0"
                            },
                            {
                                "asset_address": "0x0000000000000000000000000000000000000002",
                                "start_date": "2024-06-01",
                                "end_date": "2024-12-31",
                                "rate_per_day": "1.0",
                                "total_rewards": "300.0"
                            }
                        ]
                    }
                ],
                "limit": 1,
                "count": 1,
                "next_cursor": "next"
            }));
        });

        let response = client.current_rewards(None).await?;

        let market_reward = CurrentRewardResponseBuilder::default()
            .condition_id("cond_abc123")
            .rewards_max_spread(dec!(0.05))
            .rewards_min_size(dec!(20.0))
            .rewards_config(vec![
                RewardsConfigBuilder::default()
                    .asset_address(address!("0x0000000000000000000000000000000000000001"))
                    .start_date("2024-01-01".parse().unwrap())
                    .end_date("2024-12-31".parse().unwrap())
                    .rate_per_day(dec!(2.0))
                    .total_rewards(dec!(750.0))
                    .build()?,
                RewardsConfigBuilder::default()
                    .asset_address(address!("0x0000000000000000000000000000000000000002"))
                    .start_date("2024-06-01".parse().unwrap())
                    .end_date("2024-12-31".parse().unwrap())
                    .rate_per_day(dec!(1.0))
                    .total_rewards(dec!(300.0))
                    .build()?,
            ])
            .build()?;
        let expected = PageBuilder::default()
            .limit(1)
            .count(1)
            .next_cursor("next")
            .data(vec![market_reward])
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn raw_rewards_for_market_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = create_authenticated(&server).await?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/rewards/markets/1")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .query_param("next_cursor", "1");
            then.status(StatusCode::OK).json_body(json!({
                "data": [
                    {
                        "condition_id": "1",
                        "question": "Will BTC reach $100k in 2025?",
                        "market_slug": "btc-100k-2025",
                        "event_slug": "btc-2025",
                        "image": "https://example.com/markets/btc.png",
                        "rewards_max_spread": "0.05",
                        "rewards_min_size": "15.0",
                        "market_competitiveness": 0.05,
                        "tokens": [
                            {
                                "token_id": "YES_TOKEN",
                                "outcome": "YES",
                                "price": "0.58",
                                "winner": true
                            },
                            {
                                "token_id": "NO_TOKEN",
                                "outcome": "NO",
                                "price": "0.42",
                                "winner": false
                            }
                        ],
                        "rewards_config": [
                            {
                                "id": "1",
                                "asset_address": "0x0000000000000000000000000000000000000001",
                                "start_date": "2024-01-01",
                                "end_date": "2024-12-31",
                                "rate_per_day": "1.25",
                                "total_rewards": "400.0",
                                "total_days": 10
                            },
                            {
                                "id": "2",
                                "asset_address": "0x0000000000000000000000000000000000000002",
                                "start_date": "2024-06-01",
                                "end_date": "2024-12-31",
                                "rate_per_day": "0.80",
                                "total_rewards": "200.0",
                                "total_days": 10
                            }
                        ]
                    }
                ],
                "limit": 1,
                "count": 1,
                "next_cursor": "2"
            }));
        });

        let response = client
            .raw_rewards_for_market("1", Some("1".to_owned()))
            .await?;

        let market_reward = MarketRewardResponseBuilder::default()
            .condition_id("1")
            .question("Will BTC reach $100k in 2025?")
            .market_slug("btc-100k-2025")
            .event_slug("btc-2025")
            .image("https://example.com/markets/btc.png")
            .rewards_max_spread(dec!(0.05))
            .rewards_min_size(dec!(15.0))
            .market_competitiveness(dec!(0.05))
            .tokens(vec![
                TokenBuilder::default()
                    .token_id("YES_TOKEN")
                    .outcome("YES")
                    .price(dec!(0.58))
                    .winner(true)
                    .build()?,
                TokenBuilder::default()
                    .token_id("NO_TOKEN")
                    .outcome("NO")
                    .price(dec!(0.42))
                    .winner(false)
                    .build()?,
            ])
            .rewards_config(vec![
                MarketRewardsConfigBuilder::default()
                    .id("1")
                    .asset_address(address!("0x0000000000000000000000000000000000000001"))
                    .start_date("2024-01-01".parse().unwrap())
                    .end_date("2024-12-31".parse().unwrap())
                    .rate_per_day(dec!(1.25))
                    .total_rewards(dec!(400.0))
                    .total_days(Decimal::TEN)
                    .build()?,
                MarketRewardsConfigBuilder::default()
                    .id("2")
                    .asset_address(address!("0x0000000000000000000000000000000000000002"))
                    .start_date("2024-06-01".parse().unwrap())
                    .end_date("2024-12-31".parse().unwrap())
                    .rate_per_day(dec!(0.80))
                    .total_rewards(dec!(200.0))
                    .total_days(Decimal::TEN)
                    .build()?,
            ])
            .build()?;
        let expected = PageBuilder::default()
            .limit(1)
            .count(1)
            .next_cursor("2")
            .data(vec![market_reward])
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }
}

mod builder_authenticated {
    use alloy::signers::Signer as _;
    use alloy::signers::local::LocalSigner;
    use httpmock::Method::DELETE;
    use polymarket_client_sdk::auth::builder::Config as BuilderConfig;
    use polymarket_client_sdk::clob::ConfigBuilder;
    use polymarket_client_sdk::clob::types::{
        BuilderApiKeyResponseBuilder, BuilderTradeResponseBuilder, OrderStatusType, PageBuilder,
        Side, TradesRequestBuilder,
    };

    use super::*;
    use crate::common::{
        API_KEY, BUILDER_API_KEY, BUILDER_PASSPHRASE, PASSPHRASE, POLY_BUILDER_API_KEY,
        POLY_BUILDER_PASSPHRASE, POLY_BUILDER_SIGNATURE, POLY_BUILDER_TIMESTAMP, POLY_NONCE,
        POLY_SIGNATURE, POLY_TIMESTAMP, SECRET, SIGNATURE, TIMESTAMP,
    };

    #[tokio::test]
    async fn builder_api_keys_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();

        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/auth/derive-api-key")
                .header(POLY_ADDRESS, signer.address().to_string().to_lowercase())
                .header(POLY_NONCE, "0")
                .header(POLY_SIGNATURE, SIGNATURE)
                .header(POLY_TIMESTAMP, TIMESTAMP);
            then.status(StatusCode::OK).json_body(json!({
                "apiKey": API_KEY,
                "passphrase": PASSPHRASE,
                "secret": SECRET
            }));
        });
        let mock2 = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/time");
            then.status(StatusCode::OK)
                .json_body(TIMESTAMP.parse::<i64>().unwrap());
        });

        let config = ConfigBuilder::default().use_server_time(true).build()?;
        let builder_config = BuilderConfig::remote(&server.base_url(), Some("token".to_owned()))?;
        let client = Client::new(&server.base_url(), config)?
            .authentication_builder(&signer)
            .authenticate()
            .await?;

        let client = client.promote_to_builder(builder_config)?;

        let mock3 = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/")
                .header("authorization", "Bearer token");

            then.status(StatusCode::OK).json_body(json!({
                POLY_BUILDER_API_KEY: BUILDER_API_KEY,
                POLY_BUILDER_PASSPHRASE: BUILDER_PASSPHRASE,
                POLY_BUILDER_SIGNATURE: "signature",
                POLY_BUILDER_TIMESTAMP: "1",
            }));
        });

        let time = Utc::now();
        let mock4 = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/auth/builder-api-key")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .header(POLY_BUILDER_API_KEY, BUILDER_API_KEY)
                .header(POLY_BUILDER_PASSPHRASE, BUILDER_PASSPHRASE)
                .header(POLY_BUILDER_SIGNATURE, "signature")
                .header(POLY_BUILDER_TIMESTAMP, "1");

            then.status(StatusCode::OK).json_body(json!(
                [
                    {
                        "key": Uuid::nil(),
                        "createdAt": time
                    }
                ]
            ));
        });

        let response = client.builder_api_keys().await?;

        let expected = vec![
            BuilderApiKeyResponseBuilder::default()
                .key(Uuid::nil())
                .created_at(time)
                .build()?,
        ];

        assert_eq!(response, expected);
        mock.assert();
        mock2.assert_calls(3);
        mock3.assert();
        mock4.assert();

        Ok(())
    }

    #[tokio::test]
    async fn revoke_builder_api_key_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();

        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/auth/derive-api-key")
                .header(POLY_ADDRESS, signer.address().to_string().to_lowercase())
                .header(POLY_NONCE, "0")
                .header(POLY_SIGNATURE, SIGNATURE)
                .header(POLY_TIMESTAMP, TIMESTAMP);
            then.status(StatusCode::OK).json_body(json!({
                "apiKey": API_KEY,
                "passphrase": PASSPHRASE,
                "secret": SECRET
            }));
        });
        let mock2 = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/time");
            then.status(StatusCode::OK)
                .json_body(TIMESTAMP.parse::<i64>().unwrap());
        });

        let config = ConfigBuilder::default().use_server_time(true).build()?;
        let builder_config = BuilderConfig::remote(&server.base_url(), Some("token".to_owned()))?;
        let client = Client::new(&server.base_url(), config)?
            .authentication_builder(&signer)
            .authenticate()
            .await?;

        let client = client.promote_to_builder(builder_config)?;

        let mock3 = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/")
                .header("authorization", "Bearer token");

            then.status(StatusCode::OK).json_body(json!({
                POLY_BUILDER_API_KEY: BUILDER_API_KEY,
                POLY_BUILDER_PASSPHRASE: BUILDER_PASSPHRASE,
                POLY_BUILDER_SIGNATURE: "signature",
                POLY_BUILDER_TIMESTAMP: "1",
            }));
        });

        let mock4 = server.mock(|when, then| {
            when.method(DELETE)
                .path("/auth/builder-api-key")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .header(POLY_BUILDER_API_KEY, BUILDER_API_KEY)
                .header(POLY_BUILDER_PASSPHRASE, BUILDER_PASSPHRASE)
                .header(POLY_BUILDER_SIGNATURE, "signature")
                .header(POLY_BUILDER_TIMESTAMP, "1");
            then.status(StatusCode::OK).json_body(json!(null));
        });

        client.revoke_builder_api_key().await?;

        mock.assert();
        mock2.assert_calls(3);
        mock3.assert();
        mock4.assert();

        Ok(())
    }

    #[tokio::test]
    async fn builder_trades_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();

        let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));

        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/auth/derive-api-key")
                .header(POLY_ADDRESS, signer.address().to_string().to_lowercase())
                .header(POLY_NONCE, "0")
                .header(POLY_SIGNATURE, SIGNATURE)
                .header(POLY_TIMESTAMP, TIMESTAMP);
            then.status(StatusCode::OK).json_body(json!({
                "apiKey": API_KEY,
                "passphrase": PASSPHRASE,
                "secret": SECRET
            }));
        });
        let mock2 = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/time");
            then.status(StatusCode::OK)
                .json_body(TIMESTAMP.parse::<i64>().unwrap());
        });

        let config = ConfigBuilder::default().use_server_time(true).build()?;
        let builder_config = BuilderConfig::remote(&server.base_url(), Some("token".to_owned()))?;
        let client = Client::new(&server.base_url(), config)?
            .authentication_builder(&signer)
            .authenticate()
            .await?;

        let client = client.promote_to_builder(builder_config)?;

        let mock3 = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/")
                .header("authorization", "Bearer token");

            then.status(StatusCode::OK).json_body(json!({
                POLY_BUILDER_API_KEY: BUILDER_API_KEY,
                POLY_BUILDER_PASSPHRASE: BUILDER_PASSPHRASE,
                POLY_BUILDER_SIGNATURE: "signature",
                POLY_BUILDER_TIMESTAMP: "1",
            }));
        });

        let mock4 = server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/builder/trades")
                .header(POLY_ADDRESS, client.address().to_string().to_lowercase())
                .header(POLY_API_KEY, API_KEY)
                .header(POLY_PASSPHRASE, PASSPHRASE)
                .header(POLY_BUILDER_API_KEY, BUILDER_API_KEY)
                .header(POLY_BUILDER_PASSPHRASE, BUILDER_PASSPHRASE)
                .header(POLY_BUILDER_SIGNATURE, "signature")
                .header(POLY_BUILDER_TIMESTAMP, "1")
                .query_param("id", "1")
                .query_param("market", "market");

            then.status(StatusCode::OK).json_body(json!({
                "data": [
                    {
                        "id": "1",
                        "tradeType": "limit",
                        "takerOrderHash": "0xtakerorderhash",
                        "builder": "builder_1",
                        "market": "market",
                        "assetId": "asset_xyz",
                        "side": "buy",
                        "size": "10.0",
                        "sizeUsdc": "100.0",
                        "price": "0.45",
                        "status": "MATCHED",
                        "outcome": "YES",
                        "outcomeIndex": 0,
                        "owner": "ffffffff-ffff-ffff-ffff-ffffffffffff",
                        "maker": "0x2222222222222222222222222222222222222222",
                        "transactionHash": "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd",
                        "matchTime": "1758579597",
                        "bucketIndex": 3,
                        "fee": "0.1",
                        "feeUsdc": "1.0",
                        "err_msg": "partial fill due to liquidity",
                        "createdAt": "2024-01-15T12:30:00Z",
                        "updatedAt": "2024-01-15T12:35:00Z"
                    }
                ],
                "limit": 1,
                "count": 1,
                "next_cursor": "next"
            }));
        });

        let request = TradesRequestBuilder::default()
            .id("1")
            .market("market")
            .build()?;
        let response = client.builder_trades(&request, None).await?;

        let trade = BuilderTradeResponseBuilder::default()
            .id("1")
            .trade_type("limit")
            .taker_order_hash("0xtakerorderhash")
            .builder("builder_1")
            .market("market")
            .asset_id("asset_xyz")
            .side(Side::Buy)
            .size(dec!(10.0))
            .size_usdc(dec!(100.0))
            .price(dec!(0.45))
            .status(OrderStatusType::Matched)
            .outcome("YES")
            .outcome_index(0)
            .owner(Uuid::max())
            .maker("0x2222222222222222222222222222222222222222")
            .transaction_hash("0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .match_time("2025-09-22T22:19:57Z".parse().unwrap())
            .bucket_index(3)
            .fee(dec!(0.1))
            .fee_usdc(dec!(1.0))
            .err_msg("partial fill due to liquidity")
            .created_at(Some("2024-01-15T12:30:00Z".parse().unwrap()))
            .updated_at(Some("2024-01-15T12:35:00Z".parse().unwrap()))
            .build()?;
        let expected = PageBuilder::default()
            .limit(1)
            .count(1)
            .data(vec![trade])
            .next_cursor("next")
            .build()?;

        assert_eq!(response, expected);
        mock.assert();
        mock2.assert_calls(3);
        mock3.assert();
        mock4.assert();

        Ok(())
    }
}
