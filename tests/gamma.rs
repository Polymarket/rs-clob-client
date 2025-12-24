#![allow(
    clippy::unwrap_used,
    reason = "Do not need additional syntax for setting up tests, and https://github.com/rust-lang/rust-clippy/issues/13981"
)]

//! Integration tests for the Gamma API client.
//!
//! These tests use `httpmock` to mock HTTP responses, ensuring deterministic
//! and fast test execution without requiring network access.
//!
//! # Running Tests
//!
//! ```bash
//! cargo test --features gamma
//! ```
//!
//! # Test Coverage
//!
//! Tests are organized by API endpoint group:
//! - `sports`: Teams, sports metadata, and market types
//! - `tags`: Tag listing and lookup by ID/slug, related tags
//! - `events`: Event listing and lookup by ID/slug, event tags
//! - `markets`: Market listing and lookup by ID/slug, market tags
//! - `series`: Series listing and lookup by ID
//! - `comments`: Comment listing and lookup by ID/user address
//! - `profiles`: Public profile lookup
//! - `search`: Search across events, markets, and profiles
//! - `health`: API health check

#![cfg(feature = "gamma")]

mod sports {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{Client, types::TeamsRequest};
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn teams_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/teams");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": 1,
                    "name": "Lakers",
                    "league": "NBA",
                    "record": "45-37",
                    "logo": "https://example.com/lakers.png",
                    "abbreviation": "LAL",
                    "alias": "Los Angeles Lakers",
                    "createdAt": "2024-01-15T10:30:00Z",
                    "updatedAt": "2024-06-20T14:45:00Z"
                },
                {
                    "id": 2,
                    "name": "Celtics",
                    "league": "NBA",
                    "record": "64-18",
                    "logo": "https://example.com/celtics.png",
                    "abbreviation": "BOS",
                    "alias": "Boston Celtics",
                    "createdAt": "2024-01-15T10:30:00Z",
                    "updatedAt": "2024-06-20T14:45:00Z"
                }
            ]));
        });

        let response = client.teams(&TeamsRequest::default()).await?;

        assert_eq!(response.len(), 2);
        assert_eq!(response[0].id, 1);
        assert_eq!(response[0].name, Some("Lakers".to_owned()));
        assert_eq!(response[0].league, Some("NBA".to_owned()));
        assert_eq!(response[1].id, 2);
        assert_eq!(response[1].name, Some("Celtics".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn sports_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/sports");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "sport": "ncaab",
                    "image": "https://example.com/basketball.png",
                    "resolution": "https://example.com",
                    "ordering": "home",
                    "tags": "1,2,3",
                    "series": "39"
                }
            ]));
        });

        let response = client.sports().await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].sport, "ncaab");
        assert_eq!(response[0].image, "https://example.com/basketball.png");
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn sports_market_types_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/sports/market-types");
            then.status(StatusCode::OK).json_body(json!({
                "marketTypes": ["moneyline", "spreads", "totals"]
            }));
        });

        let response = client.sports_market_types().await?;

        assert_eq!(
            response.market_types,
            vec!["moneyline", "spreads", "totals"]
        );
        mock.assert();

        Ok(())
    }
}

