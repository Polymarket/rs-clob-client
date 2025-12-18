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

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, Message>;
type WsStreamRead = SplitStream<WsStream>;

/// Manages WebSocket connection lifecycle, reconnection, and heartbeat.
pub struct ConnectionManager {
    /// Current connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Sender channel for outgoing messages
    sender_tx: mpsc::UnboundedSender<String>,
    /// Broadcast sender for incoming messages
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

impl ConnectionManager {
    /// Create a new connection manager and start the connection loop.
    pub fn new(endpoint: String, config: WebSocketConfig) -> Result<Self> {
        let (sender_tx, sender_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CAPACITY);

        let state = Arc::new(RwLock::new(ConnectionState::Disconnected));

        // Spawn connection task
        let connection_state = Arc::clone(&state);
        let connection_config = config;
        let connection_endpoint = endpoint;
        let broadcast_tx_clone = broadcast_tx.clone();

        tokio::spawn(async move {
            Self::connection_loop(
                connection_endpoint,
                connection_state,
                connection_config,
                sender_rx,
                broadcast_tx_clone,
            )
            .await;
        });

        Ok(Self {
            state,
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
    ) {
        let mut attempt = 0;

        loop {
            // Update state to connecting
            *state.write().await = ConnectionState::Connecting;

            // Attempt connection
            match connect_async(&endpoint).await {
                Ok((ws_stream, _)) => {
                    attempt = 0; // Reset on successful connection
                    *state.write().await = ConnectionState::Connected {
                        since: Instant::now(),
                    };

                    // Handle connection
                    match Self::handle_connection(
                        ws_stream,
                        &mut sender_rx,
                        &broadcast_tx,
                        Arc::clone(&state),
                        &config,
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
                *state.write().await = ConnectionState::Disconnected;
                break;
            }

            // Update state and calculate backoff
            *state.write().await = ConnectionState::Reconnecting { attempt };

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
            &state,
            pong_tx,
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
        _state: &Arc<RwLock<ConnectionState>>,
        pong_tx: watch::Sender<Instant>,
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

                            match parse_ws_text(&text) {
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
                    let last_pong = *pong_rx.borrow();
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
            .map_err(|_e| Error::validation("Connection closed"))?;
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
