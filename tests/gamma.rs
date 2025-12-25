#![cfg(feature = "gamma")]

mod sports {
    use chrono::{DateTime, Utc};
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::types::{ListedTeam, Sport, SportsMarketTypesResponse};
    use polymarket_client_sdk::gamma::{
        Client,
        types::{ListTeamsRequest, ListTeamsResponse, SportsMetadataResponse},
    };
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

        let response = client.teams(&ListTeamsRequest::default()).await?;

        let expected: ListTeamsResponse = vec![
            ListedTeam::builder()
                .id(1)
                .name("Lakers")
                .league("NBA")
                .record("45-37")
                .logo("https://example.com/lakers.png")
                .abbreviation("LAL")
                .alias("Los Angeles Lakers")
                .created_at("2024-01-15T10:30:00Z".parse::<DateTime<Utc>>().unwrap())
                .updated_at("2024-06-20T14:45:00Z".parse::<DateTime<Utc>>().unwrap())
                .build(),
            ListedTeam::builder()
                .id(2)
                .name("Celtics")
                .league("NBA")
                .record("64-18")
                .logo("https://example.com/celtics.png")
                .abbreviation("BOS")
                .alias("Boston Celtics")
                .created_at("2024-01-15T10:30:00Z".parse::<DateTime<Utc>>().unwrap())
                .updated_at("2024-06-20T14:45:00Z".parse::<DateTime<Utc>>().unwrap())
                .build(),
        ];

        assert_eq!(response, expected);
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

        let expected: SportsMetadataResponse = vec![
            Sport::builder()
                .sport("ncaab")
                .image("https://example.com/basketball.png")
                .resolution("https://example.com")
                .ordering("home")
                .tags("1,2,3")
                .series("39")
                .build(),
        ];

        assert_eq!(response, expected);
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

        let expected = SportsMarketTypesResponse::builder()
            .market_types(vec![
                "moneyline".to_owned(),
                "spreads".to_owned(),
                "totals".to_owned(),
            ])
            .build();

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }
}

mod tags {
    use chrono::{DateTime, Utc};
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::types::{
        RelatedTagsByIdRequest, RelatedTagsBySlugRequest, Tag, TagRelationship,
    };
    use polymarket_client_sdk::gamma::{Client, types::TagsRequest};
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

        let response = client.tags(&TagsRequest::default()).await?;

        let expected = vec![
            Tag::builder()
                .id("1")
                .label("Politics")
                .slug("politics")
                .force_show(true)
                .published_at("2024-01-15T10:30:00Z")
                .created_by(1_i64)
                .updated_by(2_i64)
                .created_at("2024-01-15T10:30:00Z".parse::<DateTime<Utc>>().unwrap())
                .updated_at("2024-06-20T14:45:00Z".parse::<DateTime<Utc>>().unwrap())
                .force_hide(false)
                .is_carousel(true)
                .build(),
        ];

        assert_eq!(response, expected);
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

        let response = client.tag_by_id(42, None).await?;

        let expected = Tag::builder()
            .id("42")
            .label("Sports")
            .slug("sports")
            .force_show(false)
            .force_hide(false)
            .is_carousel(false)
            .build();

        assert_eq!(response, expected);
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

        let response = client.tag_by_slug("crypto", None).await?;

        let expected = Tag::builder()
            .id("99")
            .label("Crypto")
            .slug("crypto")
            .force_show(true)
            .force_hide(false)
            .is_carousel(true)
            .build();

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn tag_relationships_by_id_should_succeed() -> anyhow::Result<()> {
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
        let response = client.tag_relationships_by_id(&request).await?;

        let expected = vec![
            TagRelationship::builder()
                .id("1")
                .tag_id(42_i64)
                .related_tag_id(99_u64)
                .rank(1_u64)
                .build(),
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn tag_relationships_by_slug_should_succeed() -> anyhow::Result<()> {
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
        let response = client.tag_relationships_by_slug(&request).await?;

        let expected = vec![
            TagRelationship::builder()
                .id("2")
                .tag_id(10_i64)
                .related_tag_id(20_u64)
                .rank(5_u64)
                .build(),
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn related_tags_by_id_should_succeed() -> anyhow::Result<()> {
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
        let response = client.related_tags_by_id(&request).await?;

        let expected = vec![
            Tag::builder()
                .id("99")
                .label("Related Tag")
                .slug("related-tag")
                .force_show(false)
                .force_hide(false)
                .is_carousel(false)
                .build(),
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn related_tags_by_slug_should_succeed() -> anyhow::Result<()> {
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
        let response = client.related_tags_by_slug(&request).await?;

        let expected = vec![
            Tag::builder()
                .id("50")
                .label("Elections")
                .slug("elections")
                .force_show(true)
                .force_hide(false)
                .is_carousel(true)
                .build(),
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }
}
