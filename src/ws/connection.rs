#![expect(
    clippy::module_name_repetitions,
    reason = "Connection types expose their domain in the name for clarity"
)]

use std::result::Result as StdResult;
use std::sync::Arc;
use std::time::Instant;

use futures::{
    SinkExt as _, StreamExt as _,
    stream::{SplitSink, SplitStream},
};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock, broadcast, mpsc, watch};
use tokio::time::{interval, sleep, timeout};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use tracing::{debug, warn};

use super::config::WebSocketConfig;
use super::error::WsError;
use super::interest::InterestTracker;
use super::messages::{SubscriptionRequest, WsMessage, parse_ws_text};
use crate::{
    Result,
    error::{Error, Kind},
};

/// Broadcast message type.
pub type BroadcastMessage = Arc<Result<WsMessage>>;

/// Broadcast channel capacity for incoming messages.
const BROADCAST_CAPACITY: usize = 1024;

/// Connection state tracking.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Successfully connected
    Connected {
        /// When the connection was established
        since: Instant,
    },
    /// Reconnecting after failure
    Reconnecting {
        /// Current reconnection attempt number
        attempt: u32,
    },
}

impl ConnectionState {
    /// Check if the connection is currently active.
    #[must_use]
    pub const fn is_connected(self) -> bool {
        matches!(self, Self::Connected { .. })
    }
}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, Message>;
type WsStreamRead = SplitStream<WsStream>;

