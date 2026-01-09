#![cfg(feature = "gamma")]
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

mod sports {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{Client, types::request::TeamsRequest};
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
        types::request::{
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

        let request = TagByIdRequest::builder().id("42").build();
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
                    "tagID": "42",
                    "relatedTagID": "99",
                    "rank": 1
                }
            ]));
        });

        let request = RelatedTagsByIdRequest::builder().id("42").build();
        let response = client.related_tags_by_id(&request).await?;

        assert_eq!(response.len(), 1);
        assert_eq!(response[0].id, "1");
        assert_eq!(response[0].tag_id, Some("42".to_owned()));
        assert_eq!(response[0].related_tag_id, Some("99".to_owned()));
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
                    "tagID": "10",
                    "relatedTagID": "20",
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

        let request = RelatedTagsByIdRequest::builder().id("42").build();
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
        types::request::{EventByIdRequest, EventBySlugRequest, EventsRequest},
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
        types::request::{MarketByIdRequest, MarketBySlugRequest, MarketsRequest},
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

        let request = MarketsRequest::builder().limit(10).build();
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

        let request = MarketByIdRequest::builder().id("42").build();
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

    #[tokio::test]
    async fn markets_empty_request() -> anyhow::Result<()> {
        // Tests (true, true): no base params, no clob_token_ids
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/markets");
            then.status(StatusCode::OK).json_body(json!([]));
        });

        let request = MarketsRequest::default();
        let response = client.markets(&request).await?;

        assert!(response.is_empty());
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn markets_only_clob_token_ids() -> anyhow::Result<()> {
        // Tests (true, false): only clob_token_ids, no base params
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/markets")
                .query_param("clob_token_ids", "token1")
                .query_param("clob_token_ids", "token2");
            then.status(StatusCode::OK).json_body(json!([
                {"id": "1", "question": "Market 1?", "slug": "market-1"}
            ]));
        });

        let request = MarketsRequest::builder()
            .clob_token_ids(vec!["token1".to_owned(), "token2".to_owned()])
            .build();
        let response = client.markets(&request).await?;

        assert_eq!(response.len(), 1);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn markets_with_base_and_clob_params() -> anyhow::Result<()> {
        // Tests (false, false): both base params and clob_token_ids
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/markets")
                .query_param("limit", "50")
                .query_param("clob_token_ids", "abc")
                .query_param("clob_token_ids", "def");
            then.status(StatusCode::OK).json_body(json!([
                {"id": "1", "question": "Market 1?", "slug": "market-1"},
                {"id": "2", "question": "Market 2?", "slug": "market-2"}
            ]));
        });

        let request = MarketsRequest::builder()
            .limit(50)
            .clob_token_ids(vec!["abc".to_owned(), "def".to_owned()])
            .build();
        let response = client.markets(&request).await?;

        assert_eq!(response.len(), 2);
        mock.assert();

        Ok(())
    }
}

mod search {
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{Client, types::request::SearchRequest};
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
        types::request::{SeriesByIdRequest, SeriesListRequest},
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

        let request = SeriesByIdRequest::builder().id("42").build();
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
    use polymarket_client_sdk::gamma::types::ParentEntityType;
    use polymarket_client_sdk::gamma::{
        Client,
        types::request::{CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest},
    };
    use polymarket_client_sdk::types::address;
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

        let request = CommentsRequest::builder()
            .parent_entity_type(ParentEntityType::Event)
            .parent_entity_id("123")
            .build();
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
            .parent_entity_id("123")
            .limit(10)
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

        let request = CommentsByIdRequest::builder().id("42").build();
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
            // Address is serialized with EIP-55 checksum format
            when.method(GET)
                .path("/comments/user_address/0x56687BF447DB6fFA42FFE2204a05EDAA20f55839");
            then.status(StatusCode::OK).json_body(json!([
                {
                    "id": "1",
                    "body": "User comment",
                    "userAddress": "0x56687BF447DB6fFA42FFE2204a05EDAA20f55839"
                },
                {
                    "id": "2",
                    "body": "Another comment",
                    "userAddress": "0x56687BF447DB6fFA42FFE2204a05EDAA20f55839"
                }
            ]));
        });

        let request = CommentsByUserAddressRequest::builder()
            .user_address(address!("0x56687bf447db6ffa42ffe2204a05edaa20f55839"))
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
    use polymarket_client_sdk::gamma::{Client, types::request::PublicProfileRequest};
    use polymarket_client_sdk::types::address;
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn public_profile_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = Client::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            // Address serializes to lowercase hex via serde
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
            .address(address!("0x56687bf447db6ffa42ffe2204a05edaa20f55839"))
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
    use polymarket_client_sdk::gamma::{Client, types::request::EventTagsRequest};
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

        let request = EventTagsRequest::builder().id("123").build();
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
    use polymarket_client_sdk::gamma::{Client, types::request::MarketTagsRequest};
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

        let request = MarketTagsRequest::builder().id("456").build();
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

