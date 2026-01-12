#![cfg(feature = "data")]

use polymarket_client_sdk::types::{Address, B256, U256, address, b256};

const TEST_USER: Address = address!("1234567890abcdef1234567890abcdef12345678");
const TEST_CONDITION_ID: B256 =
    b256!("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");

fn test_user() -> Address {
    TEST_USER
}

fn test_condition_id() -> B256 {
    TEST_CONDITION_ID
}

mod health {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::Client;
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn health_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(StatusCode::OK).json_body(json!({
                "data": "OK"
            }));
        });

        let response = client.health().await?;

        assert_eq!(response.data, "OK");
        mock.assert();

        Ok(())
    }
}

mod positions {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{Client, types::request::PositionsRequest};
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::{test_condition_id, test_user};

    #[tokio::test]
    async fn positions_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/positions")
                .query_param("user", "0x1234567890abcdef1234567890abcdef12345678");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "proxyWallet": "0x1234567890abcdef1234567890abcdef12345678",
                    "asset": "0x1111111111111111111111111111111111111111111111111111111111111111",
                    "conditionId": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                    "size": 100.5,
                    "avgPrice": 0.65,
                    "initialValue": 65.325,
                    "currentValue": 70.35,
                    "cashPnl": 5.025,
                    "percentPnl": 7.69,
                    "totalBought": 100.5,
                    "realizedPnl": 0.0,
                    "percentRealizedPnl": 0.0,
                    "curPrice": 0.70,
                    "redeemable": false,
                    "mergeable": false,
                    "title": "Will BTC hit $100k?",
                    "slug": "btc-100k",
                    "icon": "https://example.com/btc.png",
                    "eventSlug": "crypto-prices",
                    "outcome": "Yes",
                    "outcomeIndex": 0,
                    "oppositeOutcome": "No",
                    "oppositeAsset": "0x1111111111111111111111111111111111111111111111111111111111111111",
                    "endDate": "2025-12-31",
                    "negativeRisk": false
                }
            ]));
        });

        let request = PositionsRequest::builder().user(test_user()).build();

        let response = client.positions(&request).await?;

        assert_eq!(response.len(), 1);
        let pos = &response[0];
        assert_eq!(pos.proxy_wallet, test_user());
        assert_eq!(pos.condition_id, test_condition_id());
        assert_eq!(pos.size, dec!(100.5));
        assert_eq!(pos.title, "Will BTC hit $100k?");
        assert!(!pos.redeemable);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn positions_with_filters_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/positions")
                .query_param("user", "0x1234567890abcdef1234567890abcdef12345678")
                .query_param("limit", "10")
                .query_param("offset", "5")
                .query_param("redeemable", "true");
            then.status(StatusCode::OK).json_body(json!([]));
        });

        let request = PositionsRequest::builder()
            .user(test_user())
            .limit(10)?
            .offset(5)?
            .redeemable(true)
            .build();

        let response = client.positions(&request).await?;

        assert!(response.is_empty());
        mock.assert();

        Ok(())
    }
}

mod trades {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{Client, types::Side, types::request::TradesRequest};
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::{test_condition_id, test_user};

    #[tokio::test]
    async fn trades_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/trades");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "proxyWallet": "0x1234567890abcdef1234567890abcdef12345678",
                    "side": "BUY",
                    "asset": "0x1111111111111111111111111111111111111111111111111111111111111111",
                    "conditionId": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                    "size": 50.0,
                    "price": 0.55,
                    "timestamp": 1_703_980_800,
                    "title": "Market Title",
                    "slug": "market-slug",
                    "icon": "https://example.com/icon.png",
                    "eventSlug": "event-slug",
                    "outcome": "Yes",
                    "outcomeIndex": 0,
                    "name": "Trader Name",
                    "pseudonym": "TraderX",
                    "bio": "A trader",
                    "profileImage": "https://example.com/avatar.png",
                    "profileImageOptimized": "https://example.com/avatar-opt.png",
                    "transactionHash": "0x2222222222222222222222222222222222222222222222222222222222222222"
                }
            ]));
        });

        let response = client.trades(&TradesRequest::default()).await?;

        assert_eq!(response.len(), 1);
        let trade = &response[0];
        assert_eq!(trade.proxy_wallet, test_user());
        assert_eq!(trade.condition_id, test_condition_id());
        assert_eq!(trade.side, Side::Buy);
        assert_eq!(trade.size, dec!(50.0));
        assert_eq!(trade.price, dec!(0.55));
        assert_eq!(trade.timestamp, 1_703_980_800);
        mock.assert();

        Ok(())
    }
}

