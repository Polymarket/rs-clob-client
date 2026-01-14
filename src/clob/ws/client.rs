use std::sync::Arc;

use async_stream::try_stream;
use dashmap::mapref::one::{Ref, RefMut};
use dashmap::{DashMap, Entry};
use futures::Stream;
use futures::StreamExt as _;

use super::interest::InterestTracker;
use super::subscription::{ChannelType, SubscriptionManager};
use super::types::response::{
    BestBidAsk, BookUpdate, LastTradePrice, MarketResolved, MidpointUpdate, NewMarket,
    OrderMessage, PriceChange, TickSizeChange, TradeMessage, WsMessage,
};
use crate::Result;
use crate::auth::state::{Authenticated, State, Unauthenticated};
use crate::auth::{Credentials, Kind as AuthKind, Normal};
use crate::error::Error;
use crate::types::{Address, B256, Decimal, U256};
use crate::ws::ConnectionManager;
use crate::ws::config::Config;
use crate::ws::connection::ConnectionState;

/// WebSocket client for real-time market data and user updates.
///
/// This client uses a type-state pattern to enforce authentication requirements at compile time:
/// - [`Client<Unauthenticated>`]: Can only access public market data
/// - [`Client<Authenticated<K>>`]: Can access both public and user-specific data
///
/// # Examples
///
/// ```rust, no_run
/// use std::str::FromStr as _;
///
/// use polymarket_client_sdk::clob::ws::Client;
/// use polymarket_client_sdk::types::U256;
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create unauthenticated client
///     let client = Client::default();
///
///     let stream = client.subscribe_orderbook(vec![U256::from_str("106585164761922456203746651621390029417453862034640469075081961934906147433548")?])?;
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
    /// Resources for each WebSocket channel (lazily initialized)
    channels: DashMap<ChannelType, ChannelResources>,
}

impl Client<Unauthenticated> {
    /// Create a new unauthenticated WebSocket client.
    ///
    /// The `endpoint` should be the base WebSocket URL (e.g. `wss://...polymarket.com`);
    /// channel paths (`/ws/market` or `/ws/user`) are appended automatically.
    ///
    /// The WebSocket connection is established lazily upon the first subscription.
    pub fn new(endpoint: &str, config: Config) -> Result<Self> {
        let base_endpoint = normalize_base_endpoint(endpoint);

        Ok(Self {
            inner: Arc::new(ClientInner {
                state: Unauthenticated,
                config,
                base_endpoint,
                channels: DashMap::new(),
            }),
        })
    }

    /// Authenticate this client and elevate to authenticated state.
    ///
    /// Returns an error if there are other references to this client (e.g., from clones).
    /// Ensure all clones are dropped before calling this method.
    ///
    /// The user WebSocket connection is established lazily upon the first subscription.
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
            channels,
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
                base_endpoint,
                channels,
            }),
        })
    }
}