mod query_string {
    use chrono::{TimeZone as _, Utc};
    use polymarket_client_sdk::ToQueryParams as _;
    use polymarket_client_sdk::gamma::types::request::{
        CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest, EventByIdRequest,
        EventBySlugRequest, EventTagsRequest, EventsRequest, MarketByIdRequest,
        MarketBySlugRequest, MarketTagsRequest, MarketsRequest, PublicProfileRequest,
        RelatedTagsByIdRequest, RelatedTagsBySlugRequest, SearchRequest, SeriesByIdRequest,
        SeriesListRequest, TagByIdRequest, TagBySlugRequest, TagsRequest, TeamsRequest,
    };
    use polymarket_client_sdk::gamma::types::{ParentEntityType, RelatedTagsStatus};
    use polymarket_client_sdk::types::{address, b256};
    use rust_decimal_macros::dec;

    #[test]
    fn teams_request_all_params() {
        let request = TeamsRequest::builder()
            .limit(10)
            .offset(5)
            .order("name".to_owned())
            .ascending(true)
            .league(vec!["NBA".to_owned(), "NFL".to_owned()])
            .name(vec!["Lakers".to_owned()])
            .abbreviation(vec!["LAL".to_owned(), "BOS".to_owned()])
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("limit=10"));
        assert!(qs.contains("offset=5"));
        assert!(qs.contains("order=name"));
        assert!(qs.contains("ascending=true"));
        // Arrays should be repeated params, not comma-separated
        assert!(qs.contains("league=NBA"));
        assert!(qs.contains("league=NFL"));
        assert!(qs.contains("name=Lakers"));
        assert!(qs.contains("abbreviation=LAL"));
        assert!(qs.contains("abbreviation=BOS"));
    }

    #[test]
    fn teams_request_empty_arrays_not_included() {
        let request = TeamsRequest::builder()
            .league(vec![])
            .name(vec![])
            .abbreviation(vec![])
            .build();

        let qs = request.query_params(None);
        assert!(!qs.contains("league="));
        assert!(!qs.contains("name="));
        assert!(!qs.contains("abbreviation="));
    }

    #[test]
    fn tags_request_all_params() {
        let request = TagsRequest::builder()
            .limit(20)
            .offset(10)
            .order("label".to_owned())
            .ascending(false)
            .include_template(true)
            .is_carousel(true)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("limit=20"));
        assert!(qs.contains("offset=10"));
        assert!(qs.contains("order=label"));
        assert!(qs.contains("ascending=false"));
        assert!(qs.contains("include_template=true"));
        assert!(qs.contains("is_carousel=true"));
    }

    #[test]
    fn tag_by_id_request_with_include_template() {
        let request = TagByIdRequest::builder()
            .id("42")
            .include_template(true)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("include_template=true"));
    }

    #[test]
    fn tag_by_slug_request_with_include_template() {
        let request = TagBySlugRequest::builder()
            .slug("politics")
            .include_template(false)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("include_template=false"));
    }

    #[test]
    fn related_tags_by_id_all_params() {
        let request = RelatedTagsByIdRequest::builder()
            .id("42")
            .omit_empty(true)
            .status(RelatedTagsStatus::Active)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("omit_empty=true"));
        assert!(qs.contains("status=active"));
    }

    #[test]
    fn related_tags_by_slug_all_params() {
        let request = RelatedTagsBySlugRequest::builder()
            .slug("crypto")
            .omit_empty(false)
            .status(RelatedTagsStatus::Closed)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("omit_empty=false"));
        assert!(qs.contains("status=closed"));
    }

    #[test]
    fn related_tags_status_all() {
        let request = RelatedTagsByIdRequest::builder()
            .id("1")
            .status(RelatedTagsStatus::All)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("status=all"));
    }

    #[test]
    fn events_request_all_params() {
        let start_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end_date = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let request = EventsRequest::builder()
            .limit(50)
            .offset(10)
            .order(vec!["startDate".to_owned()])
            .ascending(true)
            .id(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])
            .tag_id("42")
            .exclude_tag_id(vec!["10".to_owned(), "20".to_owned()])
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
            .liquidity_min(dec!(1000))
            .liquidity_max(dec!(100_000))
            .volume_min(dec!(500))
            .volume_max(dec!(50000))
            .start_date_min(start_date)
            .start_date_max(end_date)
            .end_date_min(start_date)
            .end_date_max(end_date)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("limit=50"));
        assert!(qs.contains("offset=10"));
        assert!(qs.contains("order=startDate"));
        assert!(qs.contains("ascending=true"));
        // Arrays should be repeated params, not comma-separated
        assert!(qs.contains("id=1"));
        assert!(qs.contains("id=2"));
        assert!(qs.contains("id=3"));
        assert!(qs.contains("tag_id=42"));
        assert!(qs.contains("exclude_tag_id=10"));
        assert!(qs.contains("exclude_tag_id=20"));
        assert!(qs.contains("slug=event-1"));
        assert!(qs.contains("slug=event-2"));
        assert!(qs.contains("tag_slug=politics"));
        assert!(qs.contains("related_tags=true"));
        assert!(qs.contains("active=true"));
        assert!(qs.contains("archived=false"));
        assert!(qs.contains("featured=true"));
        assert!(qs.contains("cyom=false"));
        assert!(qs.contains("include_chat=true"));
        assert!(qs.contains("include_template=true"));
        assert!(qs.contains("recurrence=weekly"));
        assert!(qs.contains("closed=false"));
        assert!(qs.contains("liquidity_min=1000"));
        assert!(qs.contains("liquidity_max=100000"));
        assert!(qs.contains("volume_min=500"));
        assert!(qs.contains("volume_max=50000"));
        assert!(qs.contains("start_date_min="));
        assert!(qs.contains("start_date_max="));
        assert!(qs.contains("end_date_min="));
        assert!(qs.contains("end_date_max="));
    }

    #[test]
    fn events_request_empty_arrays_not_included() {
        let request = EventsRequest::builder()
            .id(vec![])
            .exclude_tag_id(vec![])
            .slug(vec![])
            .build();

        let qs = request.query_params(None);
        assert!(!qs.contains("id="));
        assert!(!qs.contains("exclude_tag_id="));
        assert!(!qs.contains("slug="));
    }

    #[test]
    fn event_by_id_request_all_params() {
        let request = EventByIdRequest::builder()
            .id("123")
            .include_chat(true)
            .include_template(false)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("include_chat=true"));
        assert!(qs.contains("include_template=false"));
    }

    #[test]
    fn event_by_slug_request_all_params() {
        let request = EventBySlugRequest::builder()
            .slug("my-event")
            .include_chat(false)
            .include_template(true)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("include_chat=false"));
        assert!(qs.contains("include_template=true"));
    }

    #[test]
    fn event_tags_request_empty_params() {
        let request = EventTagsRequest::builder().id("123").build();
        let qs = request.query_params(None);
        assert!(qs.is_empty());
    }

    #[test]
    fn markets_request_all_params() {
        let start_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end_date = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let request = MarketsRequest::builder()
            .limit(100)
            .offset(50)
            .order("volume".to_owned())
            .ascending(false)
            .id(vec!["1".to_owned(), "2".to_owned()])
            .slug(vec!["market-1".to_owned()])
            .clob_token_ids(vec!["token1".to_owned(), "token2".to_owned()])
            .condition_ids(vec![b256!(
                "0x0000000000000000000000000000000000000000000000000000000000000001"
            )])
            .market_maker_address(vec![address!("0x0000000000000000000000000000000000000123")])
            .liquidity_num_min(dec!(1000))
            .liquidity_num_max(dec!(100_000))
            .volume_num_min(dec!(500))
            .volume_num_max(dec!(50000))
            .start_date_min(start_date)
            .start_date_max(end_date)
            .end_date_min(start_date)
            .end_date_max(end_date)
            .tag_id("42")
            .related_tags(true)
            .cyom(false)
            .uma_resolution_status("resolved".to_owned())
            .game_id("game123".to_owned())
            .sports_market_types(vec!["moneyline".to_owned(), "spread".to_owned()])
            .rewards_min_size(dec!(100))
            .question_ids(vec![
                b256!("0x0000000000000000000000000000000000000000000000000000000000000001"),
                b256!("0x0000000000000000000000000000000000000000000000000000000000000002"),
            ])
            .include_tag(true)
            .closed(false)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("limit=100"));
        assert!(qs.contains("offset=50"));
        assert!(qs.contains("order=volume"));
        assert!(qs.contains("ascending=false"));
        // Arrays should be repeated params, not comma-separated
        assert!(qs.contains("id=1"));
        assert!(qs.contains("id=2"));
        assert!(qs.contains("slug=market-1"));
        // clob_token_ids is now handled with repeated params like all other arrays
        assert!(qs.contains("clob_token_ids=token1"));
        assert!(qs.contains("clob_token_ids=token2"));
        // B256 and Address serialize to lowercase hex via serde (repeated params)
        assert!(qs.contains(
            "condition_ids=0x0000000000000000000000000000000000000000000000000000000000000001"
        ));
        assert!(qs.contains("market_maker_address=0x0000000000000000000000000000000000000123"));
        assert!(qs.contains("liquidity_num_min=1000"));
        assert!(qs.contains("liquidity_num_max=100000"));
        assert!(qs.contains("volume_num_min=500"));
        assert!(qs.contains("volume_num_max=50000"));
        assert!(qs.contains("start_date_min="));
        assert!(qs.contains("start_date_max="));
        assert!(qs.contains("end_date_min="));
        assert!(qs.contains("end_date_max="));
        assert!(qs.contains("tag_id=42"));
        assert!(qs.contains("related_tags=true"));
        assert!(qs.contains("cyom=false"));
        assert!(qs.contains("uma_resolution_status=resolved"));
        assert!(qs.contains("game_id=game123"));
        assert!(qs.contains("sports_market_types=moneyline"));
        assert!(qs.contains("sports_market_types=spread"));
        assert!(qs.contains("rewards_min_size=100"));
        // B256 question_ids serialize to lowercase hex via serde (repeated params)
        assert!(qs.contains(
            "question_ids=0x0000000000000000000000000000000000000000000000000000000000000001"
        ));
        assert!(qs.contains(
            "question_ids=0x0000000000000000000000000000000000000000000000000000000000000002"
        ));
        assert!(qs.contains("include_tag=true"));
        assert!(qs.contains("closed=false"));
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

        let qs = request.query_params(None);
        assert!(!qs.contains("id="));
        assert!(!qs.contains("slug="));
        assert!(!qs.contains("clob_token_ids="));
        assert!(!qs.contains("condition_ids="));
        assert!(!qs.contains("market_maker_address="));
        assert!(!qs.contains("sports_market_types="));
        assert!(!qs.contains("question_ids="));
    }

    #[test]
    fn market_by_id_request_with_include_tag() {
        let request = MarketByIdRequest::builder()
            .id("42")
            .include_tag(true)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("include_tag=true"));
    }

    #[test]
    fn market_by_slug_request_with_include_tag() {
        let request = MarketBySlugRequest::builder()
            .slug("my-market")
            .include_tag(false)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("include_tag=false"));
    }

    #[test]
    fn market_tags_request_empty_params() {
        let request = MarketTagsRequest::builder().id("456").build();
        let qs = request.query_params(None);
        assert!(qs.is_empty());
    }

    #[test]
    fn series_list_request_all_params() {
        let request = SeriesListRequest::builder()
            .limit(25)
            .offset(5)
            .order("title".to_owned())
            .ascending(true)
            .slug(vec!["series-1".to_owned(), "series-2".to_owned()])
            .categories_ids(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()])
            .categories_labels(vec!["Sports".to_owned(), "Politics".to_owned()])
            .closed(false)
            .include_chat(true)
            .recurrence("daily".to_owned())
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("limit=25"));
        assert!(qs.contains("offset=5"));
        assert!(qs.contains("order=title"));
        assert!(qs.contains("ascending=true"));
        // Arrays should be repeated params, not comma-separated
        assert!(qs.contains("slug=series-1"));
        assert!(qs.contains("slug=series-2"));
        assert!(qs.contains("categories_ids=1"));
        assert!(qs.contains("categories_ids=2"));
        assert!(qs.contains("categories_ids=3"));
        assert!(qs.contains("categories_labels=Sports"));
        assert!(qs.contains("categories_labels=Politics"));
        assert!(qs.contains("closed=false"));
        assert!(qs.contains("include_chat=true"));
        assert!(qs.contains("recurrence=daily"));
    }

    #[test]
    fn series_list_request_empty_arrays_not_included() {
        let request = SeriesListRequest::builder()
            .slug(vec![])
            .categories_ids(vec![])
            .categories_labels(vec![])
            .build();

        let qs = request.query_params(None);
        assert!(!qs.contains("slug="));
        assert!(!qs.contains("categories_ids="));
        assert!(!qs.contains("categories_labels="));
    }

    #[test]
    fn series_by_id_request_with_include_chat() {
        let request = SeriesByIdRequest::builder()
            .id("42")
            .include_chat(true)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("include_chat=true"));
    }

    #[test]
    fn comments_request_all_params() {
        let request = CommentsRequest::builder()
            .limit(50)
            .offset(10)
            .order("createdAt".to_owned())
            .ascending(false)
            .parent_entity_type(ParentEntityType::Event)
            .parent_entity_id("123")
            .get_positions(true)
            .holders_only(true)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("limit=50"));
        assert!(qs.contains("offset=10"));
        assert!(qs.contains("order=createdAt"));
        assert!(qs.contains("ascending=false"));
        assert!(qs.contains("parent_entity_type=Event"));
        assert!(qs.contains("parent_entity_id=123"));
        assert!(qs.contains("get_positions=true"));
        assert!(qs.contains("holders_only=true"));
    }

    #[test]
    fn comments_request_series_entity_type() {
        let request = CommentsRequest::builder()
            .parent_entity_type(ParentEntityType::Series)
            .parent_entity_id("series-123")
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("parent_entity_type=Series"));
        assert!(qs.contains("parent_entity_id=series-123"));
    }

    #[test]
    fn comments_request_market_entity_type() {
        let request = CommentsRequest::builder()
            .parent_entity_type(ParentEntityType::Market)
            .parent_entity_id("market-456")
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("parent_entity_type=market"));
        assert!(qs.contains("parent_entity_id=market-456"));
    }

    #[test]
    fn comments_by_id_request_with_get_positions() {
        let request = CommentsByIdRequest::builder()
            .id("42")
            .get_positions(true)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("get_positions=true"));
    }

    #[test]
    fn comments_by_user_address_request_all_params() {
        let request = CommentsByUserAddressRequest::builder()
            .user_address(address!("0x56687bf447db6ffa42ffe2204a05edaa20f55839"))
            .limit(20)
            .offset(5)
            .order("createdAt".to_owned())
            .ascending(true)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("limit=20"));
        assert!(qs.contains("offset=5"));
        assert!(qs.contains("order=createdAt"));
        assert!(qs.contains("ascending=true"));
    }

    #[test]
    fn public_profile_request_params() {
        let request = PublicProfileRequest::builder()
            .address(address!("0x56687bf447db6ffa42ffe2204a05edaa20f55839"))
            .build();

        let qs = request.query_params(None);
        // Address serializes to lowercase hex via serde
        assert!(qs.contains("address=0x56687bf447db6ffa42ffe2204a05edaa20f55839"));
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
            .exclude_tag_id(vec!["1".to_owned(), "2".to_owned()])
            .optimized(true)
            .build();

        let qs = request.query_params(None);
        assert!(qs.contains("q=bitcoin"));
        assert!(qs.contains("cache=true"));
        assert!(qs.contains("events_status=active"));
        assert!(qs.contains("limit_per_type=10"));
        assert!(qs.contains("page=2"));
        // Arrays should be repeated params, not comma-separated
        assert!(qs.contains("events_tag=crypto"));
        assert!(qs.contains("events_tag=finance"));
        assert!(qs.contains("keep_closed_markets=5"));
        assert!(qs.contains("sort=volume"));
        assert!(qs.contains("ascending=false"));
        assert!(qs.contains("search_tags=true"));
        assert!(qs.contains("search_profiles=true"));
        assert!(qs.contains("recurrence=weekly"));
        assert!(qs.contains("exclude_tag_id=1"));
        assert!(qs.contains("exclude_tag_id=2"));
        assert!(qs.contains("optimized=true"));
    }

    #[test]
    fn search_request_empty_arrays_not_included() {
        let request = SearchRequest::builder()
            .q("test")
            .events_tag(vec![])
            .exclude_tag_id(vec![])
            .build();

        let qs = request.query_params(None);
        assert!(!qs.contains("events_tag="));
        assert!(!qs.contains("exclude_tag_id="));
    }

    #[test]
    fn unit_query_string_returns_empty() {
        let qs = ().query_params(None);
        assert!(qs.is_empty());
    }
}
