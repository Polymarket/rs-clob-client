#![expect(
    clippy::module_name_repetitions,
    reason = "Subscription types deliberately include the module name for clarity"
)]

use std::sync::{Arc, PoisonError, RwLock};
use std::time::Instant;

use async_stream::try_stream;
use dashmap::{DashMap, Entry};
use futures::Stream;
use tokio::sync::broadcast::error::RecvError;

use super::error::RtdsError;
use super::types::request::{Subscription, SubscriptionRequest};
use super::types::response::{RtdsMessage, parse_messages};
use crate::Result;
use crate::auth::Credentials;
use crate::ws::ConnectionManager;
use crate::ws::connection::ConnectionState;

#[non_exhaustive]
#[derive(Clone)]
pub struct SimpleParser;

impl crate::ws::traits::MessageParser<RtdsMessage> for SimpleParser {
    fn parse(&self, bytes: &[u8]) -> Result<Vec<RtdsMessage>> {
        parse_messages(bytes)
    }
}

/// Unique identifier for a topic/type subscription combination.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicType {
    /// Topic name (e.g., `crypto_prices`, `comments`)
    pub topic: String,
    /// Message type (e.g., `update`, `comment_created`, `*`)
    pub msg_type: String,
}

impl TopicType {
    /// Create a new topic/type identifier.
    #[must_use]
    pub fn new(topic: String, msg_type: String) -> Self {
        Self { topic, msg_type }
    }
}

/// Information about an active subscription.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SubscriptionInfo {
    /// Topic and message type this subscription targets
    pub topic_type: TopicType,
    /// Optional filters for this subscription
    pub filters: Option<String>,
    /// CLOB authentication if required
    pub clob_auth: Option<Credentials>,
    /// When the subscription was created
    pub created_at: Instant,
}

/// Manages active subscriptions and routes messages to subscribers.
pub struct SubscriptionManager {
    connection: ConnectionManager<RtdsMessage, SimpleParser>,
    active_subs: DashMap<String, SubscriptionInfo>,
    /// Subscribed topics with reference counts (for multiplexing)
    subscribed_topics: DashMap<TopicType, usize>,
    last_auth: RwLock<Option<Credentials>>,
}

