#![cfg(feature = "data-api")]

use polymarket_client_sdk::data_api::types::{Address, Hash64};

const TEST_USER_STR: &str = "0x1234567890abcdef1234567890abcdef12345678";
const TEST_CONDITION_ID_STR: &str =
    "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
const TEST_ASSET_STR: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";

fn test_user() -> Address {
    Address::new(TEST_USER_STR).expect("valid test user address")
}

fn test_condition_id() -> Hash64 {
    Hash64::new(TEST_CONDITION_ID_STR).expect("valid test condition id")
}

fn test_asset() -> Hash64 {
    Hash64::new(TEST_ASSET_STR).expect("valid test asset")
}

mod health {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::Client;
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
    use polymarket_client_sdk::data_api::{
        Client,
        types::{PositionsLimit, PositionsOffset, PositionsRequest},
    };
    use reqwest::StatusCode;
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
        assert_eq!(pos.proxy_wallet.as_str(), test_user().as_str());
        assert_eq!(pos.condition_id.as_str(), test_condition_id().as_str());
        assert!((pos.size - 100.5).abs() < f64::EPSILON);
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
            .limit(PositionsLimit::new(10)?)
            .offset(PositionsOffset::new(5)?)
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
    use polymarket_client_sdk::data_api::{
        Client,
        types::{Side, TradesRequest},
    };
    use reqwest::StatusCode;
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
        assert_eq!(trade.proxy_wallet.as_str(), test_user().as_str());
        assert_eq!(trade.condition_id.as_str(), test_condition_id().as_str());
        assert_eq!(trade.side, Side::Buy);
        assert!((trade.size - 50.0).abs() < f64::EPSILON);
        assert!((trade.price - 0.55).abs() < f64::EPSILON);
        assert_eq!(trade.timestamp, 1_703_980_800);
        mock.assert();

        Ok(())
    }
}

mod activity {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{
        Client,
        types::{ActivityRequest, ActivityType, Side},
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
        assert_eq!(response[0].proxy_wallet.as_str(), test_user().as_str());
        assert_eq!(
            response[0].condition_id.as_str(),
            test_condition_id().as_str()
        );
        assert_eq!(response[0].activity_type, ActivityType::Trade);
        assert_eq!(response[0].side, Some(Side::Buy));
        assert_eq!(response[1].activity_type, ActivityType::Redeem);
        mock.assert();

        Ok(())
    }
}

mod holders {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{
        Client,
        types::{Address, HoldersRequest},
    };
    use reqwest::StatusCode;
    use serde_json::json;

    use super::{test_asset, test_condition_id, test_user};

    #[tokio::test]
    async fn holders_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let holder2 = Address::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")?;

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
        assert_eq!(response[0].token, test_asset().as_str());
        let holders = &response[0].holders;
        assert_eq!(holders.len(), 2);
        assert_eq!(holders[0].proxy_wallet.as_str(), test_user().as_str());
        assert!((holders[0].amount - 10000.0).abs() < f64::EPSILON);
        assert_eq!(holders[1].proxy_wallet.as_str(), holder2.as_str());
        assert!((holders[1].amount - 5000.0).abs() < f64::EPSILON);
        mock.assert();

        Ok(())
    }
}

mod value {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{Client, types::ValueRequest};
    use reqwest::StatusCode;
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
        assert_eq!(response[0].user.as_str(), test_user().as_str());
        assert!((response[0].value - 12345.67).abs() < f64::EPSILON);
        mock.assert();

        Ok(())
    }
}

mod closed_positions {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{Client, types::ClosedPositionsRequest};
    use reqwest::StatusCode;
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
                    "endDate": "2025-12-31"
                }
            ]));
        });

        let request = ClosedPositionsRequest::builder().user(test_user()).build();

        let response = client.closed_positions(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].proxy_wallet.as_str(), test_user().as_str());
        assert_eq!(
            response[0].condition_id.as_str(),
            test_condition_id().as_str()
        );
        assert!((response[0].realized_pnl - 55.0).abs() < f64::EPSILON);
        assert!((response[0].cur_price - 1.0).abs() < f64::EPSILON);
        assert_eq!(response[0].timestamp, 1_703_980_800);
        mock.assert();

        Ok(())
    }
}

mod leaderboard {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{
        Client,
        types::{
            Address, LeaderboardCategory, LeaderboardOrderBy, TimePeriod, TraderLeaderboardLimit,
            TraderLeaderboardRequest,
        },
    };
    use reqwest::StatusCode;
    use serde_json::json;

    use super::test_user;

    #[tokio::test]
    async fn leaderboard_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let second_user = Address::new("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")?;

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
        assert_eq!(response[0].rank, "1");
        assert_eq!(response[0].proxy_wallet.as_str(), test_user().as_str());
        assert!((response[0].pnl - 150_000.0).abs() < f64::EPSILON);
        assert_eq!(response[0].verified_badge, Some(true));
        assert_eq!(response[1].rank, "2");
        assert_eq!(response[1].proxy_wallet.as_str(), second_user.as_str());
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
            .limit(TraderLeaderboardLimit::new(10)?)
            .build();

