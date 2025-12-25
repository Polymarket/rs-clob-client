use std::borrow::Cow;
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;

use alloy::dyn_abi::Eip712Domain;
use alloy::primitives::{Address, U256};
use alloy::signers::Signer;
use alloy::sol_types::SolStruct as _;
use async_stream::try_stream;
use bon::Builder;
use chrono::{NaiveDate, Utc};
use dashmap::DashMap;
use futures::Stream;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client as ReqwestClient, Method, Request, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::json;
use url::Url;

use crate::auth::builder::{Builder, Config as BuilderConfig};
use crate::auth::state::{Authenticated, State, Unauthenticated};
use crate::auth::{Credentials, Kind, Normal};
use crate::clob::order_builder::{Limit, Market, OrderBuilder, generate_seed};
use crate::clob::types::{
    ApiKeysResponse, BalanceAllowanceRequest, BalanceAllowanceResponse, BanStatusResponse,
    BuilderApiKeyResponse, BuilderTradeResponse, CancelMarketOrderRequest, CancelOrdersResponse,
    CurrentRewardResponse, DeleteNotificationsRequest, FeeRateResponse, LastTradePriceRequest,
    LastTradePriceResponse, LastTradesPricesResponse, MarketResponse, MarketRewardResponse,
    MidpointRequest, MidpointResponse, MidpointsResponse, NegRiskResponse, NotificationResponse,
    OpenOrderResponse, OrderBookSummaryRequest, OrderBookSummaryResponse, OrderScoringResponse,
    OrdersRequest, OrdersScoringResponse, Page, PostOrderResponse, PriceRequest, PriceResponse,
    PricesResponse, RewardsPercentagesResponse, SignableOrder, SignatureType, SignedOrder,
    SimplifiedMarketResponse, SpreadRequest, SpreadResponse, SpreadsResponse, TickSize,
    TickSizeResponse, TotalUserEarningResponse, TradeResponse, TradesRequest,
    UpdateBalanceAllowanceRequest, UserEarningResponse, UserRewardsEarningRequest,
    UserRewardsEarningResponse,
};
use crate::error::{Error, Synchronization};
use crate::{AMOY, POLYGON, Result, Timestamp, auth, contract_config};

const ORDER_NAME: Option<Cow<'static, str>> = Some(Cow::Borrowed("Polymarket CTF Exchange"));
const VERSION: Option<Cow<'static, str>> = Some(Cow::Borrowed("1"));

const TERMINAL_CURSOR: &str = "LTE="; // base64("-1")

/// The type used to build a request to authenticate the inner [`Client<Unauthorized>`]. Calling
/// `authenticate` on this will elevate that inner `client` into an [`Client<Authenticated<K>>`].
pub struct AuthenticationBuilder<'signer, S: Signer, K: Kind = Normal> {
    /// The initially unauthenticated client that is "carried forward" into the authenticated client.
    client: Client<Unauthenticated>,
    /// The signer used to generate the L1 headers that will return a set of [`Credentials`].
    signer: &'signer S,
    /// If [`Credentials`] are supplied, then those are used instead of making new calls to obtain one.
    credentials: Option<Credentials>,
    /// An optional `nonce` value, when `credentials` are not present, to pass along to the call to
    /// create or derive [`Credentials`].
    nonce: Option<u32>,
    /// The [`Kind`] that this [`AuthenticationBuilder`] exhibits. Used to generate additional
    /// headers for different types of authentication, e.g. Builder.
    kind: K,
    /// The optional [`Address`] used to represent the funder for this `client`. If a funder is set
    /// then `signature_type` must match `Some(SignatureType::Proxy | Signature::GnosisSafe)`. Conversely,
    /// if funder is not set, then `signature_type` must be `Some(SignatureType::Eoa)`.
    funder: Option<Address>,
    /// The optional [`SignatureType`], see `funder` for more information.
    signature_type: Option<SignatureType>,
    /// The optional salt/seed generator for use in creating [`SignableOrder`]s
    salt_generator: Option<fn() -> u64>,
}

