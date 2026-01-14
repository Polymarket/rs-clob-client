#![expect(
    clippy::module_name_repetitions,
    reason = "Subscription types deliberately include the module name for clarity"
)]

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, PoisonError, RwLock};
use std::time::Instant;

use async_stream::try_stream;
use dashmap::{DashMap, Entry};
use futures::Stream;
use tokio::sync::broadcast::error::RecvError;

use super::interest::{InterestTracker, MessageInterest};
use super::types::request::SubscriptionRequest;
use super::types::response::WsMessage;
use crate::Result;
use crate::auth::Credentials;
use crate::types::{B256, U256};
use crate::ws::ConnectionManager;
use crate::ws::WsError;
use crate::ws::connection::ConnectionState;

/// What a subscription is targeting.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SubscriptionTarget {
    /// Subscribed to market data for specific assets.
    Assets(Vec<U256>),
    /// Subscribed to user events for specific markets.
    Markets(Vec<B256>),
}

impl SubscriptionTarget {
    /// Returns the channel type this target belongs to.
    #[must_use]
    pub const fn channel(&self) -> ChannelType {
        match self {
            Self::Assets(_) => ChannelType::Market,
            Self::Markets(_) => ChannelType::User,
        }
    }
}

/// Information about an active subscription.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SubscriptionInfo {
    /// What this subscription is targeting.
    pub target: SubscriptionTarget,
    /// When the subscription was created.
    pub created_at: Instant,
}