mod tags {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{
        Client,
        types::{
            RelatedTagsByIdRequest, RelatedTagsBySlugRequest, TagByIdRequest, TagBySlugRequest,
            TagsRequest,
        },
    };
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn tags_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/tags");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "1",
                    "label": "Politics",
                    "slug": "politics",
                    "forceShow": true,
                    "publishedAt": "2024-01-15T10:30:00Z",
                    "createdBy": 1,
                    "updatedBy": 2,
                    "createdAt": "2024-01-15T10:30:00Z",
                    "updatedAt": "2024-06-20T14:45:00Z",
                    "forceHide": false,
                    "isCarousel": true
                }
            ]));
        });

        let request = TagsRequest::builder().build();
        let response = client.tags(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "1");
        assert_eq!(response[0].label, Some("Politics".to_owned()));
        assert_eq!(response[0].slug, Some("politics".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn tag_by_id_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/tags/42");
            then.status(StatusCode::OK).json_body(json!({
                "id": "42",
                "label": "Sports",
                "slug": "sports",
                "forceShow": false,
                "forceHide": false,
                "isCarousel": false
            }));
        });

        let request = TagByIdRequest::builder().id(42_u32).build();
        let response = client.tag_by_id(&request).await?;

        assert_eq!(response.id, "42");
        assert_eq!(response.label, Some("Sports".to_owned()));
        assert_eq!(response.slug, Some("sports".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn tag_by_slug_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/tags/slug/crypto");
            then.status(StatusCode::OK).json_body(json!({
                "id": "99",
                "label": "Crypto",
                "slug": "crypto",
                "forceShow": true,
                "forceHide": false,
                "isCarousel": true
            }));
        });

        let request = TagBySlugRequest::builder().slug("crypto").build();
        let response = client.tag_by_slug(&request).await?;

        assert_eq!(response.id, "99");
        assert_eq!(response.label, Some("Crypto".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn related_tags_by_id_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/tags/42/related-tags");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "1",
                    "tagID": 42,
                    "relatedTagID": 99,
                    "rank": 1
                }
            ]));
        });

        let request = RelatedTagsByIdRequest::builder().id(42_u64).build();
        let response = client.related_tags_by_id(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "1");
        assert_eq!(response[0].tag_id, Some(42));
        assert_eq!(response[0].related_tag_id, Some(99));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn related_tags_by_slug_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/tags/slug/politics/related-tags");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "2",
                    "tagID": 10,
                    "relatedTagID": 20,
                    "rank": 5
                }
            ]));
        });

        let request = RelatedTagsBySlugRequest::builder().slug("politics").build();
        let response = client.related_tags_by_slug(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "2");
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn tags_related_to_tag_by_id_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/tags/42/related-tags/tags");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "99",
                    "label": "Related Tag",
                    "slug": "related-tag",
                    "forceShow": false,
                    "forceHide": false,
                    "isCarousel": false
                }
            ]));
        });

        let request = RelatedTagsByIdRequest::builder().id(42_u64).build();
        let response = client.tags_related_to_tag_by_id(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "99");
        assert_eq!(response[0].label, Some("Related Tag".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn tags_related_to_tag_by_slug_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/tags/slug/politics/related-tags/tags");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "50",
                    "label": "Elections",
                    "slug": "elections",
                    "forceShow": true,
                    "forceHide": false,
                    "isCarousel": true
                }
            ]));
        });

        let request = RelatedTagsBySlugRequest::builder().slug("politics").build();
        let response = client.tags_related_to_tag_by_slug(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "50");
        assert_eq!(response[0].label, Some("Elections".to_owned()));
        mock.assert();

        Ok(())
    }
}

mod events {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{
        Client,
        types::{EventByIdRequest, EventBySlugRequest, EventsRequest},
    };
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn events_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/events")
                .query_param("active", "true");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "123",
                    "title": "Test Event",
                    "slug": "test-event",
                    "active": true
                }
            ]));
        });

        let request = EventsRequest::builder().active(true).build();
        let response = client.events(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "123");
        assert_eq!(response[0].title, Some("Test Event".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn event_by_id_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/events/456");
            then.status(StatusCode::OK).json_body(json!({
                "id": "456",
                "title": "Specific Event",
                "slug": "specific-event"
            }));
        });

        let request = EventByIdRequest::builder().id("456").build();
        let response = client.event_by_id(&request).await?;

        assert_eq!(response.id, "456");
        assert_eq!(response.title, Some("Specific Event".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn event_by_slug_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/events/slug/my-event");
            then.status(StatusCode::OK).json_body(json!({
                "id": "789",
                "title": "My Event",
                "slug": "my-event"
            }));
        });

        let request = EventBySlugRequest::builder().slug("my-event").build();
        let response = client.event_by_slug(&request).await?;

        assert_eq!(response.id, "789");
        assert_eq!(response.slug, Some("my-event".to_owned()));
        mock.assert();

        Ok(())
    }
}

mod markets {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{
        Client,
        types::{MarketByIdRequest, MarketBySlugRequest, MarketsRequest},
    };
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn markets_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/markets").query_param("limit", "10");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "1",
                    "question": "Test Market?",
                    "slug": "test-market"
                }
            ]));
        });

        let request = MarketsRequest::builder().limit(10_u32).build();
        let response = client.markets(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "1");
        assert_eq!(response[0].question, Some("Test Market?".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn market_by_id_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/markets/42");
            then.status(StatusCode::OK).json_body(json!({
                "id": "42",
                "question": "Specific Market?",
                "slug": "specific-market"
            }));
        });

        let request = MarketByIdRequest::builder().id(42_u32).build();
        let response = client.market_by_id(&request).await?;

        assert_eq!(response.id, "42");
        assert_eq!(response.question, Some("Specific Market?".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn market_by_slug_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/markets/slug/my-market");
            then.status(StatusCode::OK).json_body(json!({
                "id": "99",
                "question": "My Market?",
                "slug": "my-market"
            }));
        });

        let request = MarketBySlugRequest::builder().slug("my-market").build();
        let response = client.market_by_slug(&request).await?;

        assert_eq!(response.id, "99");
        assert_eq!(response.slug, Some("my-market".to_owned()));
        mock.assert();

        Ok(())
    }
}