impl<S: Signer, K: Kind> AuthenticationBuilder<'_, S, K> {
    #[must_use]
    pub fn nonce(mut self, nonce: u32) -> Self {
        self.nonce = Some(nonce);
        self
    }

    #[must_use]
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    #[must_use]
    pub fn funder(mut self, funder: Address) -> Self {
        self.funder = Some(funder);
        self
    }

    #[must_use]
    pub fn signature_type(mut self, signature_type: SignatureType) -> Self {
        self.signature_type = Some(signature_type);
        self
    }

    #[must_use]
    pub fn salt_generator(mut self, salt_generator: fn() -> u64) -> Self {
        self.salt_generator = Some(salt_generator);
        self
    }

    /// Attempt to elevate the inner `client` to [`Client<Authenticated<K>>`] using the optional
    /// fields supplied in the builder.
    pub async fn authenticate(self) -> Result<Client<Authenticated<K>>> {
        let inner = Arc::into_inner(self.client.inner).ok_or(Synchronization)?;

        match self.signer.chain_id() {
            Some(chain) if chain == POLYGON || chain == AMOY => {}
            Some(chain) => {
                return Err(Error::validation(format!(
                    "Only Polygon and AMOY are supported, got {chain}"
                )));
            }
            None => {
                return Err(Error::validation(
                    "Chain id not set, be sure to provide one on the signer",
                ));
            }
        }

        match (self.funder, self.signature_type) {
            (Some(_), Some(sig @ SignatureType::Eoa)) => {
                return Err(Error::validation(format!(
                    "Cannot have a funder address with a {sig} signature type"
                )));
            }
            (
                Some(Address::ZERO),
                Some(sig @ (SignatureType::Proxy | SignatureType::GnosisSafe)),
            ) => {
                return Err(Error::validation(format!(
                    "Cannot have a zero funder address with a {sig} signature type"
                )));
            }
            (None, Some(sig @ (SignatureType::Proxy | SignatureType::GnosisSafe))) => {
                return Err(Error::validation(format!(
                    "Must have a funder address with a {sig} signature type"
                )));
            }
            _ => {}
        }

        let credentials = match self.credentials {
            Some(_) if self.nonce.is_some() => {
                return Err(Error::validation(
                    "Credentials and nonce are both set. If nonce is set, then you must not supply credentials",
                ));
            }
            Some(credentials) => credentials,
            None => {
                inner
                    .create_or_derive_api_key(self.signer, self.nonce)
                    .await?
            }
        };

        let state = Authenticated {
            address: self.signer.address(),
            credentials,
            kind: self.kind,
        };

        Ok(Client {
            inner: Arc::new(ClientInner {
                state,
                config: inner.config,
                host: inner.host,
                client: inner.client,
                tick_sizes: inner.tick_sizes,
                neg_risk: inner.neg_risk,
                fee_rate_bps: inner.fee_rate_bps,
                funder: self.funder,
                signature_type: self.signature_type.unwrap_or(SignatureType::Eoa),
                salt_generator: self.salt_generator.unwrap_or(generate_seed),
            }),
        })
    }
}

/// The main way for API users to interact with the Polymarket CLOB.
///
/// A [`Client`] can either be [`Unauthenticated`] or [`Authenticated`], that is, authenticated
/// with a particular [`Signer`], `S`, and a particular [`AuthKind`], `K`. That [`AuthKind`] lets
/// the client know if it's authenticating [`Normal`]ly or as a [`auth::builder::Builder`].
///
/// Only the allowed methods will be available for use when in a particular state, i.e. only
/// unauthenticated methods will be visible when unauthenticated, same for authenticated/builder
/// authenticated methods.
///
/// [`Client`] is thread-safe
///
/// Create an unauthenticated client:
/// ```rust
/// use polymarket_client_sdk::Result;
/// use polymarket_client_sdk::clob::{Client, Config};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = Client::new("https://clob.polymarket.com", Config::default())?;
///
///     let ok = client.ok().await?;
///     println!("Ok: {ok}");
///
///     Ok(())
/// }
/// ```
///
/// Elevate into an authenticated client:
/// ```rust,no_run
/// use std::str::FromStr as _;
///
/// use alloy::signers::Signer as _;
/// use alloy::signers::local::LocalSigner;
/// use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
/// use polymarket_client_sdk::clob::{Client, Config};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need a private key");
///     let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));
///     let client = Client::new("https://clob.polymarket.com", Config::default())?
///         .authentication_builder(&signer)
///         .authenticate()
///         .await?;
///
///     let ok = client.ok().await?;
///     println!("Ok: {ok}");
///
///     let api_keys = client.api_keys().await?;
///     println!("API keys: {api_keys:?}");
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Client<S: State = Unauthenticated> {
    inner: Arc<ClientInner<S>>,
}

impl Default for Client<Unauthenticated> {
    fn default() -> Self {
        Client::new("https://clob.polymarket.com", Config::default())
            .expect("Client with default endpoint should succeed")
    }
}

/// Configuration for [`Client`]
#[derive(Clone, Debug, Default, Builder)]
pub struct Config {
    /// Whether the [`Client`] will use the server time provided by Polymarket when creating auth
    /// headers. This adds another round trip to the requests.
    use_server_time: bool,
}

