#![expect(
    clippy::module_name_repetitions,
    reason = "Public WebSocket types intentionally include the module name for clarity"
)]

use std::sync::Arc;

use alloy::primitives::Address;
use async_stream::stream;
use futures::Stream;
use futures::StreamExt as _;
use rust_decimal_macros::dec;

use super::config::WebSocketConfig;
use super::connection::{ConnectionManager, ConnectionState};
use super::messages::{
    AuthPayload, BookUpdate, MidpointUpdate, OrderMessage, PriceChange, TradeMessage, WsMessage,
};
use super::subscription::SubscriptionManager;
use crate::Result;
use crate::auth::{Credentials, Kind as AuthKind, Normal};
use crate::clob::state::{Authenticated, State, Unauthenticated};
use crate::error::{Error, Synchronization};

/// WebSocket client for real-time market data and user updates.
///
/// This client uses a type-state pattern to enforce authentication requirements at compile time:
/// - [`WebSocketClient<Unauthenticated>`]: Can only access public market data
/// - [`WebSocketClient<Authenticated<K>>`]: Can access both public and user-specific data
///
/// # Examples
///
/// ```no_run
/// use polymarket_client_sdk::ws::{WebSocketClient, WebSocketConfig};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create unauthenticated client
///     let client = WebSocketClient::new(
///         "wss://ws-subscriptions-clob.polymarket.com/ws/market",
///         WebSocketConfig::default()
///     )?;
///
///     // Subscribe to orderbook updates
///     let mut stream = client
///         .subscribe_orderbook(vec!["asset123".to_owned()])
///         .await?;
///
///     while let Some(book) = stream.next().await {
///         println!("Orderbook: {:?}", book?);
///     }
///
///     Ok(())
/// }
/// ```
pub struct WebSocketClient<S: State = Unauthenticated> {
    inner: Arc<WsClientInner<S>>,
}

struct WsClientInner<S: State> {
    /// Current state of the client (authenticated or unauthenticated)
    state: S,
    /// Configuration for the WebSocket connection
    config: WebSocketConfig,
    /// Connection manager handling the WebSocket connection
    connection: Arc<ConnectionManager>,
    /// Subscription manager for handling active subscriptions
    subscriptions: Arc<SubscriptionManager>,
}

impl WebSocketClient<Unauthenticated> {
    /// Create a new unauthenticated WebSocket client.
    pub fn new(endpoint: &str, config: WebSocketConfig) -> Result<Self> {
        let connection = ConnectionManager::new(endpoint.to_owned(), config.clone())?;
        let connection = Arc::new(connection);
        let subscriptions = SubscriptionManager::new(Arc::clone(&connection));

        Ok(Self {
            inner: Arc::new(WsClientInner {
                state: Unauthenticated,
                config,
                connection,
                subscriptions: Arc::new(subscriptions),
            }),
        })
    }

    /// Authenticate this client and elevate to authenticated state.
    ///
    /// Returns an error if another thread is currently authenticating or deauthenticating.
    pub fn authenticate(
        self,
        credentials: Credentials,
        address: Address,
    ) -> Result<WebSocketClient<Authenticated<Normal>>> {
        let inner = Arc::try_unwrap(self.inner).map_err(|_e| Synchronization)?;

        Ok(WebSocketClient {
            inner: Arc::new(WsClientInner {
                state: Authenticated {
                    address,
                    credentials,
                    kind: Normal,
                },
                config: inner.config,
                connection: inner.connection,
                subscriptions: inner.subscriptions,
            }),
        })
    }
}

// Methods available in any state
impl<S: State> WebSocketClient<S> {
    /// Subscribe to orderbook updates for specific assets.
    pub fn subscribe_orderbook(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<BookUpdate>>> {
        let stream = self.inner.subscriptions.subscribe_market(asset_ids)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::Book(book)) => Some(Ok(book)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Subscribe to price changes for specific assets.
    pub fn subscribe_prices(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<PriceChange>>> {
        let stream = self.inner.subscriptions.subscribe_market(asset_ids)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::PriceChange(price)) => Some(Ok(price)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Subscribe to midpoint updates (calculated from best bid/ask).
    pub fn subscribe_midpoints(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<MidpointUpdate>>> {
        let stream = self.subscribe_orderbook(asset_ids)?;

        Ok(stream! {
            for await book_result in stream {
                match book_result {
                    Ok(book) => {
                        // Calculate midpoint from best bid/ask
                        let best_bid = book.bids.first();
                        let best_ask = book.asks.first();

                        match (best_bid, best_ask) {
                            (Some(bid), Some(ask)) => {
                                let midpoint = (bid.price + ask.price) / dec!(2);
                                yield Ok(MidpointUpdate {
                                    asset_id: book.asset_id,
                                    market: book.market,
                                    midpoint,
                                    timestamp: book.timestamp,
                                });
                            }
                            _ => {
                                yield Err(Error::validation("No bid or ask available for midpoint"));
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                    }
                }
            }
        })
    }

    /// Get the current connection state.
    pub async fn connection_state(&self) -> ConnectionState {
        self.inner.connection.state().await
    }

    /// Get the number of active subscriptions.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.inner.subscriptions.subscription_count()
    }
}

// Methods only available for authenticated clients
impl<K: AuthKind> WebSocketClient<Authenticated<K>> {
    /// Subscribe to user's order updates.
    pub fn subscribe_orders(
        &self,
        markets: Vec<String>,
    ) -> Result<impl Stream<Item = Result<OrderMessage>>> {
        let auth = AuthPayload {
            api_key: self.inner.state.credentials.key.to_string(),
            secret: self.inner.state.credentials.secret.reveal().clone(),
            passphrase: self.inner.state.credentials.passphrase.reveal().clone(),
        };

        let stream = self.inner.subscriptions.subscribe_user(markets, auth)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::Order(order)) => Some(Ok(order)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Subscribe to user's trade executions.
    pub fn subscribe_trades(
        &self,
        markets: Vec<String>,
    ) -> Result<impl Stream<Item = Result<TradeMessage>>> {
        let auth = AuthPayload {
            api_key: self.inner.state.credentials.key.to_string(),
            secret: self.inner.state.credentials.secret.reveal().clone(),
            passphrase: self.inner.state.credentials.passphrase.reveal().clone(),
        };

        let stream = self.inner.subscriptions.subscribe_user(markets, auth)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::Trade(trade)) => Some(Ok(trade)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Deauthenticate and return to unauthenticated state.
    pub fn deauthenticate(self) -> Result<WebSocketClient<Unauthenticated>> {
        let inner = Arc::try_unwrap(self.inner).map_err(|_e| Synchronization)?;

        Ok(WebSocketClient {
            inner: Arc::new(WsClientInner {
                state: Unauthenticated,
                config: inner.config,
                connection: inner.connection,
                subscriptions: inner.subscriptions,
            }),
        })
    }
}

impl<S: State> Clone for WebSocketClient<S> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
