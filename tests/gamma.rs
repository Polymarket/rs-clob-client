mod sports {
    use chrono::{DateTime, Utc};
    use httpmock::{Method::GET, MockServer};
    use polymarket_client_sdk::gamma::{
        GammaClient,
        types::{
            ListTeamsRequest, ListTeamsResponse, ListedTeamBuilder, SportBuilder,
            SportsMarketTypesResponseBuilder, SportsMetadataResponse,
        },
    };
    use reqwest::StatusCode;
    use serde_json::json;

    #[tokio::test]
    async fn teams_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = GammaClient::new(&server.base_url())?;

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
            ListedTeamBuilder::default()
                .id(1u32)
                .name("Lakers")
                .league("NBA")
                .record("45-37")
                .logo("https://example.com/lakers.png")
                .abbreviation("LAL")
                .alias("Los Angeles Lakers")
                .created_at("2024-01-15T10:30:00Z".parse::<DateTime<Utc>>().unwrap())
                .updated_at("2024-06-20T14:45:00Z".parse::<DateTime<Utc>>().unwrap())
                .build()?,
            ListedTeamBuilder::default()
                .id(2u32)
                .name("Celtics")
                .league("NBA")
                .record("64-18")
                .logo("https://example.com/celtics.png")
                .abbreviation("BOS")
                .alias("Boston Celtics")
                .created_at("2024-01-15T10:30:00Z".parse::<DateTime<Utc>>().unwrap())
                .updated_at("2024-06-20T14:45:00Z".parse::<DateTime<Utc>>().unwrap())
                .build()?,
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn sports_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = GammaClient::new(&server.base_url())?;

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
            SportBuilder::default()
                .sport("ncaab")
                .image("https://example.com/basketball.png")
                .resolution("https://example.com")
                .ordering("home")
                .tags("1,2,3")
                .series("39")
                .build()?,
        ];

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn sports_market_types_should_succeed() -> anyhow::Result<()> {
        let server = MockServer::start();
        let client = GammaClient::new(&server.base_url())?;

        let mock = server.mock(|when, then| {
            when.method(GET).path("/sports/market-types");
            then.status(StatusCode::OK).json_body(json!({
                "marketTypes": ["moneyline", "spreads", "totals"]
            }));
        });

        let response = client.sports_market_types().await?;

        let expected = SportsMarketTypesResponseBuilder::default()
            .market_types(vec![
                "moneyline".to_string(),
                "spreads".to_string(),
                "totals".to_string(),
            ])
            .build()?;

        assert_eq!(response, expected);
        mock.assert();

        Ok(())
    }
}
