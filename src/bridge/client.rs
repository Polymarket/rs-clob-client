use reqwest::{Client as ReqwestClient, Method};
use url::Url;

use super::types::{DepositRequest, DepositResponse, SupportedAssetsResponse};
use crate::Result;

/// Client for the Polymarket Bridge API.
///
/// The Bridge API enables bridging assets from various chains (EVM, Solana, Bitcoin)
/// to USDC.e on Polygon for trading on Polymarket.
///
/// # Example
///
/// ```no_run
/// use polymarket_client_sdk::types::address;
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
        Self::new_with_proxy(host, None)
    }

    /// Creates a new Bridge API client with a custom host URL and optional proxy.
    ///
    /// # Arguments
    ///
    /// * `host` - The base URL for the Bridge API (e.g., `"https://bridge.polymarket.com"`).
    /// * `proxy` - Optional proxy URL (e.g., `"http://user:pass@proxy:8080"` or `"socks5://127.0.0.1:1080"`).
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid, the proxy URL is invalid, or the HTTP client cannot be created.
    pub fn new_with_proxy(host: &str, proxy: Option<&str>) -> Result<Client> {
        let client = crate::http::client_builder(proxy)?.build()?;

        Ok(Self {
            host: Url::parse(host)?,
            client,
        })
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
    /// use polymarket_client_sdk::types::address;
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

        crate::request(&self.client, request, None).await
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

        crate::request(&self.client, request, None).await
    }
}