mod activity {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{
        Client,
        types::request::ActivityRequest,
        types::{ActivityType, Side},
    };
    use reqwest::StatusCode;
    use serde_json::json;

    use super::{test_condition_id, test_user};

    #[tokio::test]
    async fn activity_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/activity")
                .query_param("user", "0x1234567890abcdef1234567890abcdef12345678");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "proxyWallet": "0x1234567890abcdef1234567890abcdef12345678",
                    "timestamp": 1_703_980_800,
                    "conditionId": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                    "type": "TRADE",
                    "size": 100.0,
                    "usdcSize": 55.0,
                    "transactionHash": "0x2222222222222222222222222222222222222222222222222222222222222222",
                    "price": 0.55,
                    "asset": "0x1111111111111111111111111111111111111111111111111111111111111111",
                    "side": "BUY",
                    "outcomeIndex": 0,
                    "title": "Market",
                    "slug": "market-slug",
                    "outcome": "Yes"
                },
                {
                    "proxyWallet": "0x1234567890abcdef1234567890abcdef12345678",
                    "timestamp": 1_703_980_900,
                    "conditionId": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                    "type": "REDEEM",
                    "size": 100.0,
                    "usdcSize": 100.0,
                    "transactionHash": "0x2222222222222222222222222222222222222222222222222222222222222222"
                }
            ]));
        });

        let request = ActivityRequest::builder().user(test_user()).build();

        let response = client.activity(&request).await?;

        assert_eq!(response.len(), 2);
        assert_eq!(response[0].proxy_wallet, test_user());
        assert_eq!(response[0].condition_id, Some(test_condition_id()));
        assert_eq!(response[0].activity_type, ActivityType::Trade);
        assert_eq!(response[0].side, Some(Side::Buy));
        assert_eq!(response[1].activity_type, ActivityType::Redeem);
        mock.assert();

        Ok(())
    }
}

mod holders {
    use std::str::FromStr as _;

    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{Client, types::request::HoldersRequest};
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::{U256, address, test_condition_id, test_user};

    #[tokio::test]
    async fn holders_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let holder2 = address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/holders")
                .query_param(
                    "market",
                    "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                );
            then.status(StatusCode::OK).json_body(json!([
                {
                    "token": "0x1111111111111111111111111111111111111111111111111111111111111111",
                    "holders": [
                        {
                            "proxyWallet": "0x1234567890abcdef1234567890abcdef12345678",
                            "bio": "Whale trader",
                            "asset": "0x1111111111111111111111111111111111111111111111111111111111111111",
                            "pseudonym": "WhaleX",
                            "amount": 10000.0,
                            "displayUsernamePublic": true,
                            "outcomeIndex": 0,
                            "name": "Holder One",
                            "profileImage": "https://example.com/h1.png"
                        },
                        {
                            "proxyWallet": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                            "asset": "0x1111111111111111111111111111111111111111111111111111111111111111",
                            "amount": 5000.0,
                            "outcomeIndex": 0
                        }
                    ]
                }
            ]));
        });

        let request = HoldersRequest::builder()
            .markets(vec![test_condition_id()])
            .build();

        let response = client.holders(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0].token,
            U256::from_str("0x1111111111111111111111111111111111111111111111111111111111111111")?
        );
        let holders = &response[0].holders;
        assert_eq!(holders.len(), 2);
        assert_eq!(holders[0].proxy_wallet, test_user());
        assert_eq!(holders[0].amount, dec!(10000.0));
        assert_eq!(holders[1].proxy_wallet, holder2);
        assert_eq!(holders[1].amount, dec!(5000.0));
        mock.assert();

        Ok(())
    }
}

mod value {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{Client, types::request::ValueRequest};
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::test_user;

    #[tokio::test]
    async fn value_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/value")
                .query_param("user", "0x1234567890abcdef1234567890abcdef12345678");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "user": "0x1234567890abcdef1234567890abcdef12345678",
                    "value": 12345.67
                }
            ]));
        });

        let request = ValueRequest::builder().user(test_user()).build();

        let response = client.value(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].user, test_user());
        assert_eq!(response[0].value, dec!(12345.67));
        mock.assert();

        Ok(())
    }
}

mod closed_positions {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{Client, types::request::ClosedPositionsRequest};
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::{test_condition_id, test_user};

