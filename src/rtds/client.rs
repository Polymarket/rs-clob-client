use std::sync::Arc;

use futures::Stream;
use futures::StreamExt as _;

use super::subscription::{SimpleParser, SubscriptionManager, TopicType};
use super::types::request::Subscription;
use super::types::response::{ChainlinkPrice, Comment, CommentType, CryptoPrice, RtdsMessage};
use crate::Result;
use crate::auth::state::{Authenticated, State, Unauthenticated};
use crate::auth::{Credentials, Normal};
use crate::error::Error;
use crate::types::Address;
use crate::ws::ConnectionManager;
use crate::ws::config::Config;
use crate::ws::connection::ConnectionState;
use crate::ws::task::AbortOnDrop;

/// RTDS (Real-Time Data Socket) client for streaming Polymarket data.
///
/// - [`Client<Unauthenticated>`]: All streams, comments without auth
/// - [`Client<Authenticated<Normal>>`]: All streams, comments with CLOB auth
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

impl Default for Client<Unauthenticated> {
    fn default() -> Self {
        Self::new("wss://ws-live-data.polymarket.com", Config::default())
            .expect("RTDS client with default endpoint should succeed")
    }
}

struct ClientInner<S: State> {
    /// Current state of the client
    state: S,
    /// Configuration for the RTDS connection
    config: Config,
    /// Base endpoint for the WebSocket
    endpoint: String,
    /// Connection manager for the WebSocket
    connection: ConnectionManager<RtdsMessage, SimpleParser>,
    /// Subscription manager for handling subscriptions
    subscriptions: Arc<SubscriptionManager>,
    /// Owns the reconnection task spawned by
    /// [`SubscriptionManager::start_reconnection_handler`]. The wrapper
    /// aborts the task on drop, which releases the strong
    /// `Arc<SubscriptionManager>` clone held by the task's future and
    /// breaks the reference cycle that would otherwise leak the whole
    /// client (task, WebSocket, subscription manager) for the lifetime
    /// of the process — see issue #325 and [`AbortOnDrop`].
    reconnect_handle: AbortOnDrop,
}

impl Client<Unauthenticated> {
    /// Create a new unauthenticated RTDS client with the specified endpoint and configuration.
    pub fn new(endpoint: &str, config: Config) -> Result<Self> {
        let connection = ConnectionManager::new(endpoint.to_owned(), config.clone(), SimpleParser)?;
        let subscriptions = Arc::new(SubscriptionManager::new(connection.clone()));

        // Start reconnection handler to re-subscribe on connection recovery.
        // The handle is retained in an `AbortOnDrop` so the task is
        // cancelled when the client is dropped — see the field docs.
        let reconnect_handle = AbortOnDrop::new(subscriptions.start_reconnection_handler());

        Ok(Self {
            inner: Arc::new(ClientInner {
                state: Unauthenticated,
                config,
                endpoint: endpoint.to_owned(),
                connection,
                subscriptions,
                reconnect_handle,
            }),
        })
    }

    /// Authenticate with CLOB credentials.
    ///
    /// Returns an authenticated client that can subscribe to comments with auth.
    pub fn authenticate(
        self,
        address: Address,
        credentials: Credentials,
    ) -> Result<Client<Authenticated<Normal>>> {
        let inner = Arc::into_inner(self.inner).ok_or(Error::validation(
            "Cannot authenticate while other references to this client exist",
        ))?;

        Ok(Client {
            inner: Arc::new(ClientInner {
                state: Authenticated {
                    address,
                    credentials,
                    kind: Normal,
                },
                config: inner.config,
                endpoint: inner.endpoint,
                connection: inner.connection,
                subscriptions: inner.subscriptions,
                reconnect_handle: inner.reconnect_handle,
            }),
        })
    }

    /// Subscribe to comment events (unauthenticated).
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Optional comment event type to filter
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
}

