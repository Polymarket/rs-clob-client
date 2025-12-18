#![expect(
    clippy::module_name_repetitions,
    reason = "Subscription types deliberately include the module name for clarity"
)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use async_stream::stream;
use dashmap::DashMap;
use futures::Stream;
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;

use super::connection::ConnectionManager;
use super::messages::{AuthPayload, SubscriptionRequest, WsMessage};
use crate::Result;
use crate::error::Error;

/// Information about an active subscription.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SubscriptionInfo {
    /// Channel type subscribed to
    pub channel: ChannelType,
    /// Asset IDs subscribed to
    pub asset_ids: Vec<String>,
    /// When the subscription was created
    pub created_at: Instant,
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
}

impl SubscriptionManager {
    /// Create a new subscription manager.
    #[must_use]
    pub fn new(connection: Arc<ConnectionManager>) -> Self {
        Self {
            connection,
            active_subs: Arc::new(DashMap::new()),
        }
    }

    /// Subscribe to public market data channel.
    pub fn subscribe_market(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<impl Stream<Item = Result<WsMessage>>> {
        // Send subscription request
        let request = SubscriptionRequest::market(asset_ids.clone());
        self.connection.send(&request)?;

        // Register subscription
        let sub_id = format!("market:{}", asset_ids.join(","));
        self.active_subs.insert(
            sub_id,
            SubscriptionInfo {
                channel: ChannelType::Market,
                asset_ids: asset_ids.clone(),
                created_at: Instant::now(),
            },
        );

        // Create filtered stream with its own receiver
        let mut rx = self.connection.subscribe();
        let asset_ids_set: HashSet<String> = asset_ids.into_iter().collect();

        Ok(stream! {
            loop {
                match rx.recv().await {
                    Ok(arc_result) => {
                        match arc_result.as_ref() {
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
                                yield Err(Error::validation(e.to_string()));
                            }
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        warn!("Subscription lagged, missed {n} messages");
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
        // Send authenticated subscription request
        let request = SubscriptionRequest::user(markets, auth);
        self.connection.send(&request)?;

        // Register subscription
        let sub_id = "user:authenticated".to_owned();
        self.active_subs.insert(
            sub_id,
            SubscriptionInfo {
                channel: ChannelType::User,
                asset_ids: vec![],
                created_at: Instant::now(),
            },
        );

        // Create stream for user messages
        let mut rx = self.connection.subscribe();

        Ok(stream! {
            loop {
                match rx.recv().await {
                    Ok(arc_result) => {
                        match arc_result.as_ref() {
                            Ok(msg) => {
                                // Only yield user messages
                                if msg.is_user() {
                                    yield Ok(msg.clone());
                                }
                            }
                            Err(e) => {
                                yield Err(Error::validation(e.to_string()));
                            }
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        warn!("Subscription lagged, missed {n} messages");
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
                .entry(entry.value().channel)
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