    #[tokio::test]
    async fn closed_positions_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/closed-positions")
                .query_param("user", "0x1234567890abcdef1234567890abcdef12345678");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "proxyWallet": "0x1234567890abcdef1234567890abcdef12345678",
                    "asset": "0x1111111111111111111111111111111111111111111111111111111111111111",
                    "conditionId": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                    "avgPrice": 0.45,
                    "totalBought": 100.0,
                    "realizedPnl": 55.0,
                    "curPrice": 1.0,
                    "timestamp": 1_703_980_800,
                    "title": "Resolved Market",
                    "slug": "resolved-market",
                    "icon": "https://example.com/icon.png",
                    "eventSlug": "event-slug",
                    "outcome": "Yes",
                    "outcomeIndex": 0,
                    "oppositeOutcome": "No",
                    "oppositeAsset": "0x1111111111111111111111111111111111111111111111111111111111111111",
                    "endDate": "2025-12-31T00:00:00Z",
                }
            ]));
        });

        let request = ClosedPositionsRequest::builder().user(test_user()).build();

        let response = client.closed_positions(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].proxy_wallet, test_user());
        assert_eq!(response[0].condition_id, test_condition_id());
        assert_eq!(response[0].realized_pnl, dec!(55.0));
        assert_eq!(response[0].cur_price, dec!(1.0));
        assert_eq!(response[0].timestamp, 1_703_980_800);
        mock.assert();

        Ok(())
    }
}

mod leaderboard {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{
        Client,
        types::request::TraderLeaderboardRequest,
        types::{LeaderboardCategory, LeaderboardOrderBy, TimePeriod},
    };
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::{address, test_user};

    #[tokio::test]
    async fn leaderboard_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let second_user = address!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");

        let mock = server.mock(|when, then| {
            when.method(GET).path("/v1/leaderboard");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "rank": "1",
                    "proxyWallet": "0x1234567890abcdef1234567890abcdef12345678",
                    "userName": "TopTrader",
                    "vol": 1_000_000.0,
                    "pnl": 150_000.0,
                    "profileImage": "https://example.com/top.png",
                    "xUsername": "toptrader",
                    "verifiedBadge": true
                },
                {
                    "rank": "2",
                    "proxyWallet": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    "userName": "SecondPlace",
                    "vol": 500_000.0,
                    "pnl": 75_000.0,
                    "verifiedBadge": false
                }
            ]));
        });

        let request = TraderLeaderboardRequest::builder().build();

        let response = client.leaderboard(&request).await?;

        assert_eq!(response.len(), 2);
        assert_eq!(response[0].rank, 1);
        assert_eq!(response[0].proxy_wallet, test_user());
        assert_eq!(response[0].pnl, dec!(150_000.0));
        assert_eq!(response[0].verified_badge, Some(true));
        assert_eq!(response[1].rank, 2);
        assert_eq!(response[1].proxy_wallet, second_user);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn leaderboard_with_filters_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v1/leaderboard")
                .query_param("category", "POLITICS")
                .query_param("timePeriod", "WEEK")
                .query_param("orderBy", "VOL")
                .query_param("limit", "10");
            then.status(StatusCode::OK).json_body(json!([]));
        });

        let request = TraderLeaderboardRequest::builder()
            .category(LeaderboardCategory::Politics)
            .time_period(TimePeriod::Week)
            .order_by(LeaderboardOrderBy::Vol)
            .limit(10)?
            .build();

        let response = client.leaderboard(&request).await?;

        assert!(response.is_empty());
        mock.assert();

        Ok(())
    }
}

mod traded {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{Client, types::request::TradedRequest};
    use reqwest::StatusCode;
    use serde_json::json;

    use super::test_user;

    #[tokio::test]
    async fn traded_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/traded")
                .query_param("user", "0x1234567890abcdef1234567890abcdef12345678");
            then.status(StatusCode::OK).json_body(json!({
                "user": "0x1234567890abcdef1234567890abcdef12345678",
                "traded": 42
            }));
        });

        let request = TradedRequest::builder().user(test_user()).build();

        let response = client.traded(&request).await?;

        assert_eq!(response.user, test_user());
        assert_eq!(response.traded, 42);
        mock.assert();

        Ok(())
    }
}

mod open_interest {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::types::response::Market;
    use polymarket_client_sdk::data::{Client, types::request::OpenInterestRequest};
    use polymarket_client_sdk::types::b256;
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    use super::test_condition_id;