mod search {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{Client, types::SearchRequest};
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn search_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/public-search")
                .query_param("q", "bitcoin");
            then.status(StatusCode::OK).json_body(json!({
                "events": [],
                "tags": [],
                "profiles": []
            }));
        });

        let request = SearchRequest::builder().q("bitcoin").build();
        let response = client.search(&request).await?;

        assert!(
            response.events.is_none()
                || response
                    .events
                    .as_ref()
                    .is_some_and(std::vec::Vec::is_empty)
        );
        mock.assert();

        Ok(())
    }
}

mod health {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::Client;
    use reqwest::StatusCode;

    #[tokio::test]
    async fn status_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/status");
            then.status(StatusCode::OK).body("OK");
        });

        let response = client.status().await?;

        assert_eq!(response, "OK");
        mock.assert();

        Ok(())
    }
}

mod series {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{
        Client,
        types::{SeriesByIdRequest, SeriesListRequest},
    };
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn series_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/series");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "1",
                    "title": "Weekly Elections",
                    "slug": "weekly-elections",
                    "active": true,
                    "closed": false
                }
            ]));
        });

        let request = SeriesListRequest::builder().build();
        let response = client.series(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "1");
        assert_eq!(response[0].title, Some("Weekly Elections".to_owned()));
        assert_eq!(response[0].slug, Some("weekly-elections".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn series_by_id_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/series/42");
            then.status(StatusCode::OK).json_body(json!({
                "id": "42",
                "title": "NFL Season 2024",
                "slug": "nfl-season-2024",
                "active": true,
                "recurrence": "weekly"
            }));
        });

        let request = SeriesByIdRequest::builder().id(42_u32).build();
        let response = client.series_by_id(&request).await?;

        assert_eq!(response.id, "42");
        assert_eq!(response.title, Some("NFL Season 2024".to_owned()));
        assert_eq!(response.recurrence, Some("weekly".to_owned()));
        mock.assert();

        Ok(())
    }
}

mod comments {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{
        Client,
        types::{
            Address, CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest,
            ParentEntityType,
        },
    };
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn comments_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/comments");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "1",
                    "body": "Great market!",
                    "parentEntityType": "Event",
                    "parentEntityID": 123,
                    "userAddress": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
                    "createdAt": "2024-01-15T10:30:00Z"
                }
            ]));
        });

        let request = CommentsRequest::builder().build();
        let response = client.comments(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "1");
        assert_eq!(response[0].body, Some("Great market!".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn comments_with_filters_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/comments")
                .query_param("parent_entity_type", "Event")
                .query_param("parent_entity_id", "123")
                .query_param("limit", "10");
            then.status(StatusCode::OK).json_body(json!([]));
        });

        let request = CommentsRequest::builder()
            .parent_entity_type(ParentEntityType::Event)
            .parent_entity_id(123)
            .limit(10_u32)
            .build();
        let response = client.comments(&request).await?;

        assert!(response.is_empty());
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn comments_by_id_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/comments/42");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "42",
                    "body": "This is the comment",
                    "parentEntityType": "Event",
                    "parentEntityID": 100
                }
            ]));
        });

        let request = CommentsByIdRequest::builder().id(42).build();
        let response = client.comments_by_id(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "42");
        assert_eq!(response[0].body, Some("This is the comment".to_owned()));
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn comments_by_user_address_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/comments/user_address/0x56687bf447db6ffa42ffe2204a05edaa20f55839");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "1",
                    "body": "User comment",
                    "userAddress": "0x56687bf447db6ffa42ffe2204a05edaa20f55839"
                },
                {
                    "id": "2",
                    "body": "Another comment",
                    "userAddress": "0x56687bf447db6ffa42ffe2204a05edaa20f55839"
                }
            ]));
        });

        let request = CommentsByUserAddressRequest::builder()
            .user_address(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839")?)
            .build();
        let response = client.comments_by_user_address(&request).await?;

        assert_eq!(response.len(), 2);
        assert_eq!(response[0].body, Some("User comment".to_owned()));
        assert_eq!(response[1].body, Some("Another comment".to_owned()));
        mock.assert();

        Ok(())
    }
}