impl SubscriptionManager {
    /// Create a new subscription manager.
    #[must_use]
    pub fn new(connection: ConnectionManager<RtdsMessage, SimpleParser>) -> Self {
        Self {
            connection,
            active_subs: DashMap::new(),
            subscribed_topics: DashMap::new(),
            last_auth: RwLock::new(None),
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
                            tracing::debug!("RTDS reconnected, re-establishing subscriptions");
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

    /// Re-send subscription requests for all tracked topics.
    fn resubscribe_all(&self) {
        // Get stored auth for re-subscription on reconnect.
        // We can recover from poisoned lock because Option<Credentials> has no inconsistent intermediate state.
        let auth = self
            .last_auth
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();

        let subscriptions: Vec<Subscription> = self
            .active_subs
            .iter()
            .map(|entry| {
                let info = entry.value();
                let mut sub = Subscription {
                    topic: info.topic_type.topic.clone(),
                    msg_type: info.topic_type.msg_type.clone(),
                    filters: info.filters.clone(),
                    clob_auth: None,
                };
                // Apply stored auth if subscription originally had auth
                if info.clob_auth.is_some()
                    && let Some(creds) = &auth
                {
                    sub = sub.with_clob_auth(creds.clone());
                }
                sub
            })
            .collect();

        if subscriptions.is_empty() {
            return;
        }

        #[cfg(feature = "tracing")]
        tracing::debug!(count = subscriptions.len(), "Re-subscribing to RTDS topics");

        let request = SubscriptionRequest::subscribe(subscriptions);
        if let Err(e) = self.connection.send(&request) {
            #[cfg(feature = "tracing")]
            tracing::warn!(%e, "Failed to re-subscribe to RTDS topics");
            #[cfg(not(feature = "tracing"))]
            let _: &crate::error::Error = &e;
        }
    }

    /// Subscribe to a topic with the given configuration.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Subscription is consumed to build SubscriptionInfo"
    )]
    pub fn subscribe(
        &self,
        subscription: Subscription,
    ) -> Result<impl Stream<Item = Result<RtdsMessage>>> {
        let topic_type = TopicType::new(subscription.topic.clone(), subscription.msg_type.clone());

        // Store auth for re-subscription on reconnect.
        // We can recover from poisoned lock because Option<Credentials> has no inconsistent intermediate state.
        if let Some(auth) = &subscription.clob_auth {
            *self
                .last_auth
                .write()
                .unwrap_or_else(PoisonError::into_inner) = Some(auth.clone());
        }

        // Increment refcount or insert new topic with refcount=1
        // Using Entry API to atomically check and update, with send inside the guard
        // to prevent TOCTOU race between refcount check and network send
        match self.subscribed_topics.entry(topic_type.clone()) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() += 1;
                #[cfg(feature = "tracing")]
                tracing::debug!(
                    topic = %subscription.topic,
                    msg_type = %subscription.msg_type,
                    "RTDS topic already subscribed, multiplexing"
                );
            }
            Entry::Vacant(entry) => {
                #[cfg(feature = "tracing")]
                tracing::debug!(
                    topic = %subscription.topic,
                    msg_type = %subscription.msg_type,
                    "Subscribing to RTDS topic"
                );

                // Send subscribe request while holding the entry lock to prevent
                // a concurrent unsubscribe from racing with us
                let request = SubscriptionRequest::subscribe(vec![subscription.clone()]);
                self.connection.send(&request)?;
                // Only insert after successful send
                entry.insert(1);
            }
        }

        // Register subscription info
        let sub_id = format!("{}:{}", topic_type.topic, topic_type.msg_type);
        self.active_subs.insert(
            sub_id,
            SubscriptionInfo {
                topic_type: topic_type.clone(),
                filters: subscription.filters.clone(),
                clob_auth: subscription.clob_auth.clone(),
                created_at: Instant::now(),
            },
        );

        // Create filtered stream with its own receiver
        let mut rx = self.connection.subscribe();
        let target_topic = topic_type.topic;
        let target_type = topic_type.msg_type;

        Ok(try_stream! {
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        // Filter messages by topic and type
                        let matches_topic = msg.topic == target_topic;
                        let matches_type = target_type == "*" || msg.msg_type == target_type;

                        if matches_topic && matches_type {
                            yield msg;
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        #[cfg(feature = "tracing")]
                        tracing::warn!("RTDS subscription lagged, missed {n} messages");
                        Err(RtdsError::Lagged { count: n })?;
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
    pub fn active_subscriptions(&self) -> Vec<SubscriptionInfo> {
        self.active_subs
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get the number of active subscriptions.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.active_subs.len()
    }

    /// Unsubscribe from topics.
    ///
    /// This decrements the reference count for each topic. Only sends an unsubscribe
    /// request to the server when the reference count reaches zero (no other streams
    /// are using that topic).
    pub fn unsubscribe(&self, topic_types: &[TopicType]) -> Result<()> {
        if topic_types.is_empty() {
            return Err(RtdsError::SubscriptionFailed(
                "topic_types cannot be empty: at least one topic must be provided for unsubscription"
                    .to_owned(),
            )
            .into());
        }

        // Atomically decrement refcounts and send unsubscribe while holding the entry lock
        // to prevent TOCTOU race between refcount check and network send
        for topic_type in topic_types {
            if let Entry::Occupied(mut entry) = self.subscribed_topics.entry(topic_type.clone()) {
                let refcount = entry.get_mut();
                *refcount = refcount.saturating_sub(1);
                if *refcount == 0 {
                    #[cfg(feature = "tracing")]
                    tracing::debug!(
                        topic = %topic_type.topic,
                        msg_type = %topic_type.msg_type,
                        "Unsubscribing from RTDS topic"
                    );

                    // Send unsubscribe while holding the entry lock to prevent
                    // a concurrent subscribe from racing with us
                    let request = SubscriptionRequest::unsubscribe(vec![Subscription {
                        topic: topic_type.topic.clone(),
                        msg_type: topic_type.msg_type.clone(),
                        filters: None,
                        clob_auth: None,
                    }]);
                    self.connection.send(&request)?;
                    entry.remove();
                }
            }
        }

        // Remove active_subs entries where all topics are now unsubscribed
        self.active_subs
            .retain(|_, info| self.subscribed_topics.contains_key(&info.topic_type));

        Ok(())
    }
}
