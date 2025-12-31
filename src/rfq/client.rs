//! RFQ Client implementation
//!
//! Provides the [`Client`] for interacting with the RFQ (Request for Quote) API.
//! All endpoints require L2 authentication.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE;
use chrono::Utc;
use hmac::{Hmac, Mac as _};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client as ReqwestClient, Method, Request, StatusCode};
use sha2::Sha256;
use url::Url;

use crate::Result;
use crate::ToQueryParams as _;
use crate::auth::Credentials;
use crate::error::Error;
use crate::rfq::types::request::{
    AcceptQuoteRequest, ApproveOrderRequest, CancelQuoteRequest, CancelRfqRequestRequest,
    CreateQuoteRequest, CreateRfqRequestRequest, GetQuotesRequest, GetRfqRequestsRequest,
};
use crate::rfq::types::response::{
    AcceptQuoteResponse, ApproveOrderResponse, CreateQuoteResponse, CreateRfqRequestResponse,
    Quote, RfqPage, RfqRequest,
};

const POLY_ADDRESS: &str = "POLY_ADDRESS";
const POLY_API_KEY: &str = "POLY_API_KEY";
const POLY_PASSPHRASE: &str = "POLY_PASSPHRASE";
const POLY_SIGNATURE: &str = "POLY_SIGNATURE";
const POLY_TIMESTAMP: &str = "POLY_TIMESTAMP";

/// Client for the RFQ (Request for Quote) API.
///
/// This is a standalone client that handles its own authentication and HTTP requests.
/// All endpoints require L2 authentication via credentials.
///
/// # Example
///
/// ```rust,no_run
/// use polymarket_client_sdk::rfq::Client;
/// use polymarket_client_sdk::rfq::types::request::{CreateRfqRequestRequest, UserType};
/// use polymarket_client_sdk::auth::Credentials;
/// use alloy::primitives::Address;
/// use uuid::Uuid;
/// use std::str::FromStr;
///
/// # async fn example() -> anyhow::Result<()> {
/// let address = Address::from_str("0x...")?;
/// let credentials = Credentials::new(
///     Uuid::new_v4(),
///     "your-secret".to_string(),
///     "your-passphrase".to_string(),
/// );
///
/// let rfq_client = Client::new("https://clob.polymarket.com", address, credentials)?;
///
/// let request = CreateRfqRequestRequest::builder()
///     .asset_in("12345")
///     .asset_out("0")
///     .amount_in("50000000")
///     .amount_out("3000000")
///     .user_type(UserType::Eoa)
///     .build();
///
/// let response = rfq_client.create_request(&request).await?;
/// println!("Created RFQ request: {}", response.request_id);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Client {
    host: Url,
    address: alloy::primitives::Address,
    credentials: Credentials,
    http: ReqwestClient,
}

impl Client {
    /// Creates a new RFQ client.
    ///
    /// # Arguments
    ///
    /// * `host` - The base URL for the RFQ API (e.g., `https://clob.polymarket.com`)
    /// * `address` - The user's Ethereum address
    /// * `credentials` - L2 authentication credentials
    ///
    /// # Errors
    ///
    /// Returns an error if the host URL is invalid or the HTTP client cannot be created.
    pub fn new(
        host: &str,
        address: alloy::primitives::Address,
        credentials: Credentials,
    ) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", HeaderValue::from_static("rs_clob_client"));
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let http = ReqwestClient::builder().default_headers(headers).build()?;