mod profiles {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{
        Client,
        types::{Address, PublicProfileRequest},
    };
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn public_profile_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/public-profile")
                .query_param("address", "0x56687bf447db6ffa42ffe2204a05edaa20f55839");
            then.status(StatusCode::OK).json_body(json!({
                "proxyWallet": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",
                "name": "Polymarket Trader",
                "pseudonym": "PolyTrader",
                "bio": "Trading prediction markets",
                "displayUsernamePublic": true,
                "verifiedBadge": false
            }));
        });

        let request = PublicProfileRequest::builder()
            .address(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839")?)
            .build();
        let response = client.public_profile(&request).await?;

        assert_eq!(response.name, Some("Polymarket Trader".to_owned()));
        assert_eq!(response.pseudonym, Some("PolyTrader".to_owned()));
        assert_eq!(response.verified_badge, Some(false));
        mock.assert();

        Ok(())
    }
}

mod event_tags {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{Client, types::EventTagsRequest};
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn event_tags_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/events/123/tags");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "1",
                    "label": "Politics",
                    "slug": "politics"
                },
                {
                    "id": "2",
                    "label": "Elections",
                    "slug": "elections"
                }
            ]));
        });

        let request = EventTagsRequest::builder().id(123_u32).build();
        let response = client.event_tags(&request).await?;

        assert_eq!(response.len(), 2);
        assert_eq!(response[0].id, "1");
        assert_eq!(response[0].label, Some("Politics".to_owned()));
        assert_eq!(response[1].id, "2");
        assert_eq!(response[1].label, Some("Elections".to_owned()));
        mock.assert();

        Ok(())
    }
}

mod market_tags {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{Client, types::MarketTagsRequest};
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn market_tags_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/markets/456/tags");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "3",
                    "label": "Crypto",
                    "slug": "crypto"
                }
            ]));
        });

        let request = MarketTagsRequest::builder().id(456_u32).build();
        let response = client.market_tags(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "3");
        assert_eq!(response[0].label, Some("Crypto".to_owned()));
        mock.assert();

        Ok(())
    }
}

// =============================================================================
// Unit Tests for QueryParams and Common Types
// =============================================================================

mod query_params {
    use chrono::{TimeZone as _, Utc};
    use polymarket_client_sdk::gamma::types::{
        Address, CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest,
        EventByIdRequest, EventBySlugRequest, EventTagsRequest, EventsRequest, MarketByIdRequest,
        MarketBySlugRequest, MarketTagsRequest, MarketsRequest, ParentEntityType,
        PublicProfileRequest, QueryParams as _, RelatedTagsByIdRequest, RelatedTagsBySlugRequest,
        RelatedTagsStatus, SearchRequest, SeriesByIdRequest, SeriesListRequest, TagByIdRequest,
        TagBySlugRequest, TagsRequest, TeamsRequest,
    };

    #[test]
    fn teams_request_all_params() {
        let request = TeamsRequest::builder()
            .limit(10_u32)
            .offset(5_u32)
            .order("name".to_owned())
            .ascending(true)
            .league(vec!["NBA".to_owned(), "NFL".to_owned()])
            .name(vec!["Lakers".to_owned()])
            .abbreviation(vec!["LAL".to_owned(), "BOS".to_owned()])
            .build();

        let params = request.query_params();
        assert!(params.iter().any(|(k, v)| *k == "limit" && v == "10"));
        assert!(params.iter().any(|(k, v)| *k == "offset" && v == "5"));
        assert!(params.iter().any(|(k, v)| *k == "order" && v == "name"));
        assert!(params.iter().any(|(k, v)| *k == "ascending" && v == "true"));
        assert!(params.iter().any(|(k, v)| *k == "league" && v == "NBA,NFL"));
        assert!(params.iter().any(|(k, v)| *k == "name" && v == "Lakers"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "abbreviation" && v == "LAL,BOS")
        );
    }

    #[test]
    fn teams_request_empty_arrays_not_included() {
        let request = TeamsRequest::builder()
            .league(vec![])
            .name(vec![])
            .abbreviation(vec![])
            .build();

        let params = request.query_params();
        assert!(!params.iter().any(|(k, _)| *k == "league"));
        assert!(!params.iter().any(|(k, _)| *k == "name"));
        assert!(!params.iter().any(|(k, _)| *k == "abbreviation"));
    }

    #[test]
    fn tags_request_all_params() {
        let request = TagsRequest::builder()
            .limit(20_u64)
            .offset(10_u64)
            .order("label".to_owned())
            .ascending(false)
            .include_template(true)
            .is_carousel(true)
            .build();

        let params = request.query_params();
        assert!(params.iter().any(|(k, v)| *k == "limit" && v == "20"));
        assert!(params.iter().any(|(k, v)| *k == "offset" && v == "10"));
        assert!(params.iter().any(|(k, v)| *k == "order" && v == "label"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "ascending" && v == "false")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_template" && v == "true")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "is_carousel" && v == "true")
        );
    }