impl SubscriptionInfo {
    /// Returns the channel type for this subscription.
    #[must_use]
    pub const fn channel(&self) -> ChannelType {
        self.target.channel()
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelType {
    /// Public market data channel
    Market,
    /// Authenticated user data channel
    User,
}

/// Manages active subscriptions and routes messages to subscribers.
pub struct SubscriptionManager {
    connection: ConnectionManager<WsMessage, Arc<InterestTracker>>,
    active_subs: DashMap<String, SubscriptionInfo>,
    interest: Arc<InterestTracker>,
    /// Subscribed assets with reference counts (for multiplexing)
    subscribed_assets: DashMap<U256, usize>,
    /// Subscribed markets with reference counts (for multiplexing)
    subscribed_markets: DashMap<B256, usize>,
    last_auth: Arc<RwLock<Option<Credentials>>>,
    /// Track if custom features were enabled for any market subscription
    /// (enables `best_bid_ask`, `new_market`, `market_resolved` messages)
    custom_features_enabled: AtomicBool,
}

impl SubscriptionManager {
    /// Create a new subscription manager.
    #[must_use]
    pub fn new(
        connection: ConnectionManager<WsMessage, Arc<InterestTracker>>,
        interest: Arc<InterestTracker>,
    ) -> Self {
        Self {
            connection,
            active_subs: DashMap::new(),
            interest,
            subscribed_assets: DashMap::new(),
            subscribed_markets: DashMap::new(),
            last_auth: Arc::new(RwLock::new(None)),
            custom_features_enabled: AtomicBool::new(false),
        }
    }

    /// Start the reconnection handler that re-subscribes on connection recovery.
    pub fn start_reconnection_handler(self: &Arc<Self>) {
        let this = Arc::clone(self);

        tokio::spawn(async move {
            let mut state_rx = this.connection.state_receiver();
            let mut was_connected = state_rx.borrow().is_connected();

            loop {
                // Wait for next state change
                if state_rx.changed().await.is_err() {
                    // Channel closed, connection manager is gone
                    break;
                }

                let state = *state_rx.borrow_and_update();

                match state {
                    ConnectionState::Connected { .. } => {
                        if was_connected {
                            // Reconnect to subscriptions
                            #[cfg(feature = "tracing")]
                            tracing::debug!("WebSocket reconnected, re-establishing subscriptions");
                            this.resubscribe_all();
                        }
                        was_connected = true;
                    }
                    ConnectionState::Disconnected => {
                        // Connection permanently closed
                        break;
                    }
                    _ => {
                        // Other states are no-op
                    }
                }
            }
        });
    }

    /// Re-send subscription requests for all tracked assets and markets.
    fn resubscribe_all(&self) {
        // Collect all subscribed assets
        let assets: Vec<U256> = self.subscribed_assets.iter().map(|r| *r.key()).collect();

        if !assets.is_empty() {
            let custom_features = self.custom_features_enabled.load(Ordering::Relaxed);
            #[cfg(feature = "tracing")]
            tracing::debug!(
                count = assets.len(),
                custom_features,
                "Re-subscribing to market assets"
            );
            let mut request = SubscriptionRequest::market(assets);
            if custom_features {
                request = request.with_custom_features(true);
            }
            if let Err(e) = self.connection.send(&request) {
                #[cfg(feature = "tracing")]
                tracing::warn!(%e, "Failed to re-subscribe to market channel");
                #[cfg(not(feature = "tracing"))]
                let _ = &e;
            }
        }

        // Store auth for re-subscription on reconnect.
        // We can recover from poisoned lock because Option<Credentials> has no inconsistent intermediate state.
        let auth = self
            .last_auth
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(auth) = auth {
            let markets: Vec<B256> = self.subscribed_markets.iter().map(|r| *r.key()).collect();

            #[cfg(feature = "tracing")]
            tracing::debug!(
                markets_count = markets.len(),
                "Re-subscribing to user channel"
            );
            let request = SubscriptionRequest::user(markets);
            if let Err(e) = self.connection.send_authenticated(&request, &auth) {
                #[cfg(feature = "tracing")]
                tracing::warn!(%e, "Failed to re-subscribe to user channel");
                #[cfg(not(feature = "tracing"))]
                let _ = &e;
            }
        }
    }

    /// Subscribe to public market data channel.
    ///
    /// This will fail if `asset_ids` is empty.
    pub fn subscribe_market(
        &self,
        asset_ids: Vec<U256>,
    ) -> Result<impl Stream<Item = Result<WsMessage>> + use<>> {
        self.subscribe_market_with_options(asset_ids, false)
    }

    /// Subscribe to public market data channel with options.
    ///
    /// When `custom_features` is true, enables receiving additional message types:
    /// `best_bid_ask`, `new_market`, `market_resolved`.
    ///
    /// This will fail if `asset_ids` is empty.
    pub fn subscribe_market_with_options(
        &self,
        asset_ids: Vec<U256>,
        custom_features: bool,
    ) -> Result<impl Stream<Item = Result<WsMessage>> + use<>> {
        if asset_ids.is_empty() {
            return Err(WsError::SubscriptionFailed(
                "asset_ids cannot be empty: at least one asset ID must be provided for subscription"
                    .to_owned(),
            )
            .into());
        }

        self.interest.add(MessageInterest::MARKET);

        // Track if custom features are enabled (for re-subscription on reconnect)
        if custom_features {
            self.custom_features_enabled.store(true, Ordering::Relaxed);
        }

        // Increment refcounts and determine which assets are truly new
        let new_assets: Vec<U256> = asset_ids
            .iter()
            .filter_map(|id| match self.subscribed_assets.entry(*id) {
                Entry::Occupied(mut o) => {
                    *o.get_mut() += 1;
                    None
                }
                Entry::Vacant(v) => {
                    v.insert(1);
                    Some(id.to_owned())
                }
            })
            .collect();

        // Only send subscription request for new assets
        if new_assets.is_empty() {
            #[cfg(feature = "tracing")]
            tracing::debug!("All requested assets already subscribed, multiplexing");
        } else {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                count = new_assets.len(),
                ?new_assets,
                custom_features,
                "Subscribing to new market assets"
            );
            let mut request = SubscriptionRequest::market(new_assets);
            if custom_features {
                request = request.with_custom_features(true);
            }
            self.connection.send(&request)?;
        }

        // Register subscription
        let sub_id = format!(
            "market:{}",
            asset_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        self.active_subs.insert(
            sub_id,
            SubscriptionInfo {
                target: SubscriptionTarget::Assets(asset_ids.clone()),
                created_at: Instant::now(),
            },
        );

        // Create filtered stream with its own receiver
        let mut rx = self.connection.subscribe();
        let asset_ids_set: HashSet<U256> = asset_ids.into_iter().collect();

        Ok(try_stream! {
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        // Filter messages by asset_id
                        let should_yield = match &msg {
                            WsMessage::Book(book) => asset_ids_set.contains(&book.asset_id),
                            WsMessage::PriceChange(price) => {
                                price
                                    .price_changes
                                    .iter()
                                    .any(|pc| asset_ids_set.contains(&pc.asset_id))
                            },
                            WsMessage::LastTradePrice(ltp) => asset_ids_set.contains(&ltp.asset_id),
                            WsMessage::TickSizeChange(tsc) => asset_ids_set.contains(&tsc.asset_id),
                            WsMessage::BestBidAsk(bba) => asset_ids_set.contains(&bba.asset_id),
                            WsMessage::NewMarket(nm) => {
                                nm.asset_ids.iter().any(|id| asset_ids_set.contains(id))
                            },
                            WsMessage::MarketResolved(mr) => {
                                mr.asset_ids.iter().any(|id| asset_ids_set.contains(id))
                            },
                            _ => false,
                        };

                        if should_yield {
                            yield msg
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        #[cfg(feature = "tracing")]
                        tracing::warn!("Subscription lagged, missed {n} messages");
                        Err(WsError::Lagged { count: n })?;
                    }
                    Err(RecvError::Closed) => {
                        break;
                    }
                }
            }
        })
    }

