//! Client for the Polymarket Gamma API.
//!
//! This module provides an HTTP client for interacting with the Polymarket Gamma API,
//! which offers endpoints for querying events, markets, tags, series, comments, and more.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_client_sdk::gamma::{Client, types::request::EventsRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::default();
//!
//! // List active events
//! let request = EventsRequest::builder()
//!     .active(true)
//!     .limit(10)
//!     .build();
//!
//! let events = client.events(&request).await?;
//! for event in events {
//!     println!("{}: {:?}", event.id, event.title);
//! }
//! # Ok(())
//! # }
//! ```

use reqwest::{
    Client as ReqwestClient, Method,
    header::{HeaderMap, HeaderValue},
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use super::types::request::{
    CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest, EventByIdRequest,
    EventBySlugRequest, EventTagsRequest, EventsRequest, MarketByIdRequest, MarketBySlugRequest,
    MarketTagsRequest, MarketsRequest, PublicProfileRequest, RelatedTagsByIdRequest,
    RelatedTagsBySlugRequest, SearchRequest, SeriesByIdRequest, SeriesListRequest, TagByIdRequest,
    TagBySlugRequest, TagsRequest, TeamsRequest,
};
use super::types::response::{
    Comment, Event, HealthResponse, Market, PublicProfile, RelatedTag, SearchResults, Series,
    SportsMarketTypesResponse, SportsMetadata, Tag, Team,
};
use crate::error::Error;
use crate::{Result, ToQueryParams as _};

/// HTTP client for the Polymarket Gamma API.
///
/// Provides methods for querying events, markets, tags, series, comments,
/// profiles, and search functionality.
///
/// # API Base URL
///
/// The default API endpoint is `https://gamma-api.polymarket.com`.
///
/// # Example
///
/// ```no_run
/// use polymarket_client_sdk::gamma::Client;
///
/// // Create client with default endpoint
/// let client = Client::default();
///
/// // Or with a custom endpoint
/// let client = Client::new("https://custom-api.example.com").unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct Client {
    host: Url,
    client: ReqwestClient,
}

impl Default for Client {
    fn default() -> Self {
        Client::new("https://gamma-api.polymarket.com")
            .expect("Client with default endpoint should succeed")
    }
}

impl Client {
    /// Creates a new Gamma API client with a custom host URL.
    ///
    /// # Arguments
    ///
    /// * `host` - The base URL for the Gamma API.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid or the HTTP client cannot be created.
    pub fn new(host: &str) -> Result<Client> {
        let mut headers = HeaderMap::new();

        headers.insert("User-Agent", HeaderValue::from_static("rs_clob_client"));
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        let client = ReqwestClient::builder().default_headers(headers).build()?;

        Ok(Self {
            host: Url::parse(host)?,
            client,
        })
    }

    /// Returns the base URL of the API.
    #[must_use]
    pub fn host(&self) -> &Url {
        &self.host
    }

    async fn get<Req: Serialize, Res: DeserializeOwned + Serialize>(
        &self,
        path: &str,
        req: &Req,
    ) -> Result<Res> {
        let query = req.query_params(None);
        let request = self
            .client
            .request(Method::GET, format!("{}{path}{query}", self.host))
            .build()?;
        crate::request(&self.client, request, None).await
    }

    /// Performs a health check on the Gamma API.
    ///
    /// Returns "OK" when the API is healthy and operational. Use this for monitoring
    /// and verifying the API's availability.
    ///
    /// # Errors
    ///
    /// Returns an error if the API is unreachable or returns a non-200 status code.
    pub async fn status(&self) -> Result<HealthResponse> {
        let request = self
            .client
            .request(Method::GET, format!("{}status", self.host))
            .build()?;

        let response = self.client.execute(request).await?;
        let status_code = response.status();

        if !status_code.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(Error::status(
                status_code,
                Method::GET,
                "status".to_owned(),
                message,
            ));
        }