        let response = client.leaderboard(&request).await?;

        assert!(response.is_empty());
        mock.assert();

        Ok(())
    }
}

mod traded {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{Client, types::TradedRequest};
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

        assert_eq!(response.user.as_str(), test_user().as_str());
        assert_eq!(response.traded, 42);
        mock.assert();

        Ok(())
    }
}

mod open_interest {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{
        Client,
        types::{Hash64, OpenInterestRequest},
    };
    use reqwest::StatusCode;
    use serde_json::json;

    use super::test_condition_id;

    #[tokio::test]
    async fn open_interest_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let market2 =
            Hash64::new("0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc")?;

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
                }
            ]));
        });

        let response = client
            .open_interest(&OpenInterestRequest::default())
            .await?;

        assert_eq!(response.len(), 2);
        assert_eq!(response[0].market.as_str(), test_condition_id().as_str());
        assert!((response[0].value - 1_500_000.0).abs() < f64::EPSILON);
        assert_eq!(response[1].market.as_str(), market2.as_str());
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
        assert_eq!(response[0].market.as_str(), test_condition_id().as_str());
        mock.assert();

        Ok(())
    }
}

mod live_volume {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{
        Client,
        types::{EventId, Hash64, LiveVolumeRequest},
    };
    use reqwest::StatusCode;
    use serde_json::json;

    use super::test_condition_id;

    #[tokio::test]
    async fn live_volume_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let market2 =
            Hash64::new("0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd")?;

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

        let request = LiveVolumeRequest::builder().id(EventId::new(123)?).build();

        let response = client.live_volume(&request).await?;

        assert_eq!(response.len(), 1);
        assert!((response[0].total - 250_000.0).abs() < f64::EPSILON);
        let markets = &response[0].markets;
        assert_eq!(markets.len(), 2);
        assert_eq!(markets[0].market.as_str(), test_condition_id().as_str());
        assert!((markets[0].value - 150_000.0).abs() < f64::EPSILON);
        assert_eq!(markets[1].market.as_str(), market2.as_str());
        mock.assert();

        Ok(())
    }
}

mod builder_leaderboard {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{
        Client,
        types::{BuilderLeaderboardLimit, BuilderLeaderboardRequest, TimePeriod},
    };
    use reqwest::StatusCode;
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
        assert_eq!(response[0].rank, "1");
        assert_eq!(response[0].builder, "TopBuilder");
        assert!((response[0].volume - 5_000_000.0).abs() < f64::EPSILON);
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
            .limit(BuilderLeaderboardLimit::new(5)?)
            .build();

        let response = client.builder_leaderboard(&request).await?;

        assert!(response.is_empty());
        mock.assert();

        Ok(())
    }
}

mod builder_volume {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::data_api::{
        Client,
        types::{BuilderVolumeRequest, TimePeriod},
    };
    use reqwest::StatusCode;
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
        assert_eq!(response[0].dt, "2025-01-15T00:00:00Z");
        assert_eq!(response[0].builder, "Builder1");
        assert!((response[0].volume - 100_000.0).abs() < f64::EPSILON);
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
    use polymarket_client_sdk::data_api::{Client, types::PositionsRequest};
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
    use polymarket_client_sdk::data_api::Client;

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
    use polymarket_client_sdk::data_api::types::{
        ActivityRequest, ActivityType, Address, EventId, Hash64, HoldersLimit, LeaderboardCategory,
        LeaderboardOrderBy, LiveVolumeRequest, MarketFilter, PositionSortBy, PositionsLimit,
        PositionsRequest, QueryParams as _, Side, SortDirection, TimePeriod, Title, TradeFilter,
        TradedRequest, TraderLeaderboardLimit, TraderLeaderboardRequest, TradesRequest,
    };