// Methods available in any state
impl<S: State> Client<S> {
    /// Subscribes to real-time orderbook updates for specified market assets.
    ///
    /// Returns a stream of orderbook snapshots showing all bid and ask levels.
    /// Each update contains the full orderbook state at that moment, useful for
    /// maintaining an accurate local orderbook copy.
    ///
    /// # Arguments
    ///
    /// * `asset_ids` - List of asset/token IDs to monitor
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be created or the WebSocket
    /// connection is not established.
    pub fn subscribe_orderbook(
        &self,
        asset_ids: Vec<U256>,
    ) -> Result<impl Stream<Item = Result<BookUpdate>>> {
        let resources = self.inner.get_or_create_channel(ChannelType::Market)?;
        let stream = resources.subscriptions.subscribe_market(asset_ids)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::Book(book)) => Some(Ok(book)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Subscribes to real-time last trade price updates for specified assets.
    ///
    /// Returns a stream of the most recent executed trade price for each asset.
    /// This reflects the latest market consensus price from actual transactions.
    ///
    /// # Arguments
    ///
    /// * `asset_ids` - List of asset/token IDs to monitor
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be created or the WebSocket
    /// connection is not established.
    pub fn subscribe_last_trade_price(
        &self,
        asset_ids: Vec<U256>,
    ) -> Result<impl Stream<Item = Result<LastTradePrice>>> {
        let resources = self.inner.get_or_create_channel(ChannelType::Market)?;
        let stream = resources.subscriptions.subscribe_market(asset_ids)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::LastTradePrice(last_trade_price)) => Some(Ok(last_trade_price)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Subscribes to real-time price changes for specified assets.
    ///
    /// Returns a stream of price updates when the best bid or ask changes.
    /// More lightweight than full orderbook subscriptions when you only need
    /// top-of-book prices.
    ///
    /// # Arguments
    ///
    /// * `asset_ids` - List of asset/token IDs to monitor
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be created or the WebSocket
    /// connection is not established.
    pub fn subscribe_prices(
        &self,
        asset_ids: Vec<U256>,
    ) -> Result<impl Stream<Item = Result<PriceChange>>> {
        let resources = self.inner.get_or_create_channel(ChannelType::Market)?;
        let stream = resources.subscriptions.subscribe_market(asset_ids)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::PriceChange(price)) => Some(Ok(price)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Subscribes to real-time tick size change events for specified assets.
    ///
    /// Returns a stream of tick size change when the backend adjusts the minimum
    /// price increment for an asset.
    ///
    /// # Arguments
    ///
    /// * `asset_ids` - List of asset/token IDs to monitor
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be created or the WebSocket
    /// connection is not established.
    pub fn subscribe_tick_size_change(
        &self,
        asset_ids: Vec<U256>,
    ) -> Result<impl Stream<Item = Result<TickSizeChange>>> {
        let resources = self.inner.get_or_create_channel(ChannelType::Market)?;
        let stream = resources.subscriptions.subscribe_market(asset_ids)?;

        Ok(stream.filter_map(|msg_result| async move {
            match msg_result {
                Ok(WsMessage::TickSizeChange(tsc)) => Some(Ok(tsc)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        }))
    }

    /// Subscribes to real-time midpoint price updates for specified assets.
    ///
    /// Returns a stream of midpoint prices calculated as the average of the best
    /// bid and best ask: `(best_bid + best_ask) / 2`. This provides a fair market
    /// price estimate that updates with every orderbook change.
    ///
    /// # Arguments
    ///
    /// * `asset_ids` - List of asset/token IDs to monitor
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be created or the WebSocket
    /// connection is not established.
    pub fn subscribe_midpoints(
        &self,
        asset_ids: Vec<U256>,
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
        asset_ids: Vec<U256>,
    ) -> Result<impl Stream<Item = Result<BestBidAsk>>> {
        let stream = self
            .inner
            .get_or_create_channel(ChannelType::Market)?
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
        asset_ids: Vec<U256>,
    ) -> Result<impl Stream<Item = Result<NewMarket>>> {
        let stream = self
            .inner
            .get_or_create_channel(ChannelType::Market)?
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
        asset_ids: Vec<U256>,
    ) -> Result<impl Stream<Item = Result<MarketResolved>>> {
        let stream = self
            .inner
            .get_or_create_channel(ChannelType::Market)?
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

    /// Get the current connection state for a specific channel.
    ///
    /// Returns [`ConnectionState::Disconnected`] if the channel has not been
    /// initialized yet (no subscriptions have been made).
    #[must_use]
    pub fn connection_state(&self, channel_type: ChannelType) -> ConnectionState {
        self.inner.channel(channel_type).as_deref().map_or(
            ConnectionState::Disconnected,
            ChannelResources::connection_state,
        )
    }

    /// Check if the WebSocket connection is established for a specific channel.
    ///
    /// Returns `false` if no subscriptions have been made yet for this channel.
    #[must_use]
    pub fn is_connected(&self, channel_type: ChannelType) -> bool {
        self.inner.channel(channel_type).is_some()
    }

    /// Get the number of active subscriptions.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.inner
            .channels
            .iter()
            .map(|entry| entry.value().subscriptions.subscription_count())
            .sum()
    }

    /// Unsubscribe from orderbook updates for specific assets.
    ///
    /// This decrements the reference count for each asset. The server unsubscribe
    /// is only sent when no other subscriptions are using those assets.
    pub fn unsubscribe_orderbook(&self, asset_ids: &[U256]) -> Result<()> {
        self.inner
            .unsubscribe_and_cleanup(ChannelType::Market, |subs| {
                subs.unsubscribe_market(asset_ids)
            })
    }

    /// Unsubscribe from price changes for specific assets.
    ///
    /// This decrements the reference count for each asset. The server unsubscribe
    /// is only sent when no other subscriptions are using those assets.
    pub fn unsubscribe_prices(&self, asset_ids: &[U256]) -> Result<()> {
        self.unsubscribe_orderbook(asset_ids)
    }

    /// Unsubscribe from tick size change updates for specific assets.
    ///
    /// This decrements the reference count for each asset. The server unsubscribe
    /// is only sent when no other subscriptions are using those assets.
    pub fn unsubscribe_tick_size_change(&self, asset_ids: &[U256]) -> Result<()> {
        self.unsubscribe_orderbook(asset_ids)
    }

    /// Unsubscribe from midpoint updates for specific assets.
    ///
    /// This decrements the reference count for each asset. The server unsubscribe
    /// is only sent when no other subscriptions are using those assets.
    pub fn unsubscribe_midpoints(&self, asset_ids: &[U256]) -> Result<()> {
        self.unsubscribe_orderbook(asset_ids)
    }
}

// Methods only available for authenticated clients
impl<K: AuthKind> Client<Authenticated<K>> {
    /// Subscribes to all user-specific events (orders and trades) for specified markets.
    ///
    /// Returns a stream of raw WebSocket messages containing both order updates
    /// (fills, cancellations, placements) and trade executions. Use this for
    /// comprehensive monitoring of all trading activity.
    ///
    /// # Arguments
    ///
    /// * `markets` - List of market condition IDs to monitor
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be created, the WebSocket
    /// connection is not established, or authentication fails.
    ///
    /// # Note
    ///
    /// This method is only available on authenticated clients.
    pub fn subscribe_user_events(
        &self,
        markets: Vec<B256>,
    ) -> Result<impl Stream<Item = Result<WsMessage>>> {
        let resources = self.inner.get_or_create_channel(ChannelType::User)?;

        resources
            .subscriptions
            .subscribe_user(markets, &self.inner.state.credentials)
    }

    /// Subscribes to real-time order status updates for the authenticated user.
    ///
    /// Returns a stream of order events including order placement, fills, partial fills,
    /// and cancellations. Useful for tracking the lifecycle of your orders in real-time.
    ///
    /// # Arguments
    ///
    /// * `markets` - List of market condition IDs to monitor
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be created, the WebSocket
    /// connection is not established, or authentication fails.
    ///
    /// # Note
    ///
    /// This method is only available on authenticated clients.
    pub fn subscribe_orders(
        &self,
        markets: Vec<B256>,
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

    /// Subscribes to real-time trade execution updates for the authenticated user.
    ///
    /// Returns a stream of trade events when your orders are matched and executed.
    /// Each trade event contains details about the execution price, size, maker/taker
    /// side, and associated order IDs.
    ///
    /// # Arguments
    ///
    /// * `markets` - List of market condition IDs to monitor
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription cannot be created, the WebSocket
    /// connection is not established, or authentication fails.
    ///
    /// # Note
    ///
    /// This method is only available on authenticated clients.
    pub fn subscribe_trades(
        &self,
        markets: Vec<B256>,
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
    pub fn unsubscribe_user_events(&self, markets: &[B256]) -> Result<()> {
        self.inner
            .unsubscribe_and_cleanup(ChannelType::User, |subs| subs.unsubscribe_user(markets))
    }

    /// Unsubscribe from user's order updates for specific markets.
    ///
    /// This decrements the reference count for each market. The server unsubscribe
    /// is only sent when no other subscriptions are using those markets.
    pub fn unsubscribe_orders(&self, markets: &[B256]) -> Result<()> {
        self.unsubscribe_user_events(markets)
    }

    /// Unsubscribe from user's trade executions for specific markets.
    ///
    /// This decrements the reference count for each market. The server unsubscribe
    /// is only sent when no other subscriptions are using those markets.
    pub fn unsubscribe_trades(&self, markets: &[B256]) -> Result<()> {
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
            channels,
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
    fn get_or_create_channel(
        &self,
        channel_type: ChannelType,
    ) -> Result<Ref<'_, ChannelType, ChannelResources>> {
        self.channels
            .entry(channel_type)
            .or_try_insert_with(|| {
                let endpoint = channel_endpoint(&self.base_endpoint, channel_type);
                ChannelResources::new(endpoint, self.config.clone())
            })
            .map(RefMut::downgrade)
    }

    fn channel(&self, channel_type: ChannelType) -> Option<Ref<'_, ChannelType, ChannelResources>> {
        self.channels.get(&channel_type)
    }

    /// Helper to unsubscribe and remove connection if there are no more subscriptions on this channel
    fn unsubscribe_and_cleanup<F>(&self, channel_type: ChannelType, unsubscribe_fn: F) -> Result<()>
    where
        F: FnOnce(&SubscriptionManager) -> Result<()>,
    {
        match self.channels.entry(channel_type) {
            Entry::Vacant(_) => Ok(()),
            Entry::Occupied(channel_ref) => {
                // Clone the Arc to subscriptions while holding the Entry
                let subs = Arc::clone(&channel_ref.get().subscriptions);
                drop(channel_ref); // Release Entry immediately

                // Do potentially blocking network I/O without holding the Entry lock
                unsubscribe_fn(&subs)?;

                // Atomically check and remove channel if empty
                if let Entry::Occupied(entry) = self.channels.entry(channel_type)
                    && !entry.get().subscriptions.has_subscriptions(channel_type)
                {
                    entry.remove();
                }
                Ok(())
            }
        }
    }
}

/// Resources for a WebSocket channel.
struct ChannelResources {
    connection: ConnectionManager<WsMessage, Arc<InterestTracker>>,
    subscriptions: Arc<SubscriptionManager>,
}

impl ChannelResources {
    fn new(endpoint: String, config: Config) -> Result<Self> {
        let interest = Arc::new(InterestTracker::new());
        let connection = ConnectionManager::new(endpoint, config, Arc::clone(&interest))?;
        let subscriptions = Arc::new(SubscriptionManager::new(connection.clone(), interest));

        subscriptions.start_reconnection_handler();

        Ok(Self {
            connection,
            subscriptions,
        })
    }

    fn connection_state(&self) -> ConnectionState {
        self.connection.state()
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