        Ok(response.text().await?)
    }

    /// Retrieves a list of sports teams with optional filtering.
    ///
    /// Returns teams participating in sports markets. Use filters to narrow results
    /// by sport type, league, or other criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn teams(&self, request: &TeamsRequest) -> Result<Vec<Team>> {
        self.get("teams", request).await
    }

    /// Retrieves metadata for all supported sports.
    ///
    /// Returns information about sports categories available on Polymarket,
    /// including sports like NFL, NBA, MLB, etc. Useful for discovering
    /// what sports markets are available.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn sports(&self) -> Result<Vec<SportsMetadata>> {
        self.get("sports", &()).await
    }

    /// Retrieves valid market types for sports.
    ///
    /// Returns the different types of sports markets available (e.g., moneyline,
    /// spread, over/under). Use this to understand what formats are supported.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn sports_market_types(&self) -> Result<SportsMarketTypesResponse> {
        self.get("sports/market-types", &()).await
    }

    /// Retrieves a list of tags with optional filtering.
    ///
    /// Tags categorize markets and events (e.g., "Politics", "Crypto", "Sports").
    /// Use filters to search for specific tag types or categories.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn tags(&self, request: &TagsRequest) -> Result<Vec<Tag>> {
        self.get("tags", request).await
    }

    /// Retrieves a single tag by its unique ID.
    ///
    /// Returns detailed information about a specific tag including its name,
    /// description, and associated markets.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag ID is invalid or the request fails.
    pub async fn tag_by_id(&self, request: &TagByIdRequest) -> Result<Tag> {
        self.get(&format!("tags/{}", request.id), request).await
    }

    /// Retrieves a single tag by its URL-friendly slug.
    ///
    /// Returns the same information as [`Self::tag_by_id`] but uses a human-readable
    /// slug identifier instead of a numeric ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the slug is invalid or the request fails.
    pub async fn tag_by_slug(&self, request: &TagBySlugRequest) -> Result<Tag> {
        self.get(&format!("tags/slug/{}", request.slug), request)
            .await
    }

    /// Retrieves related tag relationships for a tag by ID.
    ///
    /// Returns tags that are semantically related to the specified tag, including
    /// the relationship type (e.g., parent, child, related). Useful for discovering
    /// related markets and topics.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag ID is invalid or the request fails.
    pub async fn related_tags_by_id(
        &self,
        request: &RelatedTagsByIdRequest,
    ) -> Result<Vec<RelatedTag>> {
        self.get(&format!("tags/{}/related-tags", request.id), request)
            .await
    }

    /// Retrieves related tag relationships for a tag by slug.
    ///
    /// Same as [`Self::related_tags_by_id`] but uses a slug identifier instead of an ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the slug is invalid or the request fails.
    pub async fn related_tags_by_slug(
        &self,
        request: &RelatedTagsBySlugRequest,
    ) -> Result<Vec<RelatedTag>> {
        self.get(&format!("tags/slug/{}/related-tags", request.slug), request)
            .await
    }

    /// Retrieves tags that are related to a specified tag by ID.
    ///
    /// Returns the actual tag objects (not just relationships) for tags related to
    /// the specified tag. This provides full tag details for related topics.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag ID is invalid or the request fails.
    pub async fn tags_related_to_tag_by_id(
        &self,
        request: &RelatedTagsByIdRequest,
    ) -> Result<Vec<Tag>> {
        self.get(&format!("tags/{}/related-tags/tags", request.id), request)
            .await
    }

    /// Retrieves tags that are related to a specified tag by slug.
    ///
    /// Same as [`Self::tags_related_to_tag_by_id`] but uses a slug identifier instead of an ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the slug is invalid or the request fails.
    pub async fn tags_related_to_tag_by_slug(
        &self,
        request: &RelatedTagsBySlugRequest,
    ) -> Result<Vec<Tag>> {
        self.get(
            &format!("tags/slug/{}/related-tags/tags", request.slug),
            request,
        )
        .await
    }

    /// Retrieves a list of events with optional filtering.
    ///
    /// Events are collections of related markets (e.g., "2024 Presidential Election").
    /// Use filters to search by tags, active status, or other criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn events(&self, request: &EventsRequest) -> Result<Vec<Event>> {
        self.get("events", request).await
    }

    /// Retrieves a single event by its unique ID.
    ///
    /// Returns detailed information about an event including its markets,
    /// description, and associated tags.
    ///
    /// # Errors
    ///
    /// Returns an error if the event ID is invalid or the request fails.
    pub async fn event_by_id(&self, request: &EventByIdRequest) -> Result<Event> {
        self.get(&format!("events/{}", request.id), request).await
    }

    /// Retrieves a single event by its URL-friendly slug.
    ///
    /// Returns the same information as [`Self::event_by_id`] but uses a slug
    /// identifier instead of a numeric ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the slug is invalid or the request fails.
    pub async fn event_by_slug(&self, request: &EventBySlugRequest) -> Result<Event> {
        self.get(&format!("events/slug/{}", request.slug), request)
            .await
    }

    /// Retrieves all tags associated with an event.
    ///
    /// Returns the categorization tags for a specific event, helping understand
    /// the event's topics and categories.
    ///
    /// # Errors
    ///
    /// Returns an error if the event ID is invalid or the request fails.
    pub async fn event_tags(&self, request: &EventTagsRequest) -> Result<Vec<Tag>> {
        self.get(&format!("events/{}/tags", request.id), request)
            .await
    }

    /// Retrieves a list of prediction markets with optional filtering.
    ///
    /// Markets are the core trading instruments on Polymarket. Use filters to search
    /// by tags, events, active status, or CLOB token IDs. Returns market details
    /// including current prices, volume, and outcome information.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn markets(&self, request: &MarketsRequest) -> Result<Vec<Market>> {
        self.get("markets", request).await
    }

    /// Retrieves a single market by its unique ID.
    ///
    /// Returns detailed information about a specific market including outcomes,
    /// current prices, volume, and resolution details.
    ///
    /// # Errors
    ///
    /// Returns an error if the market ID is invalid or the request fails.
    pub async fn market_by_id(&self, request: &MarketByIdRequest) -> Result<Market> {
        self.get(&format!("markets/{}", request.id), request).await
    }

    /// Retrieves a single market by its URL-friendly slug.
    ///
    /// Returns the same information as [`Self::market_by_id`] but uses a slug
    /// identifier instead of a numeric ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the slug is invalid or the request fails.
    pub async fn market_by_slug(&self, request: &MarketBySlugRequest) -> Result<Market> {
        self.get(&format!("markets/slug/{}", request.slug), request)
            .await
    }

    /// Retrieves all tags associated with a market.
    ///
    /// Returns the categorization tags for a specific market, helping understand
    /// the market's topics and categories.
    ///
    /// # Errors
    ///
    /// Returns an error if the market ID is invalid or the request fails.
    pub async fn market_tags(&self, request: &MarketTagsRequest) -> Result<Vec<Tag>> {
        self.get(&format!("markets/{}/tags", request.id), request)
            .await
    }

    /// Retrieves a list of market series with optional filtering.
    ///
    /// Series are groups of related markets that follow a pattern (e.g., weekly
    /// sports outcomes, monthly economic indicators). Useful for tracking recurring
    /// predictions over time.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn series(&self, request: &SeriesListRequest) -> Result<Vec<Series>> {
        self.get("series", request).await
    }

    /// Retrieves a single series by its unique ID.
    ///
    /// Returns detailed information about a series including all markets in the series
    /// and their resolution history.
    ///
    /// # Errors
    ///
    /// Returns an error if the series ID is invalid or the request fails.
    pub async fn series_by_id(&self, request: &SeriesByIdRequest) -> Result<Series> {
        self.get(&format!("series/{}", request.id), request).await
    }

    /// Retrieves a list of user comments with optional filtering.
    ///
    /// Comments are user-generated discussions and analysis on markets and events.
    /// Use filters to search by market, event, or other criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn comments(&self, request: &CommentsRequest) -> Result<Vec<Comment>> {
        self.get("comments", request).await
    }

    /// Retrieves comments by their unique comment ID.
    ///
    /// Returns comments with the specified ID, including nested replies and
    /// associated metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the comment ID is invalid or the request fails.
    pub async fn comments_by_id(&self, request: &CommentsByIdRequest) -> Result<Vec<Comment>> {
        self.get(&format!("comments/{}", request.id), request).await
    }

    /// Retrieves all comments authored by a specific wallet address.
    ///
    /// Returns comments posted by a particular user, useful for viewing a user's
    /// contribution history and market analysis.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid or the request fails.
    pub async fn comments_by_user_address(
        &self,
        request: &CommentsByUserAddressRequest,
    ) -> Result<Vec<Comment>> {
        self.get(
            &format!("comments/user_address/{}", request.user_address),
            request,
        )
        .await
    }

    /// Retrieves a public trading profile for a wallet address.
    ///
    /// Returns public statistics about a trader including their trading history,
    /// win rate, and other performance metrics. Only publicly visible information
    /// is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid or the request fails.
    pub async fn public_profile(&self, request: &PublicProfileRequest) -> Result<PublicProfile> {
        self.get("public-profile", request).await
    }

    /// Searches across markets, events, and user profiles.
    ///
    /// Performs a text search to find markets, events, or users matching the query.
    /// Useful for discovery and finding specific content across the platform.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the search query is invalid.
    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResults> {
        self.get("public-search", request).await
    }
}
