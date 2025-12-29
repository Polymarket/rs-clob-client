use reqwest::{
    Client as ReqwestClient, Method, Request, StatusCode,
    header::{HeaderMap, HeaderValue},
};
use serde::de::DeserializeOwned;
use url::Url;

use super::types::{DepositRequest, DepositResponse, SupportedAssetsResponse};
use crate::Result;
use crate::error::Error;

/// Client for the Polymarket Bridge API.
///
/// The Bridge API enables bridging assets from various chains (EVM, Solana, Bitcoin)
/// to USDC.e on Polygon for trading on Polymarket.
///
/// # Example
///
/// ```no_run
/// use alloy::primitives::address;
/// use polymarket_client_sdk::bridge::{Client, types::DepositRequest};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::default();
///
/// // Get deposit addresses
/// let request = DepositRequest::builder()
///     .address(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
///     .build();
/// let response = client.deposit(&request).await?;
///
/// // Get supported assets
/// let assets = client.supported_assets().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Client {
    host: Url,
    client: ReqwestClient,
}

impl Default for Client {
    fn default() -> Self {
        Client::new("https://bridge.polymarket.com")
            .expect("Client with default endpoint should succeed")
    }
}

impl Client {
    /// Creates a new Bridge API client with a custom host.
    ///
    /// # Errors
    ///
    /// Returns an error if the host URL is invalid or the HTTP client fails to build.
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
                "Bridge API request failed"
            );

            return Err(Error::status(status_code, method, path, message));
        }

        if let Some(response) = response.json::<Option<Response>>().await? {
            Ok(response)
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(method = %method, path = %path, "Bridge API resource not found");
            Err(Error::status(
                StatusCode::NOT_FOUND,
                method,
                path,
                "Unable to find requested resource",
            ))
        }
    }

    /// Returns the host URL for the client.
    #[must_use]
    pub fn host(&self) -> &Url {
        &self.host
    }

    #[must_use]
    fn client(&self) -> &ReqwestClient {
        &self.client
    }

    /// Create deposit addresses for a Polymarket wallet.
    ///
    /// Generates unique deposit addresses for bridging assets to Polymarket.
    /// Returns addresses for EVM-compatible chains, Solana, and Bitcoin.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use alloy::primitives::address;
    /// use polymarket_client_sdk::bridge::{Client, types::DepositRequest};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::default();
    /// let request = DepositRequest::builder()
    ///     .address(address!("56687bf447db6ffa42ffe2204a05edaa20f55839"))
    ///     .build();
    ///
    /// let response = client.deposit(&request).await?;
    /// println!("EVM: {}", response.address.evm);
    /// println!("SVM: {}", response.address.svm);
    /// println!("BTC: {}", response.address.btc);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn deposit(&self, request: &DepositRequest) -> Result<DepositResponse> {
        let request = self
            .client()
            .request(Method::POST, format!("{}deposit", self.host()))
            .json(request)
            .build()?;

        self.request(request, None).await
    }

    /// Get all supported chains and tokens for deposits.
    ///
    /// Returns information about which assets can be deposited and their
    /// minimum deposit amounts in USD.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_client_sdk::bridge::Client;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::default();
    /// let response = client.supported_assets().await?;
    ///
    /// for asset in response.supported_assets {
    ///     println!(
    ///         "{} ({}) on {} - min: ${:.2}",
    ///         asset.token.name,
    ///         asset.token.symbol,
    ///         asset.chain_name,
    ///         asset.min_checkout_usd
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn supported_assets(&self) -> Result<SupportedAssetsResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}supported-assets", self.host()))
            .build()?;

        self.request(request, None).await
    }
}