    #[test]
    fn tag_by_id_request_with_include_template() {
        let request = TagByIdRequest::builder()
            .id(42_u32)
            .include_template(true)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_template" && v == "true")
        );
    }

    #[test]
    fn tag_by_slug_request_with_include_template() {
        let request = TagBySlugRequest::builder()
            .slug("politics")
            .include_template(false)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_template" && v == "false")
        );
    }

    #[test]
    fn related_tags_by_id_all_params() {
        let request = RelatedTagsByIdRequest::builder()
            .id(42_u64)
            .omit_empty(true)
            .status(RelatedTagsStatus::Active)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "omit_empty" && v == "true")
        );
        assert!(params.iter().any(|(k, v)| *k == "status" && v == "active"));
    }

    #[test]
    fn related_tags_by_slug_all_params() {
        let request = RelatedTagsBySlugRequest::builder()
            .slug("crypto")
            .omit_empty(false)
            .status(RelatedTagsStatus::Closed)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "omit_empty" && v == "false")
        );
        assert!(params.iter().any(|(k, v)| *k == "status" && v == "closed"));
    }

    #[test]
    fn related_tags_status_all() {
        let request = RelatedTagsByIdRequest::builder()
            .id(1_u64)
            .status(RelatedTagsStatus::All)
            .build();

        let params = request.query_params();
        assert!(params.iter().any(|(k, v)| *k == "status" && v == "all"));
    }

    #[test]
    fn events_request_all_params() {
        let start_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end_date = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let request = EventsRequest::builder()
            .limit(50_u32)
            .offset(10_u32)
            .order("startDate".to_owned())
            .ascending(true)
            .id(vec![1, 2, 3])
            .tag_id(42)
            .exclude_tag_id(vec![10, 20])
            .slug(vec!["event-1".to_owned(), "event-2".to_owned()])
            .tag_slug("politics".to_owned())
            .related_tags(true)
            .active(true)
            .archived(false)
            .featured(true)
            .cyom(false)
            .include_chat(true)
            .include_template(true)
            .recurrence("weekly".to_owned())
            .closed(false)
            .liquidity_min(1000.0)
            .liquidity_max(100_000.0)
            .volume_min(500.0)
            .volume_max(50000.0)
            .start_date_min(start_date)
            .start_date_max(end_date)
            .end_date_min(start_date)
            .end_date_max(end_date)
            .build();

        let params = request.query_params();
        assert!(params.iter().any(|(k, v)| *k == "limit" && v == "50"));
        assert!(params.iter().any(|(k, v)| *k == "offset" && v == "10"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "order" && v == "startDate")
        );
        assert!(params.iter().any(|(k, v)| *k == "ascending" && v == "true"));
        assert!(params.iter().any(|(k, v)| *k == "id" && v == "1,2,3"));
        assert!(params.iter().any(|(k, v)| *k == "tag_id" && v == "42"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "exclude_tag_id" && v == "10,20")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "slug" && v == "event-1,event-2")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "tag_slug" && v == "politics")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "related_tags" && v == "true")
        );
        assert!(params.iter().any(|(k, v)| *k == "active" && v == "true"));
        assert!(params.iter().any(|(k, v)| *k == "archived" && v == "false"));
        assert!(params.iter().any(|(k, v)| *k == "featured" && v == "true"));
        assert!(params.iter().any(|(k, v)| *k == "cyom" && v == "false"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_chat" && v == "true")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_template" && v == "true")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "recurrence" && v == "weekly")
        );
        assert!(params.iter().any(|(k, v)| *k == "closed" && v == "false"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "liquidity_min" && v == "1000")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "liquidity_max" && v == "100000")
        );
        assert!(params.iter().any(|(k, v)| *k == "volume_min" && v == "500"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "volume_max" && v == "50000")
        );
        assert!(params.iter().any(|(k, _)| *k == "start_date_min"));
        assert!(params.iter().any(|(k, _)| *k == "start_date_max"));
        assert!(params.iter().any(|(k, _)| *k == "end_date_min"));
        assert!(params.iter().any(|(k, _)| *k == "end_date_max"));
    }

    #[test]
    fn events_request_empty_arrays_not_included() {
        let request = EventsRequest::builder()
            .id(vec![])
            .exclude_tag_id(vec![])
            .slug(vec![])
            .build();

        let params = request.query_params();
        assert!(!params.iter().any(|(k, _)| *k == "id"));
        assert!(!params.iter().any(|(k, _)| *k == "exclude_tag_id"));
        assert!(!params.iter().any(|(k, _)| *k == "slug"));
    }

    #[test]
    fn event_by_id_request_all_params() {
        let request = EventByIdRequest::builder()
            .id("123")
            .include_chat(true)
            .include_template(false)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_chat" && v == "true")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_template" && v == "false")
        );
    }

    #[test]
    fn event_by_slug_request_all_params() {
        let request = EventBySlugRequest::builder()
            .slug("my-event")
            .include_chat(false)
            .include_template(true)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_chat" && v == "false")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_template" && v == "true")
        );
    }

    #[test]
    fn event_tags_request_empty_params() {
        let request = EventTagsRequest::builder().id(123_u32).build();
        let params = request.query_params();
        assert!(params.is_empty());
    }

    #[test]
    fn markets_request_all_params() {
        let start_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end_date = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let request = MarketsRequest::builder()
            .limit(100_u32)
            .offset(50_u32)
            .order("volume".to_owned())
            .ascending(false)
            .id(vec![1, 2])
            .slug(vec!["market-1".to_owned()])
            .clob_token_ids(vec!["token1".to_owned(), "token2".to_owned()])
            .condition_ids(vec!["cond1".to_owned()])
            .market_maker_address(vec!["0x123".to_owned()])
            .liquidity_num_min(1000.0)
            .liquidity_num_max(100_000.0)
            .volume_num_min(500.0)
            .volume_num_max(50000.0)
            .start_date_min(start_date)
            .start_date_max(end_date)
            .end_date_min(start_date)
            .end_date_max(end_date)
            .tag_id(42)
            .related_tags(true)
            .cyom(false)
            .uma_resolution_status("resolved".to_owned())
            .game_id("game123".to_owned())
            .sports_market_types(vec!["moneyline".to_owned(), "spread".to_owned()])
            .rewards_min_size(100.0)
            .question_ids(vec!["q1".to_owned(), "q2".to_owned()])
            .include_tag(true)
            .closed(false)
            .build();

        let params = request.query_params();
        assert!(params.iter().any(|(k, v)| *k == "limit" && v == "100"));
        assert!(params.iter().any(|(k, v)| *k == "offset" && v == "50"));
        assert!(params.iter().any(|(k, v)| *k == "order" && v == "volume"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "ascending" && v == "false")
        );
        assert!(params.iter().any(|(k, v)| *k == "id" && v == "1,2"));
        assert!(params.iter().any(|(k, v)| *k == "slug" && v == "market-1"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "clob_token_ids" && v == "token1,token2")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "condition_ids" && v == "cond1")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "market_maker_address" && v == "0x123")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "liquidity_num_min" && v == "1000")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "liquidity_num_max" && v == "100000")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "volume_num_min" && v == "500")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "volume_num_max" && v == "50000")
        );
        assert!(params.iter().any(|(k, _)| *k == "start_date_min"));
        assert!(params.iter().any(|(k, _)| *k == "start_date_max"));
        assert!(params.iter().any(|(k, _)| *k == "end_date_min"));
        assert!(params.iter().any(|(k, _)| *k == "end_date_max"));
        assert!(params.iter().any(|(k, v)| *k == "tag_id" && v == "42"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "related_tags" && v == "true")
        );
        assert!(params.iter().any(|(k, v)| *k == "cyom" && v == "false"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "uma_resolution_status" && v == "resolved")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "game_id" && v == "game123")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "sports_market_types" && v == "moneyline,spread")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "rewards_min_size" && v == "100")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "question_ids" && v == "q1,q2")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_tag" && v == "true")
        );
        assert!(params.iter().any(|(k, v)| *k == "closed" && v == "false"));
    }

    #[test]
    fn markets_request_empty_arrays_not_included() {
        let request = MarketsRequest::builder()
            .id(vec![])
            .slug(vec![])
            .clob_token_ids(vec![])
            .condition_ids(vec![])
            .market_maker_address(vec![])
            .sports_market_types(vec![])
            .question_ids(vec![])
            .build();

        let params = request.query_params();
        assert!(!params.iter().any(|(k, _)| *k == "id"));
        assert!(!params.iter().any(|(k, _)| *k == "slug"));
        assert!(!params.iter().any(|(k, _)| *k == "clob_token_ids"));
        assert!(!params.iter().any(|(k, _)| *k == "condition_ids"));
        assert!(!params.iter().any(|(k, _)| *k == "market_maker_address"));
        assert!(!params.iter().any(|(k, _)| *k == "sports_market_types"));
        assert!(!params.iter().any(|(k, _)| *k == "question_ids"));
    }

    #[test]
    fn market_by_id_request_with_include_tag() {
        let request = MarketByIdRequest::builder()
            .id(42_u32)
            .include_tag(true)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_tag" && v == "true")
        );
    }

    #[test]
    fn market_by_slug_request_with_include_tag() {
        let request = MarketBySlugRequest::builder()
            .slug("my-market")
            .include_tag(false)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_tag" && v == "false")
        );
    }

    #[test]
    fn market_tags_request_empty_params() {
        let request = MarketTagsRequest::builder().id(456_u32).build();
        let params = request.query_params();
        assert!(params.is_empty());
    }

    #[test]
    fn series_list_request_all_params() {
        let request = SeriesListRequest::builder()
            .limit(25_u32)
            .offset(5_u32)
            .order("title".to_owned())
            .ascending(true)
            .slug(vec!["series-1".to_owned(), "series-2".to_owned()])
            .categories_ids(vec![1, 2, 3])
            .categories_labels(vec!["Sports".to_owned(), "Politics".to_owned()])
            .closed(false)
            .include_chat(true)
            .recurrence("daily".to_owned())
            .build();

        let params = request.query_params();
        assert!(params.iter().any(|(k, v)| *k == "limit" && v == "25"));
        assert!(params.iter().any(|(k, v)| *k == "offset" && v == "5"));
        assert!(params.iter().any(|(k, v)| *k == "order" && v == "title"));
        assert!(params.iter().any(|(k, v)| *k == "ascending" && v == "true"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "slug" && v == "series-1,series-2")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "categories_ids" && v == "1,2,3")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "categories_labels" && v == "Sports,Politics")
        );
        assert!(params.iter().any(|(k, v)| *k == "closed" && v == "false"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_chat" && v == "true")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "recurrence" && v == "daily")
        );
    }

    #[test]
    fn series_list_request_empty_arrays_not_included() {
        let request = SeriesListRequest::builder()
            .slug(vec![])
            .categories_ids(vec![])
            .categories_labels(vec![])
            .build();

        let params = request.query_params();
        assert!(!params.iter().any(|(k, _)| *k == "slug"));
        assert!(!params.iter().any(|(k, _)| *k == "categories_ids"));
        assert!(!params.iter().any(|(k, _)| *k == "categories_labels"));
    }

    #[test]
    fn series_by_id_request_with_include_chat() {
        let request = SeriesByIdRequest::builder()
            .id(42_u32)
            .include_chat(true)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "include_chat" && v == "true")
        );
    }

    #[test]
    fn comments_request_all_params() {
        let request = CommentsRequest::builder()
            .limit(50_u32)
            .offset(10_u32)
            .order("createdAt".to_owned())
            .ascending(false)
            .parent_entity_type(ParentEntityType::Event)
            .parent_entity_id(123)
            .get_positions(true)
            .holders_only(true)
            .build();

        let params = request.query_params();
        assert!(params.iter().any(|(k, v)| *k == "limit" && v == "50"));
        assert!(params.iter().any(|(k, v)| *k == "offset" && v == "10"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "order" && v == "createdAt")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "ascending" && v == "false")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "parent_entity_type" && v == "Event")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "parent_entity_id" && v == "123")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "get_positions" && v == "true")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "holders_only" && v == "true")
        );
    }

    #[test]
    fn comments_request_series_entity_type() {
        let request = CommentsRequest::builder()
            .parent_entity_type(ParentEntityType::Series)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "parent_entity_type" && v == "Series")
        );
    }

    #[test]
    fn comments_request_market_entity_type() {
        let request = CommentsRequest::builder()
            .parent_entity_type(ParentEntityType::Market)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "parent_entity_type" && v == "market")
        );
    }

    #[test]
    fn comments_by_id_request_with_get_positions() {
        let request = CommentsByIdRequest::builder()
            .id(42)
            .get_positions(true)
            .build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "get_positions" && v == "true")
        );
    }

    #[test]
    fn comments_by_user_address_request_all_params() {
        let address = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap();
        let request = CommentsByUserAddressRequest::builder()
            .user_address(address)
            .limit(20_u32)
            .offset(5_u32)
            .order("createdAt".to_owned())
            .ascending(true)
            .build();

        let params = request.query_params();
        assert!(params.iter().any(|(k, v)| *k == "limit" && v == "20"));
        assert!(params.iter().any(|(k, v)| *k == "offset" && v == "5"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "order" && v == "createdAt")
        );
        assert!(params.iter().any(|(k, v)| *k == "ascending" && v == "true"));
    }

    #[test]
    fn public_profile_request_params() {
        let address = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap();
        let request = PublicProfileRequest::builder().address(address).build();

        let params = request.query_params();
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "address" && v == "0x56687bf447db6ffa42ffe2204a05edaa20f55839")
        );
    }

    #[test]
    fn search_request_all_params() {
        let request = SearchRequest::builder()
            .q("bitcoin")
            .cache(true)
            .events_status("active".to_owned())
            .limit_per_type(10)
            .page(2)
            .events_tag(vec!["crypto".to_owned(), "finance".to_owned()])
            .keep_closed_markets(5)
            .sort("volume".to_owned())
            .ascending(false)
            .search_tags(true)
            .search_profiles(true)
            .recurrence("weekly".to_owned())
            .exclude_tag_id(vec![1, 2])
            .optimized(true)
            .build();

        let params = request.query_params();
        assert!(params.iter().any(|(k, v)| *k == "q" && v == "bitcoin"));
        assert!(params.iter().any(|(k, v)| *k == "cache" && v == "true"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "events_status" && v == "active")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "limit_per_type" && v == "10")
        );
        assert!(params.iter().any(|(k, v)| *k == "page" && v == "2"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "events_tag" && v == "crypto,finance")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "keep_closed_markets" && v == "5")
        );
        assert!(params.iter().any(|(k, v)| *k == "sort" && v == "volume"));
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "ascending" && v == "false")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "search_tags" && v == "true")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "search_profiles" && v == "true")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "recurrence" && v == "weekly")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "exclude_tag_id" && v == "1,2")
        );
        assert!(params.iter().any(|(k, v)| *k == "optimized" && v == "true"));
    }

    #[test]
    fn search_request_empty_arrays_not_included() {
        let request = SearchRequest::builder()
            .q("test")
            .events_tag(vec![])
            .exclude_tag_id(vec![])
            .build();

        let params = request.query_params();
        assert!(!params.iter().any(|(k, _)| *k == "events_tag"));
        assert!(!params.iter().any(|(k, _)| *k == "exclude_tag_id"));
    }

    #[test]
    fn unit_query_params_returns_empty() {
        let params = ().query_params();
        assert!(params.is_empty());
    }
}