#[derive(Debug)]
struct ClientInner<S: State> {
    config: Config,
    /// The current [`State`] of this client
    state: S,
    /// The [`Url`] against which `client` is making requests.
    host: Url,
    /// The inner [`ReqwestClient`] used to make requests to `host`.
    client: ReqwestClient,
    /// Local cache of [`TickSize`] per token ID
    tick_sizes: DashMap<String, TickSize>,
    /// Local cache representing whether this token is part of a `neg_risk` market
    neg_risk: DashMap<String, bool>,
    /// Local cache representing the fee rate in basis points per token ID
    fee_rate_bps: DashMap<String, u32>,
    /// The funder for this [`ClientInner`]. If funder is present, then `signature_type` cannot
    /// be [`SignatureType::Eoa`]. Conversely, if funder is absent, then `signature_type` cannot be
    /// [`SignatureType::Proxy`] or [`SignatureType::GnosisSafe`].
    funder: Option<Address>,
    /// The signature type for this [`ClientInner`]. Defaults to [`SignatureType::Eoa`]
    signature_type: SignatureType,
    /// The salt/seed generator for use in creating [`SignableOrder`]s
    salt_generator: fn() -> u64,
}

impl<S: State> ClientInner<S> {
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
                "CLOB API request failed"
            );

            return Err(Error::status(status_code, method, path, message));
        }

        if let Some(response) = response.json::<Option<Response>>().await? {
            Ok(response)
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!(method = %method, path = %path, "CLOB API resource not found");
            Err(Error::status(
                StatusCode::NOT_FOUND,
                method,
                path,
                "Unable to find requested resource",
            ))
        }
    }

    pub async fn server_time(&self) -> Result<Timestamp> {
        let request = self
            .client
            .request(Method::GET, format!("{}time", self.host))
            .build()?;

        self.request(request, None).await
    }
}

impl ClientInner<Unauthenticated> {
    pub async fn create_api_key<S: Signer>(
        &self,
        signer: &S,
        nonce: Option<u32>,
    ) -> Result<Credentials> {
        let request = self
            .client
            .request(Method::POST, format!("{}auth/api-key", self.host))
            .build()?;
        let headers = self.create_headers(signer, nonce).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn derive_api_key<S: Signer>(
        &self,
        signer: &S,
        nonce: Option<u32>,
    ) -> Result<Credentials> {
        let request = self
            .client
            .request(Method::GET, format!("{}auth/derive-api-key", self.host))
            .build()?;
        let headers = self.create_headers(signer, nonce).await?;

        self.request(request, Some(headers)).await
    }

    async fn create_or_derive_api_key<S: Signer>(
        &self,
        signer: &S,
        nonce: Option<u32>,
    ) -> Result<Credentials> {
        match self.create_api_key(signer, nonce).await {
            Ok(creds) => Ok(creds),
            Err(_) => self.derive_api_key(signer, nonce).await,
        }
    }

    async fn create_headers<S: Signer>(&self, signer: &S, nonce: Option<u32>) -> Result<HeaderMap> {
        let chain_id = signer.chain_id().ok_or(Error::validation(
            "Chain id not set, be sure to provide one on the signer",
        ))?;

        let timestamp = if self.config.use_server_time {
            self.server_time().await?
        } else {
            Utc::now().timestamp()
        };

        auth::l1::create_headers(signer, chain_id, timestamp, nonce).await
    }
}

impl<S: State> Client<S> {
    #[must_use]
    pub fn host(&self) -> &Url {
        &self.inner.host
    }

    pub fn invalidate_internal_caches(&self) {
        self.inner.tick_sizes.clear();
        self.inner.fee_rate_bps.clear();
        self.inner.neg_risk.clear();
    }

    pub async fn ok(&self) -> Result<String> {
        let request = self
            .client()
            .request(Method::GET, self.host().to_owned())
            .build()?;

        self.request(request, None).await
    }

    pub async fn server_time(&self) -> Result<Timestamp> {
        self.inner.server_time().await
    }

    pub async fn midpoint(&self, request: &MidpointRequest) -> Result<MidpointResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}midpoint", self.host()))
            .query(&[("token_id", request.token_id.as_str())])
            .build()?;