    #[tokio::test]
    async fn open_interest_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/oi");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "market": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                    "value": 1_500_000.0
                },
                {
                    "market": "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                    "value": 750_000.0
                },
                {
                    "market": "GLOBAL",
                    "value": 2_250_000.0
                }
            ]));
        });

        let response = client
            .open_interest(&OpenInterestRequest::default())
            .await?;

        assert_eq!(response.len(), 3);
        assert_eq!(
            response[0].market,
            Market::Market(b256!(
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            ))
        );
        assert_eq!(response[0].value, dec!(1_500_000.0));
        assert_eq!(
            response[1].market,
            Market::Market(b256!(
                "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            ))
        );
        assert_eq!(response[2].market, Market::Global);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn open_interest_with_market_filter_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/oi").query_param(
                "market",
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            );
            then.status(StatusCode::OK).json_body(json!([
                {
                    "market": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                    "value": 500_000.0
                }
            ]));
        });

        let request = OpenInterestRequest::builder()
            .markets(vec![test_condition_id()])
            .build();

        let response = client.open_interest(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(
            response[0].market,
            Market::Market(b256!(
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            ))
        );
        mock.assert();

        Ok(())
    }
}

mod live_volume {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::types::response::Market;
    use polymarket_client_sdk::data::{Client, types::request::LiveVolumeRequest};
    use polymarket_client_sdk::types::b256;
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    #[tokio::test]
    async fn live_volume_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/live-volume")
                .query_param("id", "123");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "total": 250_000.0,
                    "markets": [
                        {
                            "market": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                            "value": 150_000.0
                        },
                        {
                            "market": "0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
                            "value": 100_000.0
                        }
                    ]
                }
            ]));
        });

        let request = LiveVolumeRequest::builder().id(123).build();

        let response = client.live_volume(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].total, dec!(250_000.0));
        let markets = &response[0].markets;
        assert_eq!(markets.len(), 2);
        assert_eq!(
            markets[0].market,
            Market::Market(b256!(
                "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            ))
        );
        assert_eq!(markets[0].value, dec!(150_000.0));
        assert_eq!(
            markets[1].market,
            Market::Market(b256!(
                "0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            ))
        );
        mock.assert();

        Ok(())
    }
}

mod builder_leaderboard {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{
        Client, types::TimePeriod, types::request::BuilderLeaderboardRequest,
    };
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    #[tokio::test]
    async fn builder_leaderboard_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/v1/builders/leaderboard");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "rank": "1",
                    "builder": "TopBuilder",
                    "volume": 5_000_000.0,
                    "activeUsers": 1500,
                    "verified": true,
                    "builderLogo": "https://example.com/builder1.png"
                },
                {
                    "rank": "2",
                    "builder": "SecondBuilder",
                    "volume": 2_500_000.0,
                    "activeUsers": 800,
                    "verified": false
                }
            ]));
        });

        let request = BuilderLeaderboardRequest::builder().build();

        let response = client.builder_leaderboard(&request).await?;

        assert_eq!(response.len(), 2);
        assert_eq!(response[0].rank, 1);
        assert_eq!(response[0].builder, "TopBuilder");
        assert_eq!(response[0].volume, dec!(5_000_000.0));
        assert_eq!(response[0].active_users, 1500);
        assert!(response[0].verified);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn builder_leaderboard_with_time_period_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v1/builders/leaderboard")
                .query_param("timePeriod", "MONTH")
                .query_param("limit", "5");
            then.status(StatusCode::OK).json_body(json!([]));
        });

        let request = BuilderLeaderboardRequest::builder()
            .time_period(TimePeriod::Month)
            .limit(5)?
            .build();

        let response = client.builder_leaderboard(&request).await?;

        assert!(response.is_empty());
        mock.assert();

        Ok(())
    }
}

mod builder_volume {
    use std::str::FromStr as _;

    use chrono::{DateTime, Utc};
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{
        Client, types::TimePeriod, types::request::BuilderVolumeRequest,
    };
    use reqwest::StatusCode;
    use rust_decimal_macros::dec;
    use serde_json::json;

    #[tokio::test]
    async fn builder_volume_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/v1/builders/volume");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "dt": "2025-01-15T00:00:00Z",
                    "builder": "Builder1",
                    "builderLogo": "https://example.com/b1.png",
                    "verified": true,
                    "volume": 100_000.0,
                    "activeUsers": 250,
                    "rank": "1"
                },
                {
                    "dt": "2025-01-14T00:00:00Z",
                    "builder": "Builder1",
                    "builderLogo": "https://example.com/b1.png",
                    "verified": true,
                    "volume": 95_000.0,
                    "activeUsers": 230,
                    "rank": "1"
                }
            ]));
        });

        let request = BuilderVolumeRequest::builder().build();

        let response = client.builder_volume(&request).await?;

        assert_eq!(response.len(), 2);
        assert_eq!(
            response[0].dt,
            DateTime::<Utc>::from_str("2025-01-15T00:00:00Z")?
        );
        assert_eq!(response[0].builder, "Builder1");
        assert_eq!(response[0].volume, dec!(100_000.0));
        assert!(response[0].verified);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn builder_volume_with_time_period_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v1/builders/volume")
                .query_param("timePeriod", "WEEK");
            then.status(StatusCode::OK).json_body(json!([]));
        });

        let request = BuilderVolumeRequest::builder()
            .time_period(TimePeriod::Week)
            .build();

        let response = client.builder_volume(&request).await?;

        assert!(response.is_empty());
        mock.assert();

        Ok(())
    }
}

