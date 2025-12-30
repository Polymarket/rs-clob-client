use std::sync::Arc;

use alloy::primitives::Address;
use futures::Stream;
use futures::StreamExt as _;

use super::config::RtdsConfig;
use super::connection::{ConnectionManager, ConnectionState};
use super::interest::InterestTracker;
use super::subscription::SubscriptionManager;
use super::types::{ChainlinkPrice, Comment, CommentType, CryptoPrice, RtdsMessage, Subscription};
use crate::Result;
use crate::auth::Credentials;
use crate::auth::state::{Authenticated, State, Unauthenticated};
use crate::auth::{Kind as AuthKind, Normal};
use crate::error::Error;

/// RTDS (Real-Time Data Socket) client for streaming Polymarket data.
///
/// This client uses a type-state pattern to enforce authentication requirements at compile time:
/// - [`Client<Unauthenticated>`]: Can only access public data streams
/// - [`Client<Authenticated<K>>`]: Can access both public and authenticated data streams
///
/// # Examples
///
/// ```rust, no_run
/// use polymarket_client_sdk::rtds::Client;
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = Client::default();
///
///     // Subscribe to BTC and ETH prices from Binance
///     let symbols = vec!["btcusdt".to_owned(), "ethusdt".to_owned()];
///     let stream = client.subscribe_crypto_prices(Some(symbols))?;
///     let mut stream = Box::pin(stream);
///
///     while let Some(price) = stream.next().await {
///         println!("Price: {:?}", price?);
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct Client<S: State = Unauthenticated> {
    inner: Arc<ClientInner<S>>,
}

struct ClientInner<S: State> {
    /// Current state of the client (authenticated or unauthenticated)
    state: S,
    /// Configuration for the RTDS connection
    config: RtdsConfig,
    /// Base endpoint for the WebSocket
    endpoint: String,
    /// Connection manager for the WebSocket
    connection: ConnectionManager,
    /// Subscription manager for handling subscriptions
    subscriptions: Arc<SubscriptionManager>,
}

impl Default for Client<Unauthenticated> {
    fn default() -> Self {
        Self::new("wss://ws-live-data.polymarket.com", RtdsConfig::default())
            .expect("RTDS client with default endpoint should succeed")
    }
}

impl Client<Unauthenticated> {
    /// Create a new unauthenticated RTDS client with the specified endpoint and configuration.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The WebSocket URL for the RTDS service
    /// * `config` - Configuration for connection behavior (heartbeat, reconnection, etc.)
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// use polymarket_client_sdk::rtds::{Client, RtdsConfig};
    ///
    /// // Use default configuration
    /// let client = Client::new("wss://ws-live-data.polymarket.com", RtdsConfig::default())?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(endpoint: &str, config: RtdsConfig) -> Result<Self> {
        let interest = Arc::new(InterestTracker::new());
        let connection = ConnectionManager::new(endpoint.to_owned(), config.clone(), &interest)?;
        let subscriptions = Arc::new(SubscriptionManager::new(connection.clone(), interest));

        // Start reconnection handler to re-subscribe on connection recovery
        subscriptions.start_reconnection_handler();

        Ok(Self {
            inner: Arc::new(ClientInner {
                state: Unauthenticated,
                config,
                endpoint: endpoint.to_owned(),
                connection,
                subscriptions,
            }),
        })
    }

    /// Authenticate this client and elevate to authenticated state.
    ///
    /// Returns an error if there are other references to this client (e.g., from clones).
    /// Ensure all clones are dropped before calling this method.
    ///
    /// # Arguments
    ///
    /// * `credentials` - CLOB API credentials (key, secret, passphrase)
    /// * `address` - User's wallet address
    pub fn authenticate(
        self,
        credentials: Credentials,
        address: Address,
    ) -> Result<Client<Authenticated<Normal>>> {
        let inner = Arc::into_inner(self.inner).ok_or(Error::validation(
            "Cannot authenticate while other references to this client exist; \
                 drop all clones before calling authenticate",
        ))?;

        let ClientInner {
            config,
            endpoint,
            connection,
            subscriptions,
            ..
        } = inner;

        Ok(Client {
            inner: Arc::new(ClientInner {
                state: Authenticated {
                    address,
                    credentials,
                    kind: Normal,
                },
                config,
                endpoint,
                connection,
                subscriptions,
            }),
        })
    }
}

