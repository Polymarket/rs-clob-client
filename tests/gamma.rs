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