mod error_handling {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data::{Client, types::request::PositionsRequest};
    use polymarket_client_sdk::error::Kind;
    use reqwest::StatusCode;
    use serde_json::json;

    use super::test_user;

    #[tokio::test]
    async fn bad_request_should_return_error() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/positions");
            then.status(StatusCode::BAD_REQUEST).json_body(json!({
                "error": "Invalid user address"
            }));
        });

        let request = PositionsRequest::builder().user(test_user()).build();

        let result = client.positions(&request).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), Kind::Status);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn server_error_should_return_error() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/positions");
            then.status(StatusCode::INTERNAL_SERVER_ERROR)
                .json_body(json!({
                    "error": "Internal server error"
                }));
        });

        let request = PositionsRequest::builder().user(test_user()).build();

        let result = client.positions(&request).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), Kind::Status);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn null_response_should_return_error() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/positions");
            then.status(StatusCode::OK).body("null");
        });

        let request = PositionsRequest::builder().user(test_user()).build();

        let result = client.positions(&request).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), Kind::Status);
        mock.assert();

        Ok(())
    }
}

mod client {
    use polymarket_client_sdk::data::Client;

    #[test]
    fn client_default_should_succeed() {
        let client = Client::default();
        assert_eq!(client.host().as_str(), "https://data-api.polymarket.com/");
    }

    #[test]
    fn client_new_with_custom_host_should_succeed() -> anyhow::Result<()> {
        let client = Client::new("https://custom-api.example.com")?;
        assert_eq!(client.host().as_str(), "https://custom-api.example.com/");
        Ok(())
    }

    #[test]
    fn client_new_with_invalid_url_should_fail() {
        Client::new("not-a-valid-url").unwrap_err();
    }
}

mod types {
    use polymarket_client_sdk::ToQueryParams as _;
    use polymarket_client_sdk::data::{
        types::request::{
            ActivityRequest, BuilderLeaderboardRequest, HoldersRequest, LiveVolumeRequest,
            PositionsRequest, TradedRequest, TraderLeaderboardRequest, TradesRequest,
        },
        types::{
            ActivityType, BoundedIntError, LeaderboardCategory, LeaderboardOrderBy, MarketFilter,
            PositionSortBy, Side, SortDirection, TimePeriod, TradeFilter,
        },
    };
    use rust_decimal_macros::dec;

    use super::{address, b256};

    #[test]
    fn bounded_limits() {
        // Test that the builder validates bounds correctly
        // PositionsRequest limit: 0-500
        drop(
            PositionsRequest::builder()
                .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
                .limit(0)
                .unwrap()
                .build(),
        );
        drop(
            PositionsRequest::builder()
                .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
                .limit(500)
                .unwrap()
                .build(),
        );
        let err = PositionsRequest::builder()
            .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
            .limit(501);
        assert!(matches!(err, Err(BoundedIntError { .. })));

        // HoldersRequest limit: 0-20
        drop(
            HoldersRequest::builder()
                .markets(vec![])
                .limit(0)
                .unwrap()
                .build(),
        );
        drop(
            HoldersRequest::builder()
                .markets(vec![])
                .limit(20)
                .unwrap()
                .build(),
        );
        let err = HoldersRequest::builder().markets(vec![]).limit(21);
        assert!(matches!(err, Err(BoundedIntError { .. })));

        // TraderLeaderboardRequest limit: 1-50
        let err = TraderLeaderboardRequest::builder().limit(0);
        assert!(matches!(err, Err(BoundedIntError { .. })));
        drop(
            TraderLeaderboardRequest::builder()
                .limit(1)
                .unwrap()
                .build(),
        );
        drop(
            TraderLeaderboardRequest::builder()
                .limit(50)
                .unwrap()
                .build(),
        );
        let err = TraderLeaderboardRequest::builder().limit(51);
        assert!(matches!(err, Err(BoundedIntError { .. })));

        // BuilderLeaderboardRequest limit: 0-50
        drop(
            BuilderLeaderboardRequest::builder()
                .limit(0)
                .unwrap()
                .build(),
        );
        drop(
            BuilderLeaderboardRequest::builder()
                .limit(50)
                .unwrap()
                .build(),
        );
        let err = BuilderLeaderboardRequest::builder().limit(51);
        assert!(matches!(err, Err(BoundedIntError { .. })));
    }