        self.request(request, None).await
    }

    pub async fn midpoints(&self, requests: &[MidpointRequest]) -> Result<MidpointsResponse> {
        let request = self
            .client()
            .request(Method::POST, format!("{}midpoints", self.host()))
            .json(requests)
            .build()?;

        self.request(request, None).await
    }

    pub async fn price(&self, request: &PriceRequest) -> Result<PriceResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}price", self.host()))
            .query(&[
                ("token_id", request.token_id.as_str()),
                ("side", &request.side.to_string()),
            ])
            .build()?;

        self.request(request, None).await
    }

    pub async fn prices(&self, requests: &[PriceRequest]) -> Result<PricesResponse> {
        let request = self
            .client()
            .request(Method::POST, format!("{}prices", self.host()))
            .json(requests)
            .build()?;

        self.request(request, None).await
    }

    pub async fn spread(&self, request: &SpreadRequest) -> Result<SpreadResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}spread", self.host()))
            .query(&[("token_id", request.token_id.as_str())])
            .build()?;

        self.request(request, None).await
    }

    pub async fn spreads(&self, requests: &[SpreadRequest]) -> Result<SpreadsResponse> {
        let request = self
            .client()
            .request(Method::POST, format!("{}spreads", self.host()))
            .json(requests)
            .build()?;

        self.request(request, None).await
    }

    pub async fn tick_size(&self, token_id: &str) -> Result<TickSizeResponse> {
        if let Some(tick_size) = self.inner.tick_sizes.get(token_id) {
            #[cfg(feature = "tracing")]
            tracing::trace!(token_id = %token_id, tick_size = ?tick_size.value(), "cache hit: tick_size");
            return Ok(TickSizeResponse {
                minimum_tick_size: *tick_size,
            });
        }

        #[cfg(feature = "tracing")]
        tracing::trace!(token_id = %token_id, "cache miss: tick_size");

        let request = self
            .client()
            .request(Method::GET, format!("{}tick-size", self.host()))
            .query(&[("token_id", token_id)])
            .build()?;

        let response = self.request::<TickSizeResponse>(request, None).await?;

        self.inner
            .tick_sizes
            .insert(token_id.to_owned(), response.minimum_tick_size);

        #[cfg(feature = "tracing")]
        tracing::trace!(token_id = %token_id, "cached tick_size");

        Ok(response)
    }

    pub async fn neg_risk(&self, token_id: &str) -> Result<NegRiskResponse> {
        if let Some(neg_risk) = self.inner.neg_risk.get(token_id) {
            #[cfg(feature = "tracing")]
            tracing::trace!(token_id = %token_id, neg_risk = *neg_risk, "cache hit: neg_risk");
            return Ok(NegRiskResponse {
                neg_risk: *neg_risk,
            });
        }

        #[cfg(feature = "tracing")]
        tracing::trace!(token_id = %token_id, "cache miss: neg_risk");

        let request = self
            .client()
            .request(Method::GET, format!("{}neg-risk", self.host()))
            .query(&[("token_id", token_id)])
            .build()?;

        let response = self.request::<NegRiskResponse>(request, None).await?;

        self.inner
            .neg_risk
            .insert(token_id.to_owned(), response.neg_risk);

        #[cfg(feature = "tracing")]
        tracing::trace!(token_id = %token_id, "cached neg_risk");

        Ok(response)
    }

    pub async fn fee_rate_bps(&self, token_id: &str) -> Result<FeeRateResponse> {
        if let Some(base_fee) = self.inner.fee_rate_bps.get(token_id) {
            #[cfg(feature = "tracing")]
            tracing::trace!(token_id = %token_id, base_fee = *base_fee, "cache hit: fee_rate_bps");
            return Ok(FeeRateResponse {
                base_fee: *base_fee,
            });
        }

        #[cfg(feature = "tracing")]
        tracing::trace!(token_id = %token_id, "cache miss: fee_rate_bps");

        let request = self
            .client()
            .request(Method::GET, format!("{}fee-rate", self.host()))
            .query(&[("token_id", token_id)])
            .build()?;

        let response = self.request::<FeeRateResponse>(request, None).await?;

        self.inner
            .fee_rate_bps
            .insert(token_id.to_owned(), response.base_fee);

        #[cfg(feature = "tracing")]
        tracing::trace!(token_id = %token_id, "cached fee_rate_bps");

        Ok(response)
    }

    pub async fn order_book(
        &self,
        request: &OrderBookSummaryRequest,
    ) -> Result<OrderBookSummaryResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}book", self.host()))
            .query(&[("token_id", request.token_id.as_str())])
            .build()?;

        self.request(request, None).await
    }

    pub async fn order_books(
        &self,
        requests: &[OrderBookSummaryRequest],
    ) -> Result<Vec<OrderBookSummaryResponse>> {
        let request = self
            .client()
            .request(Method::POST, format!("{}books", self.host()))
            .json(requests)
            .build()?;

        self.request(request, None).await
    }

    pub async fn last_trade_price(
        &self,
        request: &LastTradePriceRequest,
    ) -> Result<LastTradePriceResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}last-trade-price", self.host()))
            .query(&[("token_id", request.token_id.as_str())])
            .build()?;

        self.request(request, None).await
    }

    pub async fn last_trades_prices(
        &self,
        token_ids: &[LastTradePriceRequest],
    ) -> Result<Vec<LastTradesPricesResponse>> {
        let request = self
            .client()
            .request(Method::GET, format!("{}last-trades-prices", self.host()))
            .json(token_ids)
            .build()?;

        self.request(request, None).await
    }

    pub async fn market(&self, condition_id: &str) -> Result<MarketResponse> {
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}markets/{condition_id}", self.host()),
            )
            .build()?;

        self.request(request, None).await
    }

    pub async fn markets(&self, next_cursor: Option<String>) -> Result<Page<MarketResponse>> {
        let cursor = next_cursor.map_or(String::new(), |c| format!("?next_cursor={c}"));
        let request = self
            .client()
            .request(Method::GET, format!("{}markets{cursor}", self.host()))
            .build()?;

        self.request(request, None).await
    }

    pub async fn sampling_markets(
        &self,
        next_cursor: Option<String>,
    ) -> Result<Page<MarketResponse>> {
        let cursor = next_cursor.map_or(String::new(), |c| format!("?next_cursor={c}"));
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}sampling-markets{cursor}", self.host()),
            )
            .build()?;

        self.request(request, None).await
    }

    pub async fn simplified_markets(
        &self,
        next_cursor: Option<String>,
    ) -> Result<Page<SimplifiedMarketResponse>> {
        let cursor = next_cursor.map_or(String::new(), |c| format!("?next_cursor={c}"));
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}simplified-markets{cursor}", self.host()),
            )
            .build()?;

        self.request(request, None).await
    }

    pub async fn sampling_simplified_markets(
        &self,
        next_cursor: Option<String>,
    ) -> Result<Page<SimplifiedMarketResponse>> {
        let cursor = next_cursor.map_or(String::new(), |c| format!("?next_cursor={c}"));
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}sampling-simplified-markets{cursor}", self.host()),
            )
            .build()?;

        self.request(request, None).await
    }

    /// Returns a stream of results, using `self` to repeatedly invoke the provided closure,
    /// `call`, which takes the next cursor to query against. Each `call` returns a future
    /// that returns a [`Page<Data>`]. Each page is flattened into the underlying data in the stream.
    pub fn stream_data<'client, Call, Fut, Data>(
        &'client self,
        call: Call,
    ) -> impl Stream<Item = Result<Data>> + 'client
    where
        Call: Fn(&'client Client<S>, Option<String>) -> Fut + 'client,
        Fut: Future<Output = Result<Page<Data>>> + 'client,
        Data: 'client,
    {
        try_stream! {
            let mut cursor: Option<String> = None;

            loop {
                let page = call(self, mem::take(&mut cursor)).await?;

                for item in page.data {
                    yield item
                }

                if page.next_cursor == TERMINAL_CURSOR {
                    break;
                }

                cursor = Some(page.next_cursor);
            }
        }
    }

    async fn request<Response: DeserializeOwned>(
        &self,
        request: Request,
        headers: Option<HeaderMap>,
    ) -> Result<Response> {
        self.inner.request(request, headers).await
    }

    fn client(&self) -> &ReqwestClient {
        &self.inner.client
    }
}

