#![expect(
    clippy::module_name_repetitions,
    reason = "Subscription types deliberately include the module name for clarity"
)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use async_stream::stream;
use dashmap::{DashMap, DashSet};
use futures::Stream;
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, warn};

use super::connection::ConnectionManager;
use super::error::WsError;
use super::interest::{InterestTracker, MessageInterest};
use super::messages::{AuthPayload, SubscriptionRequest, WsMessage};
use crate::Result;

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
    connection: Arc<ConnectionManager>,
    active_subs: Arc<DashMap<String, SubscriptionInfo>>,
    interest: Arc<InterestTracker>,
    subscribed_assets: DashSet<String>,
    subscribed_markets: DashSet<String>,
}

impl SubscriptionManager {
    /// Create a new subscription manager.
    #[must_use]
    pub fn new(connection: Arc<ConnectionManager>, interest: Arc<InterestTracker>) -> Self {
        Self {
            connection,
            active_subs: Arc::new(DashMap::new()),
            interest,
            subscribed_assets: DashSet::new(),
            subscribed_markets: DashSet::new(),
        }
    }

    /// Subscribe to public market data channel.
    pub fn subscribe_market(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<WsMessage>>> {
        self.interest.add(MessageInterest::MARKET);

        // Determine which assets are not yet subscribed
        let new_assets: Vec<String> = asset_ids
            .iter()
            .filter(|id| self.subscribed_assets.insert((*id).clone()))
            .cloned()
            .collect();

        // Only send subscription request for new assets
        if new_assets.is_empty() {
            debug!("All requested assets already subscribed, multiplexing");
        } else {
            debug!(
                count = new_assets.len(),
                ?asset_ids,
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

        Ok(stream! {
            loop {
                match rx.recv().await {
                    Ok(res) => {
                        match res.as_ref() {
                            Ok(msg) => {
                                // Filter messages by asset_id
                                let should_yield = match msg {
                                    WsMessage::Book(book) => asset_ids_set.contains(&book.asset_id),
                                    WsMessage::PriceChange(price) => asset_ids_set.contains(&price.asset_id),
                                    WsMessage::LastTradePrice(ltp) => asset_ids_set.contains(&ltp.asset_id),
                                    WsMessage::TickSizeChange(tsc) => asset_ids_set.contains(&tsc.asset_id),
                                    _ => false,
                                };

                                if should_yield {
                                    yield Ok(msg.clone());
                                }
                            }
                            Err(e) => {
                                yield Err(WsError::InvalidMessage(e.to_string()).into());
                            }
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        warn!("Subscription lagged, missed {n} messages");
                        yield Err(WsError::Lagged { count: n }.into());
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
        auth: AuthPayload,
    ) -> Result<impl Stream<Item = Result<WsMessage>>> {
        self.interest.add(MessageInterest::USER);

        let new_markets: Vec<String> = markets
            .iter()
            .filter(|m| self.subscribed_markets.insert((*m).clone()))
            .cloned()
            .collect();

        // Only send subscription request for new markets (or if subscribing to all)
        if !markets.is_empty() && new_markets.is_empty() {
            debug!("All requested markets already subscribed, multiplexing");
        } else {
            debug!(
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

        Ok(stream! {
            loop {
                match rx.recv().await {
                    Ok(res) => {
                        match res.as_ref() {
                            Ok(msg) => {
                                // Only yield user messages
                                if msg.is_user() {
                                    yield Ok(msg.clone());
                                }
                            }
                            Err(e) => {
                                yield Err(WsError::InvalidMessage(e.to_string()).into());
                            }
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        warn!("Subscription lagged, missed {n} messages");
                        yield Err(WsError::Lagged { count: n }.into());
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
        let mut grouped: HashMap<ChannelType, Vec<SubscriptionInfo>> = HashMap::new();

        for entry in self.active_subs.iter() {
            grouped
                .entry(entry.value().channel())
                .or_default()
                .push(entry.value().clone());
        }

        grouped
    }

    /// Get the number of active subscriptions.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.active_subs.len()
    }
}
