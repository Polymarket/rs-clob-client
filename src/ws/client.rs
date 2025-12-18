#![expect(
    clippy::module_name_repetitions,
    reason = "Public WebSocket types intentionally include the module name for clarity"
)]

use std::collections::{HashMap, hash_map::Entry};
use std::sync::Arc;

use alloy::primitives::Address;
use async_stream::stream;
use futures::Stream;
use futures::StreamExt as _;
use rust_decimal_macros::dec;

use super::config::WebSocketConfig;
use super::connection::{ConnectionManager, ConnectionState};
use super::messages::{
    BookUpdate, MidpointUpdate, OrderMessage, PriceChange, TradeMessage, WsMessage,
};
use super::subscription::{ChannelType, SubscriptionManager};
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
/// ```rust, no_run
/// use polymarket_client_sdk::ws::WebSocketClient;
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create unauthenticated client
///     let client = WebSocketClient::default();
///
///     let stream = client.subscribe_orderbook(vec!["asset_id".to_owned()])?;
///     let mut stream = Box::pin(stream);
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

impl Default for WebSocketClient<Unauthenticated> {
    fn default() -> Self {
        Self::new(
            "wss://ws-subscriptions-clob.polymarket.com",
            WebSocketConfig::default(),
        )
        .expect("WebSocket client with default endpoint should succeed")
    }
}

struct WsClientInner<S: State> {
    /// Current state of the client (authenticated or unauthenticated)
    state: S,
    /// Configuration for the WebSocket connections
    config: WebSocketConfig,
    /// Base endpoint without channel suffix (e.g. `wss://...`)
    base_endpoint: String,
    /// Resources for each WebSocket channel
    channels: HashMap<ChannelType, ChannelHandles>,
}

impl WebSocketClient<Unauthenticated> {
    /// Create a new unauthenticated WebSocket client.
    ///
    /// The `endpoint` should be the base WebSocket URL (e.g. `wss://...polymarket.com`);
    /// channel paths (`/ws/market` or `/ws/user`) are appended automatically.
    pub fn new(endpoint: &str, config: WebSocketConfig) -> Result<Self> {
        let normalized = normalize_base_endpoint(endpoint);
        let market_handles =
            ChannelHandles::connect(channel_endpoint(&normalized, ChannelType::Market), &config)?;
        let mut channels = HashMap::new();
        channels.insert(ChannelType::Market, market_handles);

        Ok(Self {
            inner: Arc::new(WsClientInner {
                state: Unauthenticated,
                config,
                base_endpoint: normalized,
                channels,
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
        let WsClientInner {
            config,
            base_endpoint,
            mut channels,
            ..
        } = inner;

        if let Entry::Vacant(slot) = channels.entry(ChannelType::User) {
            let handles = ChannelHandles::connect(
                channel_endpoint(&base_endpoint, ChannelType::User),
                &config,
            )?;
            slot.insert(handles);
        }

        Ok(WebSocketClient {
            inner: Arc::new(WsClientInner {
                state: Authenticated {
                    address,
                    credentials,
                    kind: Normal,
                },
                config,
                base_endpoint,
                channels,
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
        let stream = self
            .market_handles()?
            .subscriptions
            .subscribe_market(asset_ids)?;

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
        let stream = self
            .market_handles()?
            .subscriptions
            .subscribe_market(asset_ids)?;

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
        if let Some(handles) = self.inner.channel(ChannelType::Market) {
            handles.connection.state().await
        } else {
            ConnectionState::Disconnected
        }
    }

    /// Get the number of active subscriptions.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.inner
            .channels
            .values()
            .map(|handles| handles.subscriptions.subscription_count())
            .sum()
    }

    fn market_handles(&self) -> Result<&ChannelHandles> {
        self.inner
            .channel(ChannelType::Market)
            .ok_or_else(|| Error::validation("Market channel unavailable; recreate client"))
    }
}

// Methods only available for authenticated clients
impl<K: AuthKind> WebSocketClient<Authenticated<K>> {
    /// Subscribe to raw user channel events (orders and trades).
    pub fn subscribe_user_events(
        &self,
        markets: Vec<String>,
    ) -> Result<impl Stream<Item = Result<WsMessage>>> {
        let auth = self.inner.state.credentials.clone();

        let handles = self
            .inner
            .channel(ChannelType::User)
            .ok_or_else(|| Error::validation("User channel unavailable; authenticate first"))?;

        handles.subscriptions.subscribe_user(markets, auth)
    }

    /// Subscribe to user's order updates.
    pub fn subscribe_orders(
        &self,
        markets: Vec<String>,
    ) -> Result<impl Stream<Item = Result<OrderMessage>>> {
        let stream = self.subscribe_user_events(markets)?;

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
        let stream = self.subscribe_user_events(markets)?;

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
        let WsClientInner {
            config,
            base_endpoint,
            mut channels,
            ..
        } = inner;
        channels.remove(&ChannelType::User);

        Ok(WebSocketClient {
            inner: Arc::new(WsClientInner {
                state: Unauthenticated,
                config,
                base_endpoint,
                channels,
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

impl<S: State> WsClientInner<S> {
    fn channel(&self, kind: ChannelType) -> Option<&ChannelHandles> {
        self.channels.get(&kind)
    }
}

/// Handles for a specific WebSocket channel.
#[derive(Clone)]
struct ChannelHandles {
    /// Manages the WebSocket connection.
    connection: Arc<ConnectionManager>,
    /// Manages active subscriptions on this channel.
    subscriptions: Arc<SubscriptionManager>,
}

impl ChannelHandles {
    fn connect(endpoint: String, config: &WebSocketConfig) -> Result<Self> {
        let connection = Arc::new(ConnectionManager::new(endpoint, config.clone())?);
        let subscriptions = Arc::new(SubscriptionManager::new(Arc::clone(&connection)));

        Ok(Self {
            connection,
            subscriptions,
        })
    }
}

fn normalize_base_endpoint(endpoint: &str) -> String {
    let trimmed = endpoint.trim_end_matches('/');
    if let Some(stripped) = trimmed.strip_suffix("/ws/market") {
        stripped.to_owned()
    } else if let Some(stripped) = trimmed.strip_suffix("/ws/user") {
        stripped.to_owned()
    } else if let Some(stripped) = trimmed.strip_suffix("/ws") {
        stripped.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn channel_endpoint(base: &str, channel: ChannelType) -> String {
    let trimmed = base.trim_end_matches('/');
    let segment = match channel {
        ChannelType::Market => "market",
        ChannelType::User => "user",
    };
    format!("{trimmed}/ws/{segment}")
}