    #[test]
    fn positions_request_query_string() {
        let req = PositionsRequest::builder()
            .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
            .limit(50)
            .unwrap()
            .sort_by(PositionSortBy::CashPnl)
            .sort_direction(SortDirection::Desc)
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("user=0x"));
        assert!(qs.contains("limit=50"));
        assert!(qs.contains("sortBy=CASHPNL"));
        assert!(qs.contains("sortDirection=DESC"));
    }

    #[test]
    fn market_filter_query_string() {
        let hash1 = b256!("dd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917");
        let hash2 = b256!("aa22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917");

        let req = PositionsRequest::builder()
            .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
            .filter(MarketFilter::markets([hash1, hash2]))
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("market="));
        assert!(qs.contains("%2C")); // URL-encoded comma
        assert!(!qs.contains("eventId="));
    }

    #[test]
    fn event_id_filter_query_string() {
        let req = PositionsRequest::builder()
            .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
            .filter(MarketFilter::event_ids(["1".to_owned(), "2".to_owned()]))
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("eventId=1%2C2")); // URL-encoded "1,2"
        assert!(!qs.contains("market="));
    }

    #[test]
    fn trade_filter() {
        TradeFilter::cash(dec!(100.0)).unwrap();
        TradeFilter::tokens(dec!(0.0)).unwrap();
        TradeFilter::cash(dec!(-1.0)).unwrap_err();
    }

    #[test]
    fn trades_request_with_filter() {
        let req = TradesRequest::builder()
            .trade_filter(TradeFilter::cash(dec!(100.0)).unwrap())
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("filterType=CASH"));
        assert!(qs.contains("filterAmount=100"));
    }

    #[test]
    fn activity_types_query_string() {
        let req = ActivityRequest::builder()
            .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
            .activity_types(vec![ActivityType::Trade, ActivityType::Redeem])
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("type=TRADE%2CREDEEM")); // URL-encoded "TRADE,REDEEM"
    }

    #[test]
    fn live_volume_request() {
        let req = LiveVolumeRequest::builder().id(123).build();

        let qs = req.query_params(None);
        assert!(qs.contains("id=123"));
    }

    #[test]
    fn traded_request() {
        let req = TradedRequest::builder()
            .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("user=0x"));
    }

    #[test]
    fn trader_leaderboard_request() {
        let req = TraderLeaderboardRequest::builder()
            .category(LeaderboardCategory::Politics)
            .time_period(TimePeriod::Week)
            .order_by(LeaderboardOrderBy::Pnl)
            .limit(10)
            .unwrap()
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("category=POLITICS"));
        assert!(qs.contains("timePeriod=WEEK"));
        assert!(qs.contains("orderBy=PNL"));
        assert!(qs.contains("limit=10"));
    }

    #[test]
    fn enum_display() {
        assert_eq!(Side::Buy.to_string(), "BUY");
        assert_eq!(Side::Sell.to_string(), "SELL");
        assert_eq!(ActivityType::Trade.to_string(), "TRADE");
        assert_eq!(PositionSortBy::CashPnl.to_string(), "CASHPNL");
        assert_eq!(PositionSortBy::PercentPnl.to_string(), "PERCENTPNL");
        assert_eq!(TimePeriod::All.to_string(), "ALL");
        assert_eq!(LeaderboardCategory::Overall.to_string(), "OVERALL");
    }

    #[test]
    fn all_activity_types_display() {
        use polymarket_client_sdk::data::types::ActivityType;
        assert_eq!(ActivityType::Split.to_string(), "SPLIT");
        assert_eq!(ActivityType::Merge.to_string(), "MERGE");
        assert_eq!(ActivityType::Redeem.to_string(), "REDEEM");
        assert_eq!(ActivityType::Reward.to_string(), "REWARD");
        assert_eq!(ActivityType::Conversion.to_string(), "CONVERSION");
    }

    #[test]
    fn all_position_sort_by_display() {
        assert_eq!(PositionSortBy::Current.to_string(), "CURRENT");
        assert_eq!(PositionSortBy::Initial.to_string(), "INITIAL");
        assert_eq!(PositionSortBy::Tokens.to_string(), "TOKENS");
        assert_eq!(PositionSortBy::Title.to_string(), "TITLE");
        assert_eq!(PositionSortBy::Resolving.to_string(), "RESOLVING");
        assert_eq!(PositionSortBy::Price.to_string(), "PRICE");
        assert_eq!(PositionSortBy::AvgPrice.to_string(), "AVGPRICE");
    }

    #[test]
    fn all_time_periods_display() {
        assert_eq!(TimePeriod::Day.to_string(), "DAY");
        assert_eq!(TimePeriod::Week.to_string(), "WEEK");
        assert_eq!(TimePeriod::Month.to_string(), "MONTH");
    }

    #[test]
    fn all_leaderboard_categories_display() {
        assert_eq!(LeaderboardCategory::Politics.to_string(), "POLITICS");
        assert_eq!(LeaderboardCategory::Sports.to_string(), "SPORTS");
        assert_eq!(LeaderboardCategory::Crypto.to_string(), "CRYPTO");
        assert_eq!(LeaderboardCategory::Culture.to_string(), "CULTURE");
        assert_eq!(LeaderboardCategory::Mentions.to_string(), "MENTIONS");
        assert_eq!(LeaderboardCategory::Weather.to_string(), "WEATHER");
        assert_eq!(LeaderboardCategory::Economics.to_string(), "ECONOMICS");
        assert_eq!(LeaderboardCategory::Tech.to_string(), "TECH");
        assert_eq!(LeaderboardCategory::Finance.to_string(), "FINANCE");
    }

    #[test]
    fn leaderboard_order_by_display() {
        assert_eq!(LeaderboardOrderBy::Vol.to_string(), "VOL");
    }

    #[test]
    fn sort_direction_display() {
        assert_eq!(SortDirection::Asc.to_string(), "ASC");
    }
}