    #[test]
    fn address_validation() {
        // Valid
        Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap();
        Address::new("0x56687BF447DB6FFA42FFE2204A05EDAA20F55839").unwrap(); // uppercase ok

        // Invalid
        Address::new("56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap_err(); // no 0x
        Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f5583").unwrap_err(); // too short
        Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f558399").unwrap_err(); // too long
        Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f5583Z").unwrap_err(); // invalid char
    }

    #[test]
    fn hash64_validation() {
        // Valid
        Hash64::new("0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917").unwrap();

        // Invalid
        Hash64::new("dd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917")
            .unwrap_err(); // no 0x
        Hash64::new("0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead11091")
            .unwrap_err(); // too short
    }

    #[test]
    fn event_id_validation() {
        EventId::new(1).unwrap();
        EventId::new(12345).unwrap();
        EventId::new(0).unwrap_err();
    }

    #[test]
    fn bounded_limits() {
        // PositionsLimit: 0-500
        PositionsLimit::new(0).unwrap();
        PositionsLimit::new(500).unwrap();
        PositionsLimit::new(501).unwrap_err();

        // HoldersLimit: 0-20
        HoldersLimit::new(0).unwrap();
        HoldersLimit::new(20).unwrap();
        HoldersLimit::new(21).unwrap_err();

        // TraderLeaderboardLimit: 1-50
        TraderLeaderboardLimit::new(0).unwrap_err();
        TraderLeaderboardLimit::new(1).unwrap();
        TraderLeaderboardLimit::new(50).unwrap();
        TraderLeaderboardLimit::new(51).unwrap_err();
    }

    #[test]
    fn title_validation() {
        Title::new("Short title").unwrap();
        Title::new("a".repeat(100)).unwrap();
        Title::new("a".repeat(101)).unwrap_err();
    }

    #[test]
    fn positions_request_query_params() {
        let req = PositionsRequest::builder()
            .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
            .limit(PositionsLimit::new(50).unwrap())
            .sort_by(PositionSortBy::CashPnl)
            .sort_direction(SortDirection::Desc)
            .build();

        let params = req.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "user" && v.starts_with("0x"))
        );
        assert!(params.iter().any(|(k, v)| *k == "limit" && v == "50"));
        assert!(params.iter().any(|(k, v)| *k == "sortBy" && v == "CASHPNL"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "sortDirection" && v == "DESC")
        );
    }

    #[test]
    fn market_filter_query_params() {
        let hash1 =
            Hash64::new("0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917")
                .unwrap();
        let hash2 =
            Hash64::new("0xaa22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917")
                .unwrap();

        let req = PositionsRequest::builder()
            .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
            .filter(MarketFilter::markets([hash1, hash2]))
            .build();

        let params = req.query_params();
        let market_param = params.iter().find(|(k, _)| *k == "market");
        assert!(market_param.is_some());
        let (_, value) = market_param.unwrap();
        assert!(value.contains(','));
        assert!(!params.iter().any(|(k, _)| *k == "eventId"));
    }

    #[test]
    fn event_id_filter_query_params() {
        let req = PositionsRequest::builder()
            .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
            .filter(MarketFilter::event_ids([
                EventId::new(1).unwrap(),
                EventId::new(2).unwrap(),
            ]))
            .build();

        let params = req.query_params();
        let event_param = params.iter().find(|(k, _)| *k == "eventId");
        assert!(event_param.is_some());
        let (_, value) = event_param.unwrap();
        assert_eq!(value, "1,2");
        assert!(!params.iter().any(|(k, _)| *k == "market"));
    }

    #[test]
    fn trade_filter() {
        TradeFilter::cash(100.0).unwrap();
        TradeFilter::tokens(0.0).unwrap();
        TradeFilter::cash(-1.0).unwrap_err();
    }

    #[test]
    fn trades_request_with_filter() {
        let req = TradesRequest::builder()
            .trade_filter(TradeFilter::cash(100.0).unwrap())
            .build();

        let params = req.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "filterType" && v == "CASH")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "filterAmount" && v == "100")
        );
    }

    #[test]
    fn activity_types_query_params() {
        let req = ActivityRequest::builder()
            .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
            .activity_types(vec![ActivityType::Trade, ActivityType::Redeem])
            .build();

        let params = req.query_params();
        let type_param = params.iter().find(|(k, _)| *k == "type");
        assert!(type_param.is_some());
        let (_, value) = type_param.unwrap();
        assert_eq!(value, "TRADE,REDEEM");
    }

    #[test]
    fn live_volume_request() {
        let req = LiveVolumeRequest::builder()
            .id(EventId::new(123).unwrap())
            .build();

        let params = req.query_params();
        assert_eq!(params.len(), 1);
        assert!(params.iter().any(|(k, v)| *k == "id" && v == "123"));
    }

    #[test]
    fn traded_request() {
        let req = TradedRequest::builder()
            .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
            .build();

        let params = req.query_params();
        assert_eq!(params.len(), 1);
        assert!(params.iter().any(|(k, _)| *k == "user"));
    }

    #[test]
    fn trader_leaderboard_request() {
        let req = TraderLeaderboardRequest::builder()
            .category(LeaderboardCategory::Politics)
            .time_period(TimePeriod::Week)
            .order_by(LeaderboardOrderBy::Pnl)
            .limit(TraderLeaderboardLimit::new(10).unwrap())
            .build();

        let params = req.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "category" && v == "POLITICS")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "timePeriod" && v == "WEEK")
        );
        assert!(params.iter().any(|(k, v)| *k == "orderBy" && v == "PNL"));
        assert!(params.iter().any(|(k, v)| *k == "limit" && v == "10"));
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
}
