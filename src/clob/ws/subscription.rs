#![expect(
    clippy::module_name_repetitions,
    reason = "Subscription types deliberately include the module name for clarity"
)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, PoisonError, RwLock};
use std::time::Instant;

use async_stream::try_stream;
use dashmap::{DashMap, DashSet};
use futures::Stream;
use tokio::sync::broadcast::error::RecvError;

use super::connection::{ConnectionManager, ConnectionState};
use super::error::WsError;
use super::interest::{InterestTracker, MessageInterest};
use super::types::{SubscriptionRequest, WsMessage};
use crate::Result;
use crate::auth::Credentials;

/// What a subscription is targeting.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SubscriptionTarget {
    /// Subscribed to market data for specific assets.
    Assets(Vec<String>),
    /// Subscribed to user events for specific markets.
    Markets(Vec<String>),
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
    connection: ConnectionManager,
    active_subs: DashMap<String, SubscriptionInfo>,
    interest: Arc<InterestTracker>,
    subscribed_assets: DashSet<String>,
    subscribed_markets: DashSet<String>,
    last_auth: Arc<RwLock<Option<Credentials>>>,
}

impl SubscriptionManager {
    /// Create a new subscription manager.
    #[must_use]
    pub fn new(connection: ConnectionManager, interest: Arc<InterestTracker>) -> Self {
        Self {
            connection,
            active_subs: DashMap::new(),
            interest,
            subscribed_assets: DashSet::new(),
            subscribed_markets: DashSet::new(),
            last_auth: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the reconnection handler that re-subscribes on connection recovery.
    pub fn start_reconnection_handler(self: &Arc<Self>) {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            this.reconnection_loop().await;
        });
    }

    /// Monitor connection state and re-subscribe when reconnection occurs.
    async fn reconnection_loop(&self) {
        let mut state_rx = self.connection.state_receiver();
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
                        self.resubscribe_all();
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
    }

    /// Re-send subscription requests for all tracked assets and markets.
    fn resubscribe_all(&self) {
        // Collect all subscribed assets
        let assets: Vec<String> = self
            .subscribed_assets
            .iter()
            .map(|r| r.key().clone())
            .collect();

        if !assets.is_empty() {
            #[cfg(feature = "tracing")]
            tracing::debug!(count = assets.len(), "Re-subscribing to market assets");
            let request = SubscriptionRequest::market(assets);
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
            let markets: Vec<String> = self
                .subscribed_markets
                .iter()
                .map(|r| r.key().clone())
                .collect();

            #[cfg(feature = "tracing")]
            tracing::debug!(
                markets_count = markets.len(),
                "Re-subscribing to user channel"
            );
            let request = SubscriptionRequest::user(markets, auth);
            if let Err(e) = self.connection.send(&request) {
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
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<WsMessage>>> {
        if asset_ids.is_empty() {
            return Err(WsError::SubscriptionFailed(
                "asset_ids cannot be empty: at least one asset ID must be provided for subscription"
                    .to_owned(),
            )
            .into());
        }

        self.interest.add(MessageInterest::MARKET);

        // Determine which assets are not yet subscribed
        let new_assets: Vec<String> = asset_ids
            .iter()
            .filter(|id| !self.subscribed_assets.contains(*id))
            .inspect(|id| _ = self.subscribed_assets.insert((*id).to_owned()))
            .map(ToOwned::to_owned)
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
                "Subscribing to new market assets"
            );
            let request = SubscriptionRequest::market(new_assets);
            self.connection.send(&request)?;
        }

        // Register subscription
        let sub_id = format!("market:{}", asset_ids.join(","));
        self.active_subs.insert(
            sub_id,
            SubscriptionInfo {
                target: SubscriptionTarget::Assets(asset_ids.clone()),
                created_at: Instant::now(),
            },
        );

        // Create filtered stream with its own receiver
        let mut rx = self.connection.subscribe();
        let asset_ids_set: HashSet<String> = asset_ids.into_iter().collect();

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
        markets: Vec<String>,
        auth: Credentials,
    ) -> Result<impl Stream<Item = Result<WsMessage>>> {
        self.interest.add(MessageInterest::USER);

        // Store auth for re-subscription on reconnect.
        // We can recover from poisoned lock because Option<Credentials> has no inconsistent intermediate state.
        *self
            .last_auth
            .write()
            .unwrap_or_else(PoisonError::into_inner) = Some(auth.clone());

        // Determine which markets are not yet subscribed
        let new_markets: Vec<String> = markets
            .iter()
            .filter(|m| !self.subscribed_markets.contains(*m))
            .inspect(|id| _ = self.subscribed_markets.insert((*id).to_owned()))
            .map(ToOwned::to_owned)
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
            let request = SubscriptionRequest::user(new_markets, auth);
            self.connection.send(&request)?;
        }

        // Register subscription
        let sub_id = format!("user:{}", markets.join(","));
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
}