    /// Subscribe to authenticated user channel.
    pub fn subscribe_user(
        &self,
        markets: Vec<B256>,
        auth: &Credentials,
    ) -> Result<impl Stream<Item = Result<WsMessage>> + use<>> {
        self.interest.add(MessageInterest::USER);

        // Store auth for re-subscription on reconnect.
        // We can recover from poisoned lock because Option<Credentials> has no inconsistent intermediate state.
        *self
            .last_auth
            .write()
            .unwrap_or_else(PoisonError::into_inner) = Some(auth.clone());

        // Increment refcounts and determine which markets are truly new
        let new_markets: Vec<B256> = markets
            .iter()
            .filter_map(|id| match self.subscribed_markets.entry(id.to_owned()) {
                Entry::Occupied(mut o) => {
                    *o.get_mut() += 1;
                    None
                }
                Entry::Vacant(v) => {
                    v.insert(1);
                    Some(id.to_owned())
                }
            })
            .collect();

        // Only send subscription request for new markets (or if subscribing to all)
        if !markets.is_empty() && new_markets.is_empty() {
            #[cfg(feature = "tracing")]
            tracing::debug!("All requested markets already subscribed, multiplexing");
        } else {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                count = new_markets.len(),
                ?new_markets,
                "Subscribing to user channel"
            );
            let request = SubscriptionRequest::user(new_markets);
            self.connection.send_authenticated(&request, auth)?;
        }