        Ok(Self {
            host: Url::parse(host)?,
            address,
            credentials,
            http,
        })
    }

    /// Returns the host URL.
    #[must_use]
    pub fn host(&self) -> &Url {
        &self.host
    }

    /// Creates L2 authentication headers for a request.
    fn create_headers(&self, request: &Request) -> Result<HeaderMap> {
        use alloy::hex::ToHexExt as _;

        let timestamp = Utc::now().timestamp();
        let message = to_message(request, timestamp);
        let signature = hmac(&self.credentials, &message)?;

        let mut map = HeaderMap::new();
        map.insert(POLY_ADDRESS, self.address.encode_hex_with_prefix().parse()?);
        map.insert(POLY_API_KEY, self.credentials.key.to_string().parse()?);
        map.insert(
            POLY_PASSPHRASE,
            self.credentials.passphrase.reveal().parse()?,
        );
        map.insert(POLY_SIGNATURE, signature.parse()?);
        map.insert(POLY_TIMESTAMP, timestamp.to_string().parse()?);

        Ok(map)
    }

    /// Executes a request and deserializes the JSON response.
    async fn request<Response: serde::de::DeserializeOwned>(
        &self,
        mut request: Request,
        headers: HeaderMap,
    ) -> Result<Response> {
        let method = request.method().clone();
        let path = request.url().path().to_owned();

        *request.headers_mut() = headers;

        let response = self.http.execute(request).await?;
        let status = response.status();

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(Error::status(status, method, path, message));
        }

        if let Some(result) = response.json::<Option<Response>>().await? {
            Ok(result)
        } else {
            Err(Error::status(
                StatusCode::NOT_FOUND,
                method,
                path,
                "Unable to find requested resource",
            ))
        }
    }

    /// Executes a request that returns text (like "OK") instead of JSON.
    async fn request_text(&self, mut request: Request, headers: HeaderMap) -> Result<()> {
        let method = request.method().clone();
        let path = request.url().path().to_owned();

        *request.headers_mut() = headers;

        let response = self.http.execute(request).await?;
        let status = response.status();

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(Error::status(status, method, path, message));
        }

        Ok(())
    }

    // =========================================================================
    // Request Endpoints
    // =========================================================================

    /// Creates an RFQ Request to buy or sell outcome tokens.
    ///
    /// This initiates the RFQ flow where market makers can provide quotes.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn create_request(
        &self,
        request: &CreateRfqRequestRequest,
    ) -> Result<CreateRfqRequestResponse> {
        let http_request = self
            .http
            .request(Method::POST, format!("{}rfq/request", self.host))
            .json(request)
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request(http_request, headers).await
    }

    /// Cancels an RFQ request.
    ///
    /// The request must be in the `STATE_ACCEPTING_QUOTES` state.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the request cannot be canceled.
    pub async fn cancel_request(&self, request: &CancelRfqRequestRequest) -> Result<()> {
        let http_request = self
            .http
            .request(Method::DELETE, format!("{}rfq/request", self.host))
            .json(request)
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request_text(http_request, headers).await
    }

    /// Gets RFQ requests.
    ///
    /// Requesters can only view their own requests.
    /// Quoters can only see their own quotes and requests that they quoted.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn get_requests(
        &self,
        request: &GetRfqRequestsRequest,
    ) -> Result<RfqPage<RfqRequest>> {
        let params = request.query_params(None);
        let http_request = self
            .http
            .request(Method::GET, format!("{}rfq/request{params}", self.host))
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request(http_request, headers).await
    }

    /// Gets the next page of RFQ requests using a cursor.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn get_requests_with_cursor(
        &self,
        request: &GetRfqRequestsRequest,
        next_cursor: &str,
    ) -> Result<RfqPage<RfqRequest>> {
        let params = request.query_params(Some(next_cursor));
        let http_request = self
            .http
            .request(Method::GET, format!("{}rfq/request{params}", self.host))
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request(http_request, headers).await
    }

    // =========================================================================
    // Quote Endpoints
    // =========================================================================

    /// Creates an RFQ Quote in response to a Request.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn create_quote(&self, request: &CreateQuoteRequest) -> Result<CreateQuoteResponse> {
        let http_request = self
            .http
            .request(Method::POST, format!("{}rfq/quote", self.host))
            .json(request)
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request(http_request, headers).await
    }

    /// Cancels an RFQ quote.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the quote cannot be canceled.
    pub async fn cancel_quote(&self, request: &CancelQuoteRequest) -> Result<()> {
        let http_request = self
            .http
            .request(Method::DELETE, format!("{}rfq/quote", self.host))
            .json(request)
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request_text(http_request, headers).await
    }

    /// Gets RFQ quotes.
    ///
    /// Requesters can view quotes for their requests.
    /// Quoters can view all quotes.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn get_quotes(&self, request: &GetQuotesRequest) -> Result<RfqPage<Quote>> {
        let params = request.query_params(None);
        let http_request = self
            .http
            .request(Method::GET, format!("{}rfq/quote{params}", self.host))
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request(http_request, headers).await
    }

    /// Gets the next page of RFQ quotes using a cursor.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn get_quotes_with_cursor(
        &self,
        request: &GetQuotesRequest,
        next_cursor: &str,
    ) -> Result<RfqPage<Quote>> {
        let params = request.query_params(Some(next_cursor));
        let http_request = self
            .http
            .request(Method::GET, format!("{}rfq/quote{params}", self.host))
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request(http_request, headers).await
    }

    // =========================================================================
    // Execution Endpoints
    // =========================================================================

    /// Requester accepts an RFQ Quote.
    ///
    /// This creates an Order that the Requester must sign. The signed order
    /// is submitted to the API to initiate the trade.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the quote cannot be accepted.
    pub async fn accept_quote(&self, request: &AcceptQuoteRequest) -> Result<AcceptQuoteResponse> {
        let http_request = self
            .http
            .request(Method::POST, format!("{}rfq/request/accept", self.host))
            .json(request)
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request_text(http_request, headers).await?;
        Ok(AcceptQuoteResponse)
    }

    /// Quoter approves an RFQ order during the last look window.
    ///
    /// This queues the order for onchain execution.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the order cannot be approved.
    pub async fn approve_order(
        &self,
        request: &ApproveOrderRequest,
    ) -> Result<ApproveOrderResponse> {
        let http_request = self
            .http
            .request(Method::POST, format!("{}rfq/quote/approve", self.host))
            .json(request)
            .build()?;
        let headers = self.create_headers(&http_request)?;

        self.request(http_request, headers).await
    }
}

/// Creates the message to sign for L2 authentication.
fn to_message(request: &Request, timestamp: i64) -> String {
    let method = request.method();
    let body = request
        .body()
        .and_then(|b| b.as_bytes())
        .map(String::from_utf8_lossy)
        .map(|b| b.replace('\'', "\""))
        .unwrap_or_default();
    let path = request.url().path();

    format!("{timestamp}{method}{path}{body}")
}

/// Creates an HMAC-SHA256 signature.
fn hmac(credentials: &Credentials, message: &str) -> Result<String> {
    let decoded_secret = URL_SAFE.decode(credentials.secret.reveal())?;
    let mut mac = Hmac::<Sha256>::new_from_slice(&decoded_secret)?;
    mac.update(message.as_bytes());

    let result = mac.finalize().into_bytes();
    Ok(URL_SAFE.encode(result))
}
