//! Client for the Polymarket Data API.
//!
//! This module provides an HTTP client for interacting with the Polymarket Data API,
//! which offers endpoints for querying user positions, trades, activity, and market data.
//!
//! # Example
//!
//! ```no_run
//! use alloy::primitives::address;
//! use polymarket_client_sdk::data_api::{Client, request::PositionsRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::default();
//!
//! let request = PositionsRequest::builder()
//!     .user(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
//!     .build();
//!
//! let positions = client.positions(&request).await?;
//! for position in positions {
//!     println!("{}: {} tokens", position.title, position.size);
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

use super::request::{
    ActivityRequest, BuilderLeaderboardRequest, BuilderVolumeRequest, ClosedPositionsRequest,
    HoldersRequest, LiveVolumeRequest, OpenInterestRequest, PositionsRequest, TradedRequest,
    TraderLeaderboardRequest, TradesRequest, ValueRequest, to_query_string,
};
use super::response::{
    Activity, BuilderLeaderboardEntry, BuilderVolumeEntry, ClosedPosition, Health, LiveVolume,
    MetaHolder, OpenInterest, Position, Trade, Traded, TraderLeaderboardEntry, Value,
};
use crate::Result;
use crate::error::Error;

/// HTTP client for the Polymarket Data API.
///
/// Provides methods for querying user positions, trades, activity, market holders,
/// open interest, volume data, and leaderboards.
///
/// # API Base URL
///
/// The default API endpoint is `https://data-api.polymarket.com`.
///
/// # Example
///
/// ```no_run
/// use polymarket_client_sdk::data_api::Client;
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
        Client::new("https://data-api.polymarket.com")
            .expect("Client with default endpoint should succeed")
    }
}

impl Client {
    /// Creates a new Data API client with a custom host URL.
    ///
    /// # Arguments
    ///
    /// * `host` - The base URL for the Data API (e.g., `https://data-api.polymarket.com`).
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
    async fn request<Response: DeserializeOwned>(
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
                "Data API request failed"
            );

            return Err(Error::status(status_code, method, path, message));
        }

        if let Some(response) = response.json::<Option<Response>>().await? {
            Ok(response)
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(method = %method, path = %path, "Data API resource not found");
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

    async fn get<Req: Serialize, Res: DeserializeOwned>(
        &self,
        path: &str,
        req: &Req,
    ) -> Result<Res> {
        let query = to_query_string(req);
        let request = self
            .client
            .request(Method::GET, format!("{}{path}{query}", self.host))
            .build()?;
        self.request(request, None).await
    }

    /// Performs a health check on the API.
    ///
    /// Returns "OK" when the API is healthy and operational.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn health(&self) -> Result<Health> {
        self.get("", &()).await
    }

    /// Fetches current (open) positions for a user.
    ///
    /// Positions represent holdings of outcome tokens in prediction markets.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn positions(&self, req: &PositionsRequest) -> Result<Vec<Position>> {
        self.get("positions", req).await
    }

    /// Fetches trade history for a user or markets.
    ///
    /// Trades represent executed orders where outcome tokens were bought or sold.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn trades(&self, req: &TradesRequest) -> Result<Vec<Trade>> {
        self.get("trades", req).await
    }

    /// Fetches on-chain activity for a user.
    ///
    /// Returns various on-chain operations including trades, splits, merges,
    /// redemptions, rewards, and conversions.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn activity(&self, req: &ActivityRequest) -> Result<Vec<Activity>> {
        self.get("activity", req).await
    }

    /// Fetches top token holders for specified markets.
    ///
    /// Returns holders grouped by token (outcome) for each market.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn holders(&self, req: &HoldersRequest) -> Result<Vec<MetaHolder>> {
        self.get("holders", req).await
    }

    /// Fetches the total value of a user's positions.
    ///
    /// Optionally filtered by specific markets.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn value(&self, req: &ValueRequest) -> Result<Vec<Value>> {
        self.get("value", req).await
    }

    /// Fetches closed (historical) positions for a user.
    ///
    /// These are positions that have been fully sold or redeemed.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn closed_positions(
        &self,
        req: &ClosedPositionsRequest,
    ) -> Result<Vec<ClosedPosition>> {
        self.get("closed-positions", req).await
    }

    /// Fetches trader leaderboard rankings.
    ///
    /// Returns trader rankings filtered by category, time period, and ordering.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn leaderboard(
        &self,
        req: &TraderLeaderboardRequest,
    ) -> Result<Vec<TraderLeaderboardEntry>> {
        self.get("v1/leaderboard", req).await
    }

    /// Fetches the total count of unique markets a user has traded.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn traded(&self, req: &TradedRequest) -> Result<Traded> {
        self.get("traded", req).await
    }

    /// Fetches open interest for markets.
    ///
    /// Open interest represents the total value of outstanding positions in a market.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn open_interest(&self, req: &OpenInterestRequest) -> Result<Vec<OpenInterest>> {
        self.get("oi", req).await
    }

    /// Fetches live trading volume for an event.
    ///
    /// Includes total volume and per-market breakdown.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn live_volume(&self, req: &LiveVolumeRequest) -> Result<Vec<LiveVolume>> {
        self.get("live-volume", req).await
    }

    /// Fetches aggregated builder leaderboard rankings.
    ///
    /// Builders are third-party applications that integrate with Polymarket.
    /// Returns one entry per builder with aggregated totals for the specified time period.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn builder_leaderboard(
        &self,
        req: &BuilderLeaderboardRequest,
    ) -> Result<Vec<BuilderLeaderboardEntry>> {
        self.get("v1/builders/leaderboard", req).await
    }

    /// Fetches daily time-series volume data for builders.
    ///
    /// Returns multiple entries per builder (one per day), each including a timestamp.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the API returns an error response.
    pub async fn builder_volume(
        &self,
        req: &BuilderVolumeRequest,
    ) -> Result<Vec<BuilderVolumeEntry>> {
        self.get("v1/builders/volume", req).await
    }
}
