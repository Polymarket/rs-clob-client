//! Client for the Polymarket Gamma API.
//!
//! This module provides an HTTP client for interacting with the Polymarket Gamma API,
//! which offers endpoints for querying events, markets, tags, series, comments, and more.
//!
//! # Example
//!
//! ```no_run
//! use polymarket_client_sdk::gamma::{Client, types::EventsRequest};
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
    Client as ReqwestClient, Method, Request, StatusCode,
    header::{HeaderMap, HeaderValue},
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use url::Url;

use super::types::{
    Comment, CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest, Event,
    EventByIdRequest, EventBySlugRequest, EventTagsRequest, EventsRequest, HealthResponse, Market,
    MarketByIdRequest, MarketBySlugRequest, MarketTagsRequest, MarketsRequest, PublicProfile,
    PublicProfileRequest, RelatedTag, RelatedTagsByIdRequest, RelatedTagsBySlugRequest,
    SearchRequest, SearchResults, Series, SeriesByIdRequest, SeriesListRequest,
    SportsMarketTypesResponse, SportsMetadata, Tag, TagByIdRequest, TagBySlugRequest, TagsRequest,
    Team, TeamsRequest,
};
use crate::Result;
use crate::error::Error;

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

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            level = "debug",
            skip(self, request, headers),
            fields(method, path, status_code)
        )
    )]
    async fn request<Response: DeserializeOwned + Serialize>(
        &self,
        mut request: Request,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        let method = request.method().clone();
        let path = request.url().path().to_owned();

        #[cfg(feature = "tracing")]
        {
            let span = tracing::Span::current();
            span.record("method", method.as_str());
            span.record("path", path.as_str());
        }

        if let Some(h) = headers {
            *request.headers_mut() = h;
        }

        let response = self.client.execute(request).await?;
        let status_code = response.status();

        #[cfg(feature = "tracing")]
        tracing::Span::current().record("status_code", status_code.as_u16());

        if !status_code.is_success() {
            let message = response.text().await.unwrap_or_default();

            #[cfg(feature = "tracing")]
            tracing::warn!(
                status = %status_code,
                method = %method,
                path = %path,
                message = %message,
                "Gamma API request failed"
            );

            return Err(Error::status(status_code, method, path, message));
        }

        if let Some(response) = response.json::<Option<Response>>().await? {
            Ok(response)
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(method = %method, path = %path, "Gamma API resource not found");

            Err(Error::status(
                StatusCode::NOT_FOUND,
                method,
                path,
                "Unable to find requested resource",
            ))
        }
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
        let params = serde_urlencoded::to_string(req).unwrap_or_default();
        let query = if params.is_empty() {
            params
        } else {
            format!("?{params}")
        };
        let request = self
            .client
            .request(Method::GET, format!("{}{path}{query}", self.host))
            .build()?;
        self.request(request, None).await
    }

    /// Performs a health check on the API.
    ///
    /// Returns "OK" when the API is healthy.
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

    /// Lists teams with optional filters.
    pub async fn teams(&self, request: &TeamsRequest) -> Result<Vec<Team>> {
        self.get("teams", request).await
    }

    /// Gets sports metadata.
    pub async fn sports(&self) -> Result<Vec<SportsMetadata>> {
        self.get("sports", &()).await
    }

    /// Gets valid sports market types.
    pub async fn sports_market_types(&self) -> Result<SportsMarketTypesResponse> {
        self.get("sports/market-types", &()).await
    }

    /// Lists tags with optional filters.
    pub async fn tags(&self, request: &TagsRequest) -> Result<Vec<Tag>> {
        self.get("tags", request).await
    }

    /// Gets a tag by ID.
    pub async fn tag_by_id(&self, request: &TagByIdRequest) -> Result<Tag> {
        self.get(&format!("tags/{}", request.id), request).await
    }

    /// Gets a tag by slug.
    pub async fn tag_by_slug(&self, request: &TagBySlugRequest) -> Result<Tag> {
        self.get(&format!("tags/slug/{}", request.slug), request)
            .await
    }

    /// Gets related tag relationships by tag ID.
    pub async fn related_tags_by_id(
        &self,
        request: &RelatedTagsByIdRequest,
    ) -> Result<Vec<RelatedTag>> {
        self.get(&format!("tags/{}/related-tags", request.id), request)
            .await
    }

    /// Gets related tag relationships by tag slug.
    pub async fn related_tags_by_slug(
        &self,
        request: &RelatedTagsBySlugRequest,
    ) -> Result<Vec<RelatedTag>> {
        self.get(&format!("tags/slug/{}/related-tags", request.slug), request)
            .await
    }

    /// Gets tags related to a tag by ID.
    pub async fn tags_related_to_tag_by_id(
        &self,
        request: &RelatedTagsByIdRequest,
    ) -> Result<Vec<Tag>> {
        self.get(&format!("tags/{}/related-tags/tags", request.id), request)
            .await
    }

    /// Gets tags related to a tag by slug.
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

    /// Lists events with optional filters.
    pub async fn events(&self, request: &EventsRequest) -> Result<Vec<Event>> {
        self.get("events", request).await
    }

    /// Gets an event by ID.
    pub async fn event_by_id(&self, request: &EventByIdRequest) -> Result<Event> {
        self.get(&format!("events/{}", request.id), request).await
    }

    /// Gets an event by slug.
    pub async fn event_by_slug(&self, request: &EventBySlugRequest) -> Result<Event> {
        self.get(&format!("events/slug/{}", request.slug), request)
            .await
    }

    /// Gets tags for an event by ID.
    pub async fn event_tags(&self, request: &EventTagsRequest) -> Result<Vec<Tag>> {
        self.get(&format!("events/{}/tags", request.id), request)
            .await
    }

    /// Lists markets with optional filters.
    pub async fn markets(&self, request: &MarketsRequest) -> Result<Vec<Market>> {
        self.get("markets", request).await
    }

    /// Gets a market by ID.
    pub async fn market_by_id(&self, request: &MarketByIdRequest) -> Result<Market> {
        self.get(&format!("markets/{}", request.id), request).await
    }

    /// Gets a market by slug.
    pub async fn market_by_slug(&self, request: &MarketBySlugRequest) -> Result<Market> {
        self.get(&format!("markets/slug/{}", request.slug), request)
            .await
    }

    /// Gets tags for a market by ID.
    pub async fn market_tags(&self, request: &MarketTagsRequest) -> Result<Vec<Tag>> {
        self.get(&format!("markets/{}/tags", request.id), request)
            .await
    }

    /// Lists series with optional filters.
    pub async fn series(&self, request: &SeriesListRequest) -> Result<Vec<Series>> {
        self.get("series", request).await
    }

    /// Gets a series by ID.
    pub async fn series_by_id(&self, request: &SeriesByIdRequest) -> Result<Series> {
        self.get(&format!("series/{}", request.id), request).await
    }

    /// Lists comments with optional filters.
    pub async fn comments(&self, request: &CommentsRequest) -> Result<Vec<Comment>> {
        self.get("comments", request).await
    }

    /// Gets comments by comment ID.
    pub async fn comments_by_id(&self, request: &CommentsByIdRequest) -> Result<Vec<Comment>> {
        self.get(&format!("comments/{}", request.id), request).await
    }

    /// Gets comments by user address.
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

    /// Gets a public profile by wallet address.
    pub async fn public_profile(&self, request: &PublicProfileRequest) -> Result<PublicProfile> {
        self.get("public-profile", request).await
    }

    /// Searches markets, events, and profiles.
    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResults> {
        self.get("public-search", request).await
    }
}