// Methods available in any state
impl<S: State> Client<S> {
    /// Subscribes to real-time cryptocurrency price updates from Binance.
    ///
    /// Returns a stream of cryptocurrency prices for the specified trading pairs.
    /// If no symbols are provided, subscribes to all available cryptocurrency pairs.
    /// Prices are sourced from Binance and updated in real-time.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Optional list of trading pair symbols (e.g., `["BTCUSDT", "ETHUSDT"]`).
    ///   If `None`, subscribes to all available pairs.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be created or the WebSocket
    /// connection fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_client_sdk::rtds::Client;
    /// use polymarket_client_sdk::ws::config::Config;
    /// use futures::StreamExt;
    /// use tokio::pin;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("wss://rtds.polymarket.com", Config::default())?;
    /// let stream = client.subscribe_crypto_prices(Some(vec!["BTCUSDT".to_string()]))?;
    ///
    /// pin!(stream);
    ///
    /// while let Some(price_result) = stream.next().await {
    ///     let price = price_result?;
    ///     println!("BTC Price: ${}", price.value);
    /// }
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

    /// Subscribe to raw RTDS messages for a custom topic/type combination.
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
    #[must_use]
    pub fn connection_state(&self) -> ConnectionState {
        self.inner.connection.state()
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

    /// Unsubscribe from Binance crypto price updates.
    ///
    /// This decrements the reference count for the `crypto_prices` topic. Only sends
    /// an unsubscribe request to the server when no other streams are using this topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the unsubscribe request fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use polymarket_client_sdk::rtds::Client;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::default();
    /// let _stream = client.subscribe_crypto_prices(None)?;
    /// // Later...
    /// client.unsubscribe_crypto_prices()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn unsubscribe_crypto_prices(&self) -> Result<()> {
        let topic = TopicType::new("crypto_prices".to_owned(), "update".to_owned());
        self.inner.subscriptions.unsubscribe(&[topic])
    }

    /// Unsubscribe from Chainlink price feed updates.
    ///
    /// This decrements the reference count for the chainlink topic. Only sends
    /// an unsubscribe request to the server when no other streams are using this topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the unsubscribe request fails.
    pub fn unsubscribe_chainlink_prices(&self) -> Result<()> {
        let topic = TopicType::new("crypto_prices_chainlink".to_owned(), "*".to_owned());
        self.inner.subscriptions.unsubscribe(&[topic])
    }

    /// Unsubscribe from comment events.
    ///
    /// # Arguments
    ///
    /// * `comment_type` - The comment type to unsubscribe from. Use `None` for wildcard (`*`).
    pub fn unsubscribe_comments(&self, comment_type: Option<CommentType>) -> Result<()> {
        let msg_type = comment_type.map_or("*".to_owned(), |t| {
            serde_json::to_string(&t)
                .ok()
                .and_then(|s| s.trim_matches('"').to_owned().into())
                .unwrap_or_else(|| "*".to_owned())
        });
        let topic = TopicType::new("comments".to_owned(), msg_type);
        self.inner.subscriptions.unsubscribe(&[topic])
    }
}

impl Client<Authenticated<Normal>> {
    /// Subscribe to comment events with CLOB authentication.
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Optional comment event type to filter
    pub fn subscribe_comments(
        &self,
        comment_type: Option<CommentType>,
    ) -> Result<impl Stream<Item = Result<Comment>>> {
        let subscription = Subscription::comments(comment_type)
            .with_clob_auth(self.inner.state.credentials.clone());
        let stream = self.inner.subscriptions.subscribe(subscription)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(msg) => msg.as_comment().map(Ok),
                Err(e) => Some(Err(e)),
            }
        }))
    }

    /// Deauthenticate and return to unauthenticated state.
    pub fn deauthenticate(self) -> Result<Client<Unauthenticated>> {
        let inner = Arc::into_inner(self.inner).ok_or(Error::validation(
            "Cannot deauthenticate while other references to this client exist",
        ))?;

        Ok(Client {
            inner: Arc::new(ClientInner {
                state: Unauthenticated,
                config: inner.config,
                endpoint: inner.endpoint,
                connection: inner.connection,
                subscriptions: inner.subscriptions,
                reconnect_handle: inner.reconnect_handle,
            }),
        })
    }
}

#[cfg(test)]
mod teardown_tests {
    //! RTDS client teardown regression tests for issue #325. These cover
    //! the `reconnect_handle` plumbing in `Client::new`,
    //! `Client::authenticate`, and `Client::deauthenticate` — each must
    //! forward the `AbortOnDrop` into the new `ClientInner` so the
    //! spawned task is still tied to the live client, and each must let
    //! the wrapper run its `Drop` (aborting the task) when the last
    //! client clone goes away.

    use std::sync::Weak;
    use std::time::Duration;

    use super::{Client, SubscriptionManager};
    use crate::auth::{Credentials, Uuid};
    use crate::types::Address;
    use crate::ws::config::Config;