impl Client<Unauthenticated> {
    pub fn new(host: &str, config: Config) -> Result<Client<Unauthenticated>> {
        let mut headers = HeaderMap::new();

        headers.insert("User-Agent", HeaderValue::from_static("rs_clob_client"));
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Connection", HeaderValue::from_static("keep-alive"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = ReqwestClient::builder().default_headers(headers).build()?;

        Ok(Self {
            inner: Arc::new(ClientInner {
                config,
                host: Url::parse(host)?,
                client,
                tick_sizes: DashMap::new(),
                neg_risk: DashMap::new(),
                fee_rate_bps: DashMap::new(),
                state: Unauthenticated,
                funder: None,
                signature_type: SignatureType::Eoa,
                salt_generator: generate_seed,
            }),
        })
    }

    pub fn authentication_builder<S: Signer>(
        self,
        signer: &S,
    ) -> AuthenticationBuilder<'_, S, Normal> {
        AuthenticationBuilder {
            signer,
            credentials: None,
            nonce: None,
            kind: Normal,
            funder: self.inner.funder,
            signature_type: Some(self.inner.signature_type),
            client: self,
            salt_generator: None,
        }
    }

    /// Attempts to create a new set of [`Credentials`] and returns an error if there already is one
    /// for the particular L2 header's (signer) `address` and `nonce`.
    pub async fn create_api_key<S: Signer>(
        &self,
        signer: &S,
        nonce: Option<u32>,
    ) -> Result<Credentials> {
        self.inner.create_api_key(signer, nonce).await
    }

    /// Attempts to derive an existing set of [`Credentials`] and returns an error if there
    /// are none for the particular L2 header's (signer) `address` and `nonce`.
    pub async fn derive_api_key<S: Signer>(
        &self,
        signer: &S,
        nonce: Option<u32>,
    ) -> Result<Credentials> {
        self.inner.derive_api_key(signer, nonce).await
    }

    /// Idempotent alternative to [`Self::create_api_key`] and [`Self::derive_api_key`], which will
    /// either create a new set of [`Credentials`] if they do not exist already, or return them if
    /// they do.
    pub async fn create_or_derive_api_key<S: Signer>(
        &self,
        signer: &S,
        nonce: Option<u32>,
    ) -> Result<Credentials> {
        self.inner.create_or_derive_api_key(signer, nonce).await
    }
}