/// Manages WebSocket connection lifecycle, reconnection, and heartbeat.
pub struct ConnectionManager {
    /// Current connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Watch channel sender for state changes (enables reconnection detection)
    state_tx: watch::Sender<ConnectionState>,
    /// Sender channel for outgoing messages
    sender_tx: mpsc::UnboundedSender<String>,
    /// Broadcast sender for incoming messages
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

impl ConnectionManager {
    /// Create a new connection manager and start the connection loop.
    ///
    /// The `interest` tracker is used to determine which message types to deserialize.
    /// Only messages that have active consumers will be fully parsed.
    pub fn new(
        endpoint: String,
        config: WebSocketConfig,
        interest: &Arc<InterestTracker>,
    ) -> Result<Self> {
        let (sender_tx, sender_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        let (state_tx, _) = watch::channel(ConnectionState::Disconnected);

        let state = Arc::new(RwLock::new(ConnectionState::Disconnected));

        // Spawn connection task
        let connection_state = Arc::clone(&state);
        let connection_config = config;
        let connection_endpoint = endpoint;
        let broadcast_tx_clone = broadcast_tx.clone();
        let connection_interest = Arc::clone(interest);
        let state_tx_clone = state_tx.clone();

        tokio::spawn(async move {
            Self::connection_loop(
                connection_endpoint,
                connection_state,
                connection_config,
                sender_rx,
                broadcast_tx_clone,
                connection_interest,
                state_tx_clone,
            )
            .await;
        });

        Ok(Self {
            state,
            state_tx,
            sender_tx,
            broadcast_tx,
        })
    }

    /// Main connection loop with automatic reconnection.
    async fn connection_loop(
        endpoint: String,
        state: Arc<RwLock<ConnectionState>>,
        config: WebSocketConfig,
        mut sender_rx: mpsc::UnboundedReceiver<String>,
        broadcast_tx: broadcast::Sender<BroadcastMessage>,
        interest: Arc<InterestTracker>,
        state_tx: watch::Sender<ConnectionState>,
    ) {
        let mut attempt = 0;

        loop {
            // Update state to connecting
            let connecting = ConnectionState::Connecting;
            *state.write().await = connecting;
            let _: StdResult<_, _> = state_tx.send(connecting);

            // Attempt connection
            match connect_async(&endpoint).await {
                Ok((ws_stream, _)) => {
                    attempt = 0; // Reset on successful connection
                    let connected = ConnectionState::Connected {
                        since: Instant::now(),
                    };
                    *state.write().await = connected;
                    let _: StdResult<_, _> = state_tx.send(connected);

                    // Handle connection
                    match Self::handle_connection(
                        ws_stream,
                        &mut sender_rx,
                        &broadcast_tx,
                        Arc::clone(&state),
                        &config,
                        &interest,
                    )
                    .await
                    {
                        Ok(()) => {}
                        Err(e) => {
                            let _: StdResult<_, _> = broadcast_tx.send(Arc::new(Err(e)));
                        }
                    }
                }
                Err(e) => {
                    let error = Error::with_source(Kind::WebSocket, WsError::Connection(e));
                    let _: StdResult<_, _> = broadcast_tx.send(Arc::new(Err(error)));
                    attempt += 1;
                }
            }

            // Check if we should stop reconnecting
            if let Some(max) = config.reconnect.max_attempts
                && attempt >= max
            {
                let disconnected = ConnectionState::Disconnected;
                *state.write().await = disconnected;
                let _: StdResult<_, _> = state_tx.send(disconnected);
                break;
            }

            // Update state and calculate backoff
            let reconnecting = ConnectionState::Reconnecting { attempt };
            *state.write().await = reconnecting;
            let _: StdResult<_, _> = state_tx.send(reconnecting);

            let backoff = config.reconnect.calculate_backoff(attempt);
            sleep(backoff).await;
        }
    }

    /// Handle an active WebSocket connection.
    async fn handle_connection(
        ws_stream: WsStream,
        sender_rx: &mut mpsc::UnboundedReceiver<String>,
        broadcast_tx: &broadcast::Sender<BroadcastMessage>,
        state: Arc<RwLock<ConnectionState>>,
        config: &WebSocketConfig,
        interest: &Arc<InterestTracker>,
    ) -> Result<()> {
        let (write, read) = ws_stream.split();

        // Channel to notify heartbeat loop when PONG is received
        let (pong_tx, pong_rx) = watch::channel(Instant::now());

        // Spawn heartbeat task
        let heartbeat_config = config.clone();
        let write_for_heartbeat = Arc::new(Mutex::new(write));
        let write_for_messages = Arc::clone(&write_for_heartbeat);
        let heartbeat_state = Arc::clone(&state);

        let heartbeat_handle = tokio::spawn(async move {
            Self::heartbeat_loop(
                write_for_heartbeat,
                heartbeat_state,
                &heartbeat_config,
                pong_rx,
            )
            .await;
        });

        // Message handling loop
        let result = Self::message_loop(
            read,
            write_for_messages,
            sender_rx,
            broadcast_tx,
            pong_tx,
            interest,
        )
        .await;

        // Cleanup
        heartbeat_handle.abort();

        result
    }

    /// Main message handling loop.
    async fn message_loop(
        mut read: WsStreamRead,
        write: Arc<Mutex<WsSink>>,
        sender_rx: &mut mpsc::UnboundedReceiver<String>,
        broadcast_tx: &broadcast::Sender<BroadcastMessage>,
        pong_tx: watch::Sender<Instant>,
        interest: &Arc<InterestTracker>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                // Handle incoming messages
                Some(msg) = read.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            // Notify heartbeat loop when PONG is received
                            if text == "PONG" {
                                let _: StdResult<(), _> = pong_tx.send(Instant::now());
                                continue;
                            }

                            // Only deserialize message types that have active consumers
                            match parse_ws_text(&text, interest.get()) {
                                Ok(messages) => {
                                    for ws_msg in messages {
                                        let _: StdResult<_, _> = broadcast_tx.send(Arc::new(Ok(ws_msg)));
                                    }
                                }
                                Err(e) => {
                                    warn!(%text, error = %e, "Failed to parse WebSocket message");
                                    let err = Error::with_source(
                                        Kind::WebSocket,
                                        WsError::MessageParse(e),
                                    );
                                    let _: StdResult<_, _> = broadcast_tx.send(Arc::new(Err(err)));
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            let err = Error::with_source(
                                Kind::WebSocket,
                                WsError::ConnectionClosed,
                            );
                            let _: StdResult<_, _> = broadcast_tx.send(Arc::new(Err(err)));
                            break;
                        }
                        Err(e) => {
                            let err = Error::with_source(
                                Kind::WebSocket,
                                WsError::Connection(e),
                            );
                            let _: StdResult<_, _> = broadcast_tx.send(Arc::new(Err(err)));
                            break;
                        }
                        _ => {
                            // Ignore binary frames and unsolicited PONG replies.
                        }
                    }
                }

                // Handle outgoing messages
                Some(text) = sender_rx.recv() => {
                    let mut write_guard = write.lock().await;
                    if write_guard.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }

                // Check if connection is still active
                else => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Heartbeat loop that sends PING messages and monitors PONG responses.
    async fn heartbeat_loop(
        write: Arc<Mutex<WsSink>>,
        state: Arc<RwLock<ConnectionState>>,
        config: &WebSocketConfig,
        mut pong_rx: watch::Receiver<Instant>,
    ) {
        let mut ping_interval = interval(config.heartbeat_interval);

        loop {
            ping_interval.tick().await;

            // Check if still connected
            if !matches!(*state.read().await, ConnectionState::Connected { .. }) {
                break;
            }

            // Mark current PONG state as seen before sending PING
            // This prevents changed() from returning immediately due to a stale PONG
            drop(pong_rx.borrow_and_update());

            // Send PING
            let ping_sent = Instant::now();
            {
                let mut write_guard = write.lock().await;
                if write_guard
                    .send(Message::Text("PING".into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }

            // Wait for PONG within timeout
            let pong_result = timeout(config.heartbeat_timeout, pong_rx.changed()).await;

            match pong_result {
                Ok(Ok(())) => {
                    let last_pong = *pong_rx.borrow_and_update();
                    if last_pong < ping_sent {
                        debug!("PONG received but older than last PING, connection may be stale");
                        break;
                    }
                }
                Ok(Err(_)) => {
                    // Channel closed, connection is terminating
                    break;
                }
                Err(_) => {
                    // Timeout waiting for PONG
                    warn!(
                        "Heartbeat timeout: no PONG received within {:?}",
                        config.heartbeat_timeout
                    );
                    break;
                }
            }
        }
    }

    /// Send a subscription request to the WebSocket server.
    pub fn send(&self, message: &SubscriptionRequest) -> Result<()> {
        let json = serde_json::to_string(message)?;
        self.sender_tx
            .send(json)
            .map_err(|_e| WsError::ConnectionClosed)?;
        Ok(())
    }

    /// Get the current connection state.
    #[must_use]
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Subscribe to incoming messages.
    ///
    /// Each call returns a new independent receiver. Multiple subscribers can
    /// receive messages concurrently without blocking each other.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastMessage> {
        self.broadcast_tx.subscribe()
    }

    /// Subscribe to connection state changes.
    ///
    /// Returns a receiver that notifies when the connection state changes.
    /// This is useful for detecting reconnections and re-establishing subscriptions.
    #[must_use]
    pub fn state_receiver(&self) -> watch::Receiver<ConnectionState> {
        self.state_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_state_transitions() {
        let state = ConnectionState::Disconnected;
        assert_eq!(state, ConnectionState::Disconnected);

        let state = ConnectionState::Connecting;
        assert_eq!(state, ConnectionState::Connecting);

        let state = ConnectionState::Connected {
            since: Instant::now(),
        };
        assert!(matches!(state, ConnectionState::Connected { .. }));
    }
}