        // Register subscription
        let sub_id = format!(
            "user:{}",
            markets
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        self.active_subs.insert(
            sub_id,
            SubscriptionInfo {
                target: SubscriptionTarget::Markets(markets),
                created_at: Instant::now(),
            },
        );

        // Create stream for user messages
        let mut rx = self.connection.subscribe();

        Ok(try_stream! {
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        if msg.is_user() {
                            yield msg;
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        #[cfg(feature = "tracing")]
                        tracing::warn!("Subscription lagged, missed {n} messages");
                        Err(WsError::Lagged { count: n })?;
                    }
                    Err(RecvError::Closed) => {
                        break;
                    }
                }
            }
        })
    }

    /// Get information about all active subscriptions.
    #[must_use]
    pub fn active_subscriptions(&self) -> HashMap<ChannelType, Vec<SubscriptionInfo>> {
        self.active_subs
            .iter()
            .fold(HashMap::new(), |mut acc, entry| {
                acc.entry(entry.value().channel())
                    .or_default()
                    .push(entry.value().clone());
                acc
            })
    }

    /// Get the number of active subscriptions.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.active_subs.len()
    }

    /// Check if there are any subscriptions for a specific channel type.
    #[must_use]
    pub fn has_subscriptions(&self, channel: ChannelType) -> bool {
        match channel {
            ChannelType::Market => !self.subscribed_assets.is_empty(),
            ChannelType::User => !self.subscribed_markets.is_empty(),
        }
    }

    /// Unsubscribe from market data for specific assets.
    ///
    /// This decrements the reference count for each asset. Only sends an unsubscribe
    /// request to the server when the reference count reaches zero (no other streams
    /// are using that asset).
    pub fn unsubscribe_market(&self, asset_ids: &[U256]) -> Result<()> {
        if asset_ids.is_empty() {
            return Err(WsError::SubscriptionFailed(
                "asset_ids cannot be empty: at least one asset ID must be provided for unsubscription"
                    .to_owned(),
            )
            .into());
        }

        let mut to_unsubscribe = Vec::new();

        // Atomically decrement refcounts and remove assets that reach zero
        // Using Entry API to prevent TOCTOU race between decrement and removal
        for id in asset_ids {
            if let Entry::Occupied(mut entry) = self.subscribed_assets.entry(*id) {
                let refcount = entry.get_mut();
                *refcount = refcount.saturating_sub(1);
                if *refcount == 0 {
                    entry.remove();
                    to_unsubscribe.push(*id);
                }
            }
        }

        // Send unsubscribe only for zero-refcount assets
        if !to_unsubscribe.is_empty() {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                count = to_unsubscribe.len(),
                ?to_unsubscribe,
                "Unsubscribing from market assets"
            );
            let request = SubscriptionRequest::market_unsubscribe(to_unsubscribe);
            self.connection.send(&request)?;
        }

        // Remove active_subs entries where all assets are now unsubscribed
        self.active_subs.retain(|_, info| {
            if let SubscriptionTarget::Assets(assets) = &info.target {
                // Keep entry only if at least one asset is still subscribed
                assets
                    .iter()
                    .any(|a| self.subscribed_assets.contains_key(a))
            } else {
                true // Keep non-market subscriptions
            }
        });

        Ok(())
    }

    /// Unsubscribe from user events for specific markets.
    ///
    /// This decrements the reference count for each market. Only sends an unsubscribe
    /// request to the server when the reference count reaches zero (no other streams
    /// are using that market).
    pub fn unsubscribe_user(&self, markets: &[B256]) -> Result<()> {
        if markets.is_empty() {
            return Err(WsError::SubscriptionFailed(
                "markets cannot be empty: at least one market ID must be provided for unsubscription"
                    .to_owned(),
            )
            .into());
        }

        let mut to_unsubscribe = Vec::new();

        // Atomically decrement refcounts and remove markets that reach zero
        // Using Entry API to prevent TOCTOU race between decrement and removal
        for m in markets {
            if let Entry::Occupied(mut entry) = self.subscribed_markets.entry(*m) {
                let refcount = entry.get_mut();
                *refcount = refcount.saturating_sub(1);
                if *refcount == 0 {
                    entry.remove();
                    to_unsubscribe.push(*m);
                }
            }
        }

        // Send unsubscribe only for zero-refcount markets
        if !to_unsubscribe.is_empty() {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                count = to_unsubscribe.len(),
                ?to_unsubscribe,
                "Unsubscribing from user markets"
            );

            // Get auth for unsubscribe request
            let auth = self
                .last_auth
                .read()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
                .ok_or(WsError::AuthenticationFailed)?;

            let request = SubscriptionRequest::user_unsubscribe(to_unsubscribe);
            self.connection.send_authenticated(&request, &auth)?;
        }

        // Remove active_subs entries where all markets are now unsubscribed
        self.active_subs.retain(|_, info| {
            if let SubscriptionTarget::Markets(markets) = &info.target {
                // Keep entry only if at least one market is still subscribed
                markets
                    .iter()
                    .any(|m| self.subscribed_markets.contains_key(m))
            } else {
                true // Keep non-user subscriptions
            }
        });

        Ok(())
    }
}