impl<K: Kind> Client<Authenticated<K>> {
    /// Demotes this authenticated [`Client<Authenticated<K>>`] to an unauthenticated one
    pub fn deauthenticate(self) -> Result<Client<Unauthenticated>> {
        let inner = Arc::into_inner(self.inner).ok_or(Synchronization)?;
        Ok(Client::<Unauthenticated> {
            inner: Arc::new(ClientInner {
                state: Unauthenticated,
                host: inner.host,
                config: inner.config,
                client: inner.client,
                tick_sizes: inner.tick_sizes,
                neg_risk: inner.neg_risk,
                fee_rate_bps: inner.fee_rate_bps,
                // Reset the order parameters that were previously stored on the client
                funder: None,
                signature_type: SignatureType::Eoa,
                salt_generator: generate_seed,
            }),
        })
    }

    #[must_use]
    pub fn state(&self) -> &Authenticated<K> {
        &self.inner.state
    }

    #[must_use]
    pub fn address(&self) -> Address {
        self.state().address
    }

    /// Return all API keys associated with the address corresponding to the inner signer in
    /// [`Authenticated<K>`].
    pub async fn api_keys(&self) -> Result<ApiKeysResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}auth/api-keys", self.host()))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn delete_api_key(&self) -> Result<serde_json::Value> {
        let request = self
            .client()
            .request(Method::DELETE, format!("{}auth/api-key", self.host()))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn closed_only_mode(&self) -> Result<BanStatusResponse> {
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}auth/ban-status/closed-only", self.host()),
            )
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    /// Creates an [`OrderBuilder<Limit, K>`] used to construct a limit order.
    #[must_use]
    pub fn limit_order(&self) -> OrderBuilder<Limit, K> {
        self.order_builder()
    }

    /// Creates an [`OrderBuilder<Market, K>`] used to construct a market order.
    #[must_use]
    pub fn market_order(&self) -> OrderBuilder<Market, K> {
        self.order_builder()
    }

    /// Attempts to sign the provided [`SignableOrder`] using the inner signer of [`Authenticated<K>`]
    #[expect(
        clippy::missing_panics_doc,
        reason = "No need to publicly document as we are guarded by the typestate pattern. \
        We cannot call `sign` without first calling `authenticate`"
    )]
    pub async fn sign<S: Signer>(
        &self,
        signer: &S,
        SignableOrder { order, order_type }: SignableOrder,
    ) -> Result<SignedOrder> {
        let token_id = order.tokenId.to_string();
        let neg_risk = self.neg_risk(&token_id).await?.neg_risk;
        let chain_id = signer
            .chain_id()
            .expect("Validated not none in `authenticate`");

        let exchange_contract = contract_config(chain_id, neg_risk)
            .ok_or(Error::missing_contract_config(chain_id, neg_risk))?
            .exchange;

        let domain = Eip712Domain {
            name: ORDER_NAME,
            version: VERSION,
            chain_id: Some(U256::from(chain_id)),
            verifying_contract: Some(exchange_contract),
            ..Eip712Domain::default()
        };

        let signature = signer
            .sign_hash(&order.eip712_signing_hash(&domain))
            .await?;

        Ok(SignedOrder {
            order,
            signature,
            order_type,
            owner: self.state().credentials.key,
        })
    }

    pub async fn post_order(&self, order: SignedOrder) -> Result<Vec<PostOrderResponse>> {
        self.post_orders(vec![order]).await
    }

    pub async fn post_orders(&self, orders: Vec<SignedOrder>) -> Result<Vec<PostOrderResponse>> {
        let request = self
            .client()
            .request(Method::POST, format!("{}orders", self.host()))
            .json(&orders)
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    /// Attempts to return the corresponding order at the provided `order_id`
    pub async fn order(&self, order_id: &str) -> Result<OpenOrderResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}data/order/{order_id}", self.host()))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn orders(
        &self,
        request: &OrdersRequest,
        next_cursor: Option<String>,
    ) -> Result<Page<OpenOrderResponse>> {
        let params = request.as_params(next_cursor.as_ref());
        let request = self
            .client()
            .request(Method::GET, format!("{}data/orders{params}", self.host()))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrdersResponse> {
        let request = self
            .client()
            .request(Method::DELETE, format!("{}order", self.host()))
            .json(&json!({ "orderId": order_id }))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn cancel_orders(&self, order_ids: &[&str]) -> Result<CancelOrdersResponse> {
        let request = self
            .client()
            .request(Method::DELETE, format!("{}orders", self.host()))
            .json(&json!(order_ids))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn cancel_all_orders(&self) -> Result<CancelOrdersResponse> {
        let request = self
            .client()
            .request(Method::DELETE, format!("{}cancel-all", self.host()))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    /// Attempts to cancel all open orders for a particular [`CancelMarketOrderRequest::market`]
    /// and/or [`CancelMarketOrderRequest::asset_id`]
    pub async fn cancel_market_orders(
        &self,
        request: &CancelMarketOrderRequest,
    ) -> Result<CancelOrdersResponse> {
        let request = self
            .client()
            .request(
                Method::DELETE,
                format!("{}cancel-market-orders", self.host()),
            )
            .json(&request)
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn trades(
        &self,
        request: &TradesRequest,
        next_cursor: Option<String>,
    ) -> Result<Page<TradeResponse>> {
        let params = request.as_params(next_cursor.as_ref());
        let request = self
            .client()
            .request(Method::GET, format!("{}data/trades{params}", self.host()))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn notifications(&self) -> Result<Vec<NotificationResponse>> {
        let request = self
            .client()
            .request(Method::GET, format!("{}notifications", self.host()))
            .query(&[("signature_type", self.inner.signature_type as u8)])
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn delete_notifications(&self, request: &DeleteNotificationsRequest) -> Result<()> {
        let params = request.as_params();
        let mut request = self
            .client()
            .request(
                Method::DELETE,
                format!("{}notifications{params}", self.host()),
            )
            .json(&request)
            .build()?;
        let headers = self.create_headers(&request).await?;
        *request.headers_mut() = headers;

        // We have to send the request separately from `self.request` because this endpoint does
        // not return anything in the response body. Otherwise, we would get an EOF error from reqwest
        self.client().execute(request).await?;

        Ok(())
    }

    pub async fn balance_allowance(
        &self,
        request: &BalanceAllowanceRequest,
    ) -> Result<BalanceAllowanceResponse> {
        let params = request.as_params(self.inner.signature_type);
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}balance-allowance{params}", self.host()),
            )
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn update_balance_allowance(
        &self,
        request: &UpdateBalanceAllowanceRequest,
    ) -> Result<()> {
        let params = request.as_params(self.inner.signature_type);
        let mut request = self
            .client()
            .request(
                Method::GET,
                format!("{}balance-allowance/update{params}", self.host()),
            )
            .build()?;
        let headers = self.create_headers(&request).await?;

        *request.headers_mut() = headers;

        // We have to send the request separately from `self.request` because this endpoint does
        // not return anything in the response body. Otherwise, we would get an EOF error from reqwest
        self.client().execute(request).await?;

        Ok(())
    }

    pub async fn is_order_scoring(&self, order_id: &str) -> Result<OrderScoringResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}order-scoring", self.host()))
            .query(&[("order_id", order_id)])
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn are_orders_scoring(&self, order_ids: &[&str]) -> Result<OrdersScoringResponse> {
        let request = self
            .client()
            .request(Method::GET, format!("{}orders-scoring", self.host()))
            .json(&order_ids)
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn earnings_for_user_for_day(
        &self,
        date: NaiveDate,
        next_cursor: Option<String>,
    ) -> Result<Page<UserEarningResponse>> {
        let cursor = next_cursor.map_or(String::new(), |c| format!("&next_cursor={c}"));
        let request = self
            .client()
            .request(Method::GET, format!("{}rewards/user{cursor}", self.host()))
            .query(&[
                ("date", date.to_string()),
                (
                    "signature_type",
                    (self.inner.signature_type as u8).to_string(),
                ),
            ])
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn total_earnings_for_user_for_day(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<TotalUserEarningResponse>> {
        let request = self
            .client()
            .request(Method::GET, format!("{}rewards/user/total", self.host()))
            .query(&[
                ("date", date.to_string()),
                (
                    "signature_type",
                    (self.inner.signature_type as u8).to_string(),
                ),
            ])
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn user_earnings_and_markets_config(
        &self,
        request: &UserRewardsEarningRequest,
        next_cursor: Option<String>,
    ) -> Result<Vec<UserRewardsEarningResponse>> {
        let params = request.as_params(next_cursor.as_ref());
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}rewards/user/total{params}", self.host()),
            )
            .query(&[(
                "signature_type",
                (self.inner.signature_type as u8).to_string(),
            )])
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn reward_percentages(&self) -> Result<RewardsPercentagesResponse> {
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}rewards/user/percentages", self.host()),
            )
            .query(&[(
                "signature_type",
                (self.inner.signature_type as u8).to_string(),
            )])
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn current_rewards(
        &self,
        next_cursor: Option<String>,
    ) -> Result<Page<CurrentRewardResponse>> {
        let cursor = next_cursor.map_or(String::new(), |c| format!("&next_cursor={c}"));
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}rewards/markets/current{cursor}", self.host()),
            )
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn raw_rewards_for_market(
        &self,
        condition_id: &str,
        next_cursor: Option<String>,
    ) -> Result<Page<MarketRewardResponse>> {
        let cursor = next_cursor.map_or(String::new(), |c| format!("?next_cursor={c}"));
        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}rewards/markets/{condition_id}{cursor}", self.host()),
            )
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn create_builder_api_key(&self) -> Result<Credentials> {
        let request = self
            .client()
            .request(Method::POST, format!("{}auth/builder-api-key", self.host()))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    async fn create_headers(&self, request: &Request) -> Result<HeaderMap> {
        let timestamp = if self.inner.config.use_server_time {
            self.server_time().await?
        } else {
            Utc::now().timestamp()
        };

        auth::l2::create_headers(self.state(), request, timestamp).await
    }

    fn order_builder<OrderKind>(&self) -> OrderBuilder<OrderKind, K> {
        OrderBuilder {
            signer: self.address(),
            signature_type: self.inner.signature_type,
            funder: self.inner.funder,
            salt_generator: self.inner.salt_generator,
            token_id: None,
            price: None,
            size: None,
            amount: None,
            side: None,
            nonce: None,
            expiration: None,
            taker: None,
            order_type: None,
            client: Client {
                inner: Arc::clone(&self.inner),
            },
            _kind: PhantomData,
        }
    }
}

impl Client<Authenticated<Normal>> {
    pub fn promote_to_builder(
        self,
        config: BuilderConfig,
    ) -> Result<Client<Authenticated<Builder>>> {
        let inner = Arc::into_inner(self.inner).ok_or(Synchronization)?;

        let state = Authenticated {
            address: inner.state.address,
            credentials: inner.state.credentials,
            kind: Builder {
                config,
                client: inner.client.clone(),
            },
        };

        let new_inner = ClientInner {
            config: inner.config,
            state,
            host: inner.host,
            client: inner.client,
            tick_sizes: inner.tick_sizes,
            neg_risk: inner.neg_risk,
            fee_rate_bps: inner.fee_rate_bps,
            funder: inner.funder,
            signature_type: inner.signature_type,
            salt_generator: inner.salt_generator,
        };

        Ok(Client {
            inner: Arc::new(new_inner),
        })
    }
}

impl Client<Authenticated<Builder>> {
    pub async fn builder_api_keys(&self) -> Result<Vec<BuilderApiKeyResponse>> {
        let request = self
            .client()
            .request(Method::GET, format!("{}auth/builder-api-key", self.host()))
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }

    pub async fn revoke_builder_api_key(&self) -> Result<()> {
        let mut request = self
            .client()
            .request(
                Method::DELETE,
                format!("{}auth/builder-api-key", self.host()),
            )
            .build()?;
        let headers = self.create_headers(&request).await?;

        *request.headers_mut() = headers;

        // We have to send the request separately from `self.request` because this endpoint does
        // not return anything in the response body. Otherwise, we would get an EOF error from reqwest
        self.client().execute(request).await?;

        Ok(())
    }

    pub async fn builder_trades(
        &self,
        request: &TradesRequest,
        next_cursor: Option<String>,
    ) -> Result<Page<BuilderTradeResponse>> {
        let params = request.as_params(next_cursor.as_ref());

        let request = self
            .client()
            .request(
                Method::GET,
                format!("{}builder/trades{params}", self.host()),
            )
            .build()?;
        let headers = self.create_headers(&request).await?;

        self.request(request, Some(headers)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_default_should_succeed() {
        _ = Client::default();
    }
}