// Methods available in any state
impl<S: State> Client<S> {
    /// Subscribe to Binance cryptocurrency price updates.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Optional list of trading pairs to filter (e.g., `["btcusdt", "ethusdt"]`).
    ///   If `None`, receives all available price updates.
    ///
    /// # Returns
    ///
    /// A stream of [`CryptoPrice`] updates.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// use polymarket_client_sdk::rtds::Client;
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = Client::default();
    ///
    /// // Subscribe to specific symbols
    /// let stream = client.subscribe_crypto_prices(Some(vec![
    ///     "btcusdt".to_owned(),
    ///     "ethusdt".to_owned(),
    /// ]))?;
    ///
    /// // Or subscribe to all prices
    /// let stream = client.subscribe_crypto_prices(None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_crypto_prices(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<impl Stream<Item = Result<CryptoPrice>>> {
        let subscription = Subscription::crypto_prices(symbols);
        let stream = self.inner.subscriptions.subscribe(subscription)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(msg) => msg.as_crypto_price().map(Ok),
                Err(e) => Some(Err(e)),
            }
        }))
    }

    /// Subscribe to Chainlink price feed updates.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Optional trading pair to filter (e.g., `"eth/usd"`).
    ///   If `None`, receives all available Chainlink price updates.
    ///
    /// # Returns
    ///
    /// A stream of [`ChainlinkPrice`] updates.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// use polymarket_client_sdk::rtds::Client;
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = Client::default();
    ///
    /// // Subscribe to ETH/USD price feed
    /// let stream = client.subscribe_chainlink_prices(Some("eth/usd".to_owned()))?;
    ///
    /// // Or subscribe to all Chainlink prices
    /// let stream = client.subscribe_chainlink_prices(None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_chainlink_prices(
        &self,
        symbol: Option<String>,
    ) -> Result<impl Stream<Item = Result<ChainlinkPrice>>> {
        let subscription = Subscription::chainlink_prices(symbol);
        let stream = self.inner.subscriptions.subscribe(subscription)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(msg) => msg.as_chainlink_price().map(Ok),
                Err(e) => Some(Err(e)),
            }
        }))
    }

    /// Subscribe to comment events.
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Optional comment event type to filter (e.g., `CommentType::CommentCreated`).
    ///   If `None`, receives all comment events.
    ///
    /// # Returns
    ///
    /// A stream of [`Comment`] events.
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// use polymarket_client_sdk::rtds::{Client, CommentType};
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = Client::default();
    ///
    /// // Subscribe to new comment events only
    /// let stream = client.subscribe_comments(Some(CommentType::CommentCreated))?;
    ///
    /// // Or subscribe to all comment events
    /// let stream = client.subscribe_comments(None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_comments(
        &self,
        comment_type: Option<CommentType>,
    ) -> Result<impl Stream<Item = Result<Comment>>> {
        let subscription = Subscription::comments(comment_type);
        let stream = self.inner.subscriptions.subscribe(subscription)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(msg) => msg.as_comment().map(Ok),
                Err(e) => Some(Err(e)),
            }
        }))
    }

    /// Subscribe to raw RTDS messages for a custom topic/type combination.
    ///
    /// This is a low-level method that allows subscribing to any topic/type
    /// combination and receiving the raw [`RtdsMessage`] objects.
    ///
    /// # Arguments
    ///
    /// * `subscription` - The subscription configuration
    ///
    /// # Examples
    ///
    /// ```rust, no_run
    /// use polymarket_client_sdk::rtds::{Client, Subscription};
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = Client::default();
    ///
    /// // Create a custom subscription
    /// let sub = Subscription::crypto_prices(None);
    /// let stream = client.subscribe_raw(sub)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_raw(
        &self,
        subscription: Subscription,
    ) -> Result<impl Stream<Item = Result<RtdsMessage>>> {
        self.inner.subscriptions.subscribe(subscription)
    }

    /// Get the current connection state.
    ///
    /// # Returns
    ///
    /// The current [`ConnectionState`] of the WebSocket connection.
    pub async fn connection_state(&self) -> ConnectionState {
        self.inner.connection.state().await
    }

    /// Get the number of active subscriptions.
    ///
    /// # Returns
    ///
    /// The count of active subscriptions managed by this client.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.inner.subscriptions.subscription_count()
    }
}

// Methods only available for authenticated clients
impl<K: AuthKind> Client<Authenticated<K>> {
    /// Subscribe to Binance cryptocurrency price updates with CLOB authentication.
    ///
    /// Uses the credentials stored in the client state.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Optional list of trading pairs to filter
    pub fn subscribe_crypto_prices_with_clob_auth(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<impl Stream<Item = Result<CryptoPrice>>> {
        let subscription = Subscription::crypto_prices(symbols)
            .with_clob_auth(self.inner.state.credentials.clone());
        let stream = self.inner.subscriptions.subscribe(subscription)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(msg) => msg.as_crypto_price().map(Ok),
                Err(e) => Some(Err(e)),
            }
        }))
    }

    /// Subscribe to Chainlink price feed updates with CLOB authentication.
    ///
    /// Uses the credentials stored in the client state.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Optional trading pair to filter
    pub fn subscribe_chainlink_prices_with_clob_auth(
        &self,
        symbol: Option<String>,
    ) -> Result<impl Stream<Item = Result<ChainlinkPrice>>> {
        let subscription = Subscription::chainlink_prices(symbol)
            .with_clob_auth(self.inner.state.credentials.clone());
        let stream = self.inner.subscriptions.subscribe(subscription)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(msg) => msg.as_chainlink_price().map(Ok),
                Err(e) => Some(Err(e)),
            }
        }))
    }

    /// Subscribe to comment events with Gamma authentication.
    ///
    /// Uses the address stored in the client state.
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Optional comment event type to filter
    pub fn subscribe_comments_with_gamma_auth(
        &self,
        comment_type: Option<CommentType>,
    ) -> Result<impl Stream<Item = Result<Comment>>> {
        let subscription =
            Subscription::comments(comment_type).with_gamma_auth(self.inner.state.address);
        let stream = self.inner.subscriptions.subscribe(subscription)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(msg) => msg.as_comment().map(Ok),
                Err(e) => Some(Err(e)),
            }
        }))
    }

    /// Deauthenticate and return to unauthenticated state.
    ///
    /// Returns an error if there are other references to this client (e.g., from clones).
    /// Ensure all clones are dropped before calling this method.
    pub fn deauthenticate(self) -> Result<Client<Unauthenticated>> {
        let inner = Arc::into_inner(self.inner).ok_or(Error::validation(
            "Cannot deauthenticate while other references to this client exist; \
                 drop all clones before calling deauthenticate",
        ))?;

        let ClientInner {
            config,
            endpoint,
            connection,
            subscriptions,
            ..
        } = inner;

        Ok(Client {
            inner: Arc::new(ClientInner {
                state: Unauthenticated,
                config,
                endpoint,
                connection,
                subscriptions,
            }),
        })
    }
}
