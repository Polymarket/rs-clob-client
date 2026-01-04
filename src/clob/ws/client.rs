use std::collections::{HashMap, hash_map::Entry};
use std::sync::Arc;

use async_stream::try_stream;
use futures::Stream;
use futures::StreamExt as _;
use once_cell::sync::OnceCell;

use super::config::Config;
use super::connection::{ConnectionManager, ConnectionState};
use super::interest::InterestTracker;
use super::subscription::{ChannelType, SubscriptionManager};
use super::types::response::{
    BestBidAsk, BookUpdate, MarketResolved, MidpointUpdate, NewMarket, OrderMessage, PriceChange,
    TradeMessage, WsMessage,
};
use crate::Result;
use crate::auth::state::{Authenticated, State, Unauthenticated};
use crate::auth::{Credentials, Kind as AuthKind, Normal};
use crate::error::Error;
use crate::types::{Address, Decimal};

/// WebSocket client for real-time market data and user updates.
///
/// This client uses a type-state pattern to enforce authentication requirements at compile time:
/// - [`Client<Unauthenticated>`]: Can only access public market data
/// - [`Client<Authenticated<K>>`]: Can access both public and user-specific data
///
/// # Examples
///
/// ```rust, no_run
/// use polymarket_client_sdk::clob::ws::Client;
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create unauthenticated client
///     let client = Client::default();
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
#[derive(Clone)]
pub struct Client<S: State = Unauthenticated> {
    inner: Arc<ClientInner<S>>,
}

impl Default for Client<Unauthenticated> {
    fn default() -> Self {
        Self::new(
            "wss://ws-subscriptions-clob.polymarket.com",
            Config::default(),
        )
        .expect("WebSocket client with default endpoint should succeed")
    }
}

struct ClientInner<S: State> {
    /// Current state of the client (authenticated or unauthenticated)
    state: S,
    /// Configuration for the WebSocket connections
    config: Config,
    /// Base endpoint without channel suffix (e.g. `wss://...`)
    base_endpoint: String,
    /// Resources for each WebSocket channel
    channels: HashMap<ChannelType, ChannelHandles>,
}

impl Client<Unauthenticated> {
    /// Create a new unauthenticated WebSocket client.
    ///
    /// The `endpoint` should be the base WebSocket URL (e.g. `wss://...polymarket.com`);
    /// channel paths (`/ws/market` or `/ws/user`) are appended automatically.
    ///
    /// Connection to the WebSocket server is deferred until the first subscription
    /// is made. This prevents unnecessary connections when no subscriptions are needed.
    pub fn new(endpoint: &str, config: Config) -> Result<Self> {
        let normalized = normalize_base_endpoint(endpoint);
        let market_handles = ChannelHandles::new_lazy(
            channel_endpoint(&normalized, ChannelType::Market),
            config.clone(),
        );
        let mut channels = HashMap::new();
        channels.insert(ChannelType::Market, market_handles);

        Ok(Self {
            inner: Arc::new(ClientInner {
                state: Unauthenticated,
                config,
                base_endpoint: normalized,
                channels,
            }),
        })
    }