mod error_display {
    use polymarket_client_sdk::data::{types::TradeFilter, types::request::PositionsRequest};
    use rust_decimal_macros::dec;

    use super::address;

    #[test]
    fn bounded_int_error_display() {
        let err = PositionsRequest::builder()
            .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
            .limit(501);
        let Err(err) = err else {
            panic!("Expected an error")
        };
        assert!(err.to_string().contains("500"));
        assert!(err.to_string().contains("501"));
    }

    #[test]
    fn trade_filter_error_display() {
        let err = TradeFilter::cash(dec!(-1.0)).unwrap_err();
        assert!(err.to_string().contains("-1"));
    }
}

mod request_query_string_extended {
    use polymarket_client_sdk::ToQueryParams as _;
    use polymarket_client_sdk::data::types::{
        ActivitySortBy, ClosedPositionSortBy, MarketFilter, PositionSortBy, Side, SortDirection,
        TradeFilter,
        request::{
            ActivityRequest, BuilderLeaderboardRequest, ClosedPositionsRequest, HoldersRequest,
            OpenInterestRequest, PositionsRequest, TraderLeaderboardRequest, TradesRequest,
            ValueRequest,
        },
    };
    use rust_decimal_macros::dec;

    use super::{Address, B256, address, b256};

    fn test_addr() -> Address {
        address!("56687bf447db6ffa42ffe2204a05edaa20f55839")
    }

    fn test_hash() -> B256 {
        b256!("dd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917")
    }