mod address_validation {
    use polymarket_client_sdk::gamma::types::Address;

    #[test]
    fn valid_address_lowercase() {
        let addr = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap();
        assert_eq!(addr.as_str(), "0x56687bf447db6ffa42ffe2204a05edaa20f55839");
    }

    #[test]
    fn valid_address_uppercase_converted_to_lowercase() {
        let addr = Address::new("0x56687BF447DB6FFA42FFE2204A05EDAA20F55839").unwrap();
        assert_eq!(addr.as_str(), "0x56687bf447db6ffa42ffe2204a05edaa20f55839");
    }

    #[test]
    fn valid_address_mixed_case_converted_to_lowercase() {
        let addr = Address::new("0x56687Bf447dB6fFa42Ffe2204A05EdaA20f55839").unwrap();
        assert_eq!(addr.as_str(), "0x56687bf447db6ffa42ffe2204a05edaa20f55839");
    }

    #[test]
    fn missing_prefix_error() {
        let result = Address::new("56687bf447db6ffa42ffe2204a05edaa20f55839");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "address must start with 0x");
    }

    #[test]
    fn invalid_length_too_short() {
        let result = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f5583");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "address must be 42 characters (got 41)");
    }

    #[test]
    fn invalid_length_too_long() {
        let result = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f558390");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "address must be 42 characters (got 43)");
    }

    #[test]
    fn invalid_hex_characters() {
        let result = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f5583g");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "address must contain only hex characters");
    }

    #[test]
    fn address_display() {
        let addr = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap();
        assert_eq!(
            format!("{addr}"),
            "0x56687bf447db6ffa42ffe2204a05edaa20f55839"
        );
    }

    #[test]
    fn address_into_string() {
        let addr = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap();
        let s: String = addr.into();
        assert_eq!(s, "0x56687bf447db6ffa42ffe2204a05edaa20f55839");
    }

    #[test]
    fn address_try_from_string() {
        Address::try_from("0x56687bf447db6ffa42ffe2204a05edaa20f55839".to_owned()).unwrap();
    }

    #[test]
    fn address_serde_roundtrip() {
        let addr = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap();
        let json = serde_json::to_string(&addr).unwrap();
        assert_eq!(json, "\"0x56687bf447db6ffa42ffe2204a05edaa20f55839\"");
        let parsed: Address = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, parsed);
    }

    #[test]
    fn address_serde_invalid_json() {
        let result: Result<Address, _> = serde_json::from_str("\"invalid\"");
        result.unwrap_err();
    }
}