    /// Authenticate this client and elevate to authenticated state.
    ///
    /// Returns an error if there are other references to this client (e.g., from clones).
    /// Ensure all clones are dropped before calling this method.
    ///
    /// Connection to the user WebSocket channel is deferred until the first
    /// subscription is made.
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
            base_endpoint,
            mut channels,
            ..
        } = inner;

        if let Entry::Vacant(slot) = channels.entry(ChannelType::User) {
            let handles = ChannelHandles::new_lazy(
                channel_endpoint(&base_endpoint, ChannelType::User),
                config.clone(),
            );
            slot.insert(handles);
        }

        Ok(Client {
            inner: Arc::new(ClientInner {
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
impl<S: State> Client<S> {
    /// Subscribe to orderbook updates for specific assets.
    pub fn subscribe_orderbook(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<BookUpdate>>> {
        let resources = self.market_resources()?;
        let stream = resources.subscriptions.subscribe_market(asset_ids)?;

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
        let resources = self.market_resources()?;
        let stream = resources.subscriptions.subscribe_market(asset_ids)?;

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

        Ok(try_stream! {
            for await book_result in stream {
                let book = book_result?;

                // Calculate midpoint from best bid/ask
                if let (Some(bid), Some(ask)) = (book.bids.first(), book.asks.first()) {
                    let midpoint = (bid.price + ask.price) / Decimal::TWO;
                    yield MidpointUpdate {
                        asset_id: book.asset_id,
                        market: book.market,
                        midpoint,
                        timestamp: book.timestamp,
                    };
                }
            }
        })
    }

    /// Subscribe to best bid/ask updates with custom features enabled.
    ///
    /// Requires `custom_feature_enabled` flag on the server side.
    pub fn subscribe_best_bid_ask(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<BestBidAsk>>> {
        let stream = self
            .market_resources()?
            .subscriptions
            .subscribe_market_with_options(asset_ids, true)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::BestBidAsk(bba)) => Some(Ok(bba)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Subscribe to new market events with custom features enabled.
    ///
    /// Requires `custom_feature_enabled` flag on the server side.
    pub fn subscribe_new_markets(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<NewMarket>>> {
        let stream = self
            .market_resources()?
            .subscriptions
            .subscribe_market_with_options(asset_ids, true)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::NewMarket(nm)) => Some(Ok(nm)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Subscribe to market resolved events with custom features enabled.
    ///
    /// Requires `custom_feature_enabled` flag on the server side.
    pub fn subscribe_market_resolutions(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<MarketResolved>>> {
        let stream = self
            .market_resources()?
            .subscriptions
            .subscribe_market_with_options(asset_ids, true)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::MarketResolved(mr)) => Some(Ok(mr)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Get the current connection state.
    ///
    /// Returns [`ConnectionState::Disconnected`] if the connection has not been
    /// initialized yet (no subscriptions have been made).
    #[must_use]
    pub fn connection_state(&self) -> ConnectionState {
        self.inner.channel(ChannelType::Market).map_or(
            ConnectionState::Disconnected,
            ChannelHandles::connection_state,
        )
    }

    /// Check if the WebSocket connection has been initialized.
    ///
    /// Returns `false` if no subscriptions have been made yet.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.inner
            .channel(ChannelType::Market)
            .is_some_and(ChannelHandles::is_connected)
    }

    /// Get the number of active subscriptions.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.inner
            .channels
            .values()
            .filter_map(|handles| handles.resources.get())
            .map(|resources| resources.subscriptions.subscription_count())
            .sum()
    }

    /// Unsubscribe from orderbook updates for specific assets.
    ///
    /// This decrements the reference count for each asset. The server unsubscribe
    /// is only sent when no other subscriptions are using those assets.
    pub fn unsubscribe_orderbook(&self, asset_ids: &[String]) -> Result<()> {
        self.market_resources()?
            .subscriptions
            .unsubscribe_market(asset_ids)
    }

    /// Unsubscribe from price changes for specific assets.
    ///
    /// This decrements the reference count for each asset. The server unsubscribe
    /// is only sent when no other subscriptions are using those assets.
    pub fn unsubscribe_prices(&self, asset_ids: &[String]) -> Result<()> {
        self.market_resources()?
            .subscriptions
            .unsubscribe_market(asset_ids)
    }

    /// Unsubscribe from midpoint updates for specific assets.
    ///
    /// This decrements the reference count for each asset. The server unsubscribe
    /// is only sent when no other subscriptions are using those assets.
    pub fn unsubscribe_midpoints(&self, asset_ids: &[String]) -> Result<()> {
        self.market_resources()?
            .subscriptions
            .unsubscribe_market(asset_ids)
    }

    fn market_handles(&self) -> Result<&ChannelHandles> {
        self.inner
            .channel(ChannelType::Market)
            .ok_or_else(|| Error::validation("Market channel unavailable; recreate client"))
    }

    fn market_resources(&self) -> Result<&LazyChannelResources> {
        self.market_handles()?.get_or_connect()
    }
}

// Methods only available for authenticated clients
impl<K: AuthKind> Client<Authenticated<K>> {
    /// Subscribe to raw user channel events (orders and trades).
    pub fn subscribe_user_events(
        &self,
        markets: Vec<String>,
    ) -> Result<impl Stream<Item = Result<WsMessage>>> {
        let resources = self.user_resources()?;

        resources
            .subscriptions
            .subscribe_user(markets, self.inner.state.credentials.clone())
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

    /// Unsubscribe from user channel events for specific markets.
    ///
    /// This decrements the reference count for each market. The server unsubscribe
    /// is only sent when no other subscriptions are using those markets.
    pub fn unsubscribe_user_events(&self, markets: &[String]) -> Result<()> {
        self.user_resources()?
            .subscriptions
            .unsubscribe_user(markets)
    }

    fn user_handles(&self) -> Result<&ChannelHandles> {
        self.inner
            .channel(ChannelType::User)
            .ok_or_else(|| Error::validation("User channel unavailable; authenticate first"))
    }

    fn user_resources(&self) -> Result<&LazyChannelResources> {
        self.user_handles()?.get_or_connect()
    }

    /// Unsubscribe from user's order updates for specific markets.
    ///
    /// This decrements the reference count for each market. The server unsubscribe
    /// is only sent when no other subscriptions are using those markets.
    pub fn unsubscribe_orders(&self, markets: &[String]) -> Result<()> {
        self.unsubscribe_user_events(markets)
    }

    /// Unsubscribe from user's trade executions for specific markets.
    ///
    /// This decrements the reference count for each market. The server unsubscribe
    /// is only sent when no other subscriptions are using those markets.
    pub fn unsubscribe_trades(&self, markets: &[String]) -> Result<()> {
        self.unsubscribe_user_events(markets)
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
            base_endpoint,
            mut channels,
            ..
        } = inner;
        channels.remove(&ChannelType::User);

        Ok(Client {
            inner: Arc::new(ClientInner {
                state: Unauthenticated,
                config,
                base_endpoint,
                channels,
            }),
        })
    }
}

impl<S: State> ClientInner<S> {
    fn channel(&self, kind: ChannelType) -> Option<&ChannelHandles> {
        self.channels.get(&kind)
    }
}

/// Lazily-initialized resources for a WebSocket channel.
struct LazyChannelResources {
    connection: ConnectionManager,
    subscriptions: Arc<SubscriptionManager>,
}

/// Handles for a specific WebSocket channel.
///
/// Uses lazy initialization to avoid connecting to the server until
/// the first subscription is made.
struct ChannelHandles {
    endpoint: String,
    config: Config,
    resources: OnceCell<LazyChannelResources>,
}

impl ChannelHandles {
    fn new_lazy(endpoint: String, config: Config) -> Self {
        Self {
            endpoint,
            config,
            resources: OnceCell::new(),
        }
    }

    fn get_or_connect(&self) -> Result<&LazyChannelResources> {
        self.resources.get_or_try_init(|| {
            let interest = Arc::new(InterestTracker::new());
            let connection =
                ConnectionManager::new(self.endpoint.clone(), self.config.clone(), &interest)?;
            let subscriptions = Arc::new(SubscriptionManager::new(connection.clone(), interest));

            subscriptions.start_reconnection_handler();

            Ok(LazyChannelResources {
                connection,
                subscriptions,
            })
        })
    }

    fn is_connected(&self) -> bool {
        self.resources.get().is_some()
    }

    fn connection_state(&self) -> ConnectionState {
        self.resources
            .get()
            .map_or(ConnectionState::Disconnected, |r| r.connection.state())
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