    #[test]
    fn positions_request_full() {
        let req = PositionsRequest::builder()
            .user(test_addr())
            .size_threshold(dec!(100))
            .mergeable(true)
            .sort_by(PositionSortBy::Current)
            .title("test")
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("sizeThreshold="));
        assert!(qs.contains("mergeable="));
        assert!(qs.contains("sortBy="));
        assert!(qs.contains("title="));
    }

    #[test]
    fn trades_request_full() {
        let req = TradesRequest::builder()
            .user(test_addr())
            .filter(MarketFilter::markets([test_hash()]))
            .limit(50)
            .unwrap()
            .taker_only(true)
            .side(Side::Buy)
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("user="));
        assert!(qs.contains("market="));
        assert!(qs.contains("limit="));
        assert!(qs.contains("takerOnly="));
        assert!(qs.contains("side="));
    }

    #[test]
    fn activity_request_full() {
        let req = ActivityRequest::builder()
            .user(test_addr())
            .filter(MarketFilter::event_ids(["1".to_owned()]))
            .limit(50)
            .unwrap()
            .start(1000)
            .end(2000)
            .sort_by(ActivitySortBy::Timestamp)
            .sort_direction(SortDirection::Asc)
            .side(Side::Sell)
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("eventId="));
        assert!(qs.contains("start="));
        assert!(qs.contains("end="));
        assert!(qs.contains("sortBy="));
        assert!(qs.contains("sortDirection="));
        assert!(qs.contains("side="));
    }

    #[test]
    fn holders_request_full() {
        let req = HoldersRequest::builder()
            .markets(vec![test_hash()])
            .min_balance(10)
            .unwrap()
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("minBalance="));
    }

    #[test]
    fn value_request_with_markets() {
        let req = ValueRequest::builder()
            .user(test_addr())
            .markets(vec![test_hash()])
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("market="));
    }

    #[test]
    fn closed_positions_request_full() {
        let req = ClosedPositionsRequest::builder()
            .user(test_addr())
            .filter(MarketFilter::markets([test_hash()]))
            .title("test")
            .limit(10)
            .unwrap()
            .sort_by(ClosedPositionSortBy::RealizedPnl)
            .sort_direction(SortDirection::Desc)
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("market="));
        assert!(qs.contains("title="));
        assert!(qs.contains("sortBy="));
        assert!(qs.contains("sortDirection="));
    }

    #[test]
    fn builder_leaderboard_request_full() {
        let req = BuilderLeaderboardRequest::builder()
            .offset(10)
            .unwrap()
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("offset="));
    }

    #[test]
    fn trader_leaderboard_request_full() {
        let req = TraderLeaderboardRequest::builder()
            .user(test_addr())
            .user_name("testuser".to_owned())
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("user="));
        assert!(qs.contains("userName="));
    }

    #[test]
    fn trade_filter_tokens() {
        let req = TradesRequest::builder()
            .trade_filter(TradeFilter::tokens(dec!(50.0)).unwrap())
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("filterType=TOKENS"));
    }

    #[test]
    fn empty_market_filter_not_added() {
        let req = PositionsRequest::builder()
            .user(test_addr())
            .filter(MarketFilter::markets([] as [B256; 0]))
            .build();

        let qs = req.query_params(None);
        assert!(!qs.contains("market="));
    }

    #[test]
    fn empty_event_id_filter_not_added() {
        let req = PositionsRequest::builder()
            .user(test_addr())
            .filter(MarketFilter::event_ids([]))
            .build();

        let qs = req.query_params(None);
        assert!(!qs.contains("eventId="));
    }

    #[test]
    fn empty_activity_types_not_added() {
        let req = ActivityRequest::builder()
            .user(test_addr())
            .activity_types(vec![])
            .build();

        let qs = req.query_params(None);
        assert!(!qs.contains("type="));
    }

    #[test]
    fn empty_holders_markets_not_added() {
        let req = HoldersRequest::builder()
            .markets(Vec::<B256>::new())
            .build();

        let qs = req.query_params(None);
        assert!(!qs.contains("market="));
    }

    #[test]
    fn empty_value_markets_not_added() {
        let req = ValueRequest::builder()
            .user(test_addr())
            .markets(Vec::<B256>::new())
            .build();

        let qs = req.query_params(None);
        assert!(!qs.contains("market="));
    }

    #[test]
    fn closed_position_sort_by_variants() {
        use polymarket_client_sdk::data::types::ClosedPositionSortBy;
        assert_eq!(ClosedPositionSortBy::Title.to_string(), "TITLE");
        assert_eq!(ClosedPositionSortBy::Price.to_string(), "PRICE");
        assert_eq!(ClosedPositionSortBy::AvgPrice.to_string(), "AVGPRICE");
        assert_eq!(ClosedPositionSortBy::Timestamp.to_string(), "TIMESTAMP");
    }

    #[test]
    fn activity_sort_by_variants() {
        assert_eq!(ActivitySortBy::Tokens.to_string(), "TOKENS");
        assert_eq!(ActivitySortBy::Cash.to_string(), "CASH");
    }

    #[test]
    fn filter_type_display() {
        use polymarket_client_sdk::data::types::FilterType;
        assert_eq!(FilterType::Cash.to_string(), "CASH");
        assert_eq!(FilterType::Tokens.to_string(), "TOKENS");
    }

    #[test]
    fn empty_request_query_string() {
        let req = TradesRequest::default();
        let qs = req.query_params(None);
        assert!(qs.is_empty());
    }

    #[test]
    fn trades_request_with_offset() {
        let req = TradesRequest::builder().offset(100).unwrap().build();

        let qs = req.query_params(None);
        assert!(qs.contains("offset=100"));
    }

    #[test]
    fn open_interest_request_with_markets() {
        let req = OpenInterestRequest::builder()
            .markets(vec![test_hash()])
            .build();

        let qs = req.query_params(None);
        assert!(qs.contains("market="));
    }

    #[test]
    fn open_interest_request_empty_markets() {
        let req = OpenInterestRequest::builder()
            .markets(Vec::<B256>::new())
            .build();

        let qs = req.query_params(None);
        assert!(!qs.contains("market="));
    }

    #[test]
    fn closed_position_sort_by_realized_pnl() {
        assert_eq!(ClosedPositionSortBy::RealizedPnl.to_string(), "REALIZEDPNL");
    }
}