    /// Resolves immediately and refuses TCP connections.
    const UNROUTABLE_ENDPOINT: &str = "ws://127.0.0.1:1";

    /// Dummy credentials for `authenticate` / `deauthenticate` round-trips.
    /// Only the struct shape matters — the test never hits the network.
    fn dummy_credentials() -> Credentials {
        Credentials::new(
            Uuid::nil(),
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_owned(),
            "passphrase".to_owned(),
        )
    }

    /// Dummy EOA used for the authenticated client state.
    fn dummy_address() -> Address {
        "0x0000000000000000000000000000000000000001"
            .parse()
            .expect("valid zero-ish address")
    }

    /// Poll the weak reference until the strong count drops to zero, with
    /// a generous timeout so a busy CI runner doesn't flake.
    async fn wait_for_drop(weak: &Weak<SubscriptionManager>) {
        let start = std::time::Instant::now();
        while weak.strong_count() != 0 && start.elapsed() < Duration::from_secs(2) {
            tokio::task::yield_now().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    #[tokio::test]
    async fn unauthenticated_client_drop_releases_subscription_manager() {
        let client = Client::new(UNROUTABLE_ENDPOINT, Config::default())
            .expect("Client::new should not fail for a well-formed endpoint string");

        let weak = std::sync::Arc::downgrade(&client.inner_subscriptions_for_test());

        drop(client);
        wait_for_drop(&weak).await;

        assert!(
            weak.upgrade().is_none(),
            "Unauthenticated RTDS Client leaked SubscriptionManager on drop: \
             strong_count={} (issue #325 regression)",
            weak.strong_count(),
        );
    }

    #[tokio::test]
    async fn authenticate_then_drop_releases_subscription_manager() {
        let client = Client::new(UNROUTABLE_ENDPOINT, Config::default()).expect("Client::new");

        let weak = std::sync::Arc::downgrade(&client.inner_subscriptions_for_test());

        let authenticated = client
            .authenticate(dummy_address(), dummy_credentials())
            .expect("authenticate should succeed when no extra clones exist");

        // `authenticate` moved the reconnect handle + subscription manager
        // into a new `ClientInner`, so the weak ref should still upgrade.
        assert!(
            weak.upgrade().is_some(),
            "authenticate prematurely dropped the SubscriptionManager"
        );

        drop(authenticated);
        wait_for_drop(&weak).await;

        assert!(
            weak.upgrade().is_none(),
            "Authenticated RTDS Client leaked SubscriptionManager on drop: \
             strong_count={} (issue #325 regression)",
            weak.strong_count(),
        );
    }

    #[tokio::test]
    async fn deauthenticate_preserves_reconnect_handle_then_drop_cleans_up() {
        let client = Client::new(UNROUTABLE_ENDPOINT, Config::default()).expect("Client::new");

        let weak = std::sync::Arc::downgrade(&client.inner_subscriptions_for_test());

        let authenticated = client
            .authenticate(dummy_address(), dummy_credentials())
            .expect("authenticate");

        // Round-trip through deauthenticate; the handle must be forwarded
        // into the new `ClientInner` so the task stays alive.
        let unauth = authenticated
            .deauthenticate()
            .expect("deauthenticate should succeed when no extra clones exist");

        // After the round-trip the manager is still reachable — nothing has
        // dropped yet.
        assert!(
            weak.upgrade().is_some(),
            "Round-tripping through authenticate/deauthenticate prematurely \
             dropped the SubscriptionManager"
        );

        drop(unauth);
        wait_for_drop(&weak).await;

        assert!(
            weak.upgrade().is_none(),
            "RTDS Client leaked SubscriptionManager after deauthenticate+drop: \
             strong_count={} (issue #325 regression)",
            weak.strong_count(),
        );
    }
}

// Test-only accessor: expose the inner Arc<SubscriptionManager> so the
// teardown tests can take a `Weak` without widening the public API. The
// field is module-private, so the accessor lives in the same file.
#[cfg(test)]
#[expect(
    clippy::multiple_inherent_impl,
    reason = "Test-only accessor kept isolated from the main public impl"
)]
impl<S: State> Client<S> {
    fn inner_subscriptions_for_test(&self) -> Arc<SubscriptionManager> {
        Arc::clone(&self.inner.subscriptions)
    }
}
