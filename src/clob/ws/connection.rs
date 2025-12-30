#![expect(
    clippy::module_name_repetitions,
    reason = "Connection types expose their domain in the name for clarity"
)]

use std::fmt::Write as _;
use std::sync::Arc;
use std::time::Instant;

use backoff::backoff::Backoff as _;
use base64::Engine as _;
use futures::{SinkExt as _, StreamExt as _};
use secrecy::ExposeSecret as _;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, watch};
use tokio::time::{interval, sleep, timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, client_async_tls, connect_async, tungstenite::Message,
};
use url::Url;

use super::config::Config;
use super::error::WsError;
use super::interest::InterestTracker;
use super::types::request::SubscriptionRequest;
use super::types::response::{WsMessage, parse_if_interested};
use crate::{
    Result,
    error::{Error, Kind},
};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

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

/// Manages WebSocket connection lifecycle, reconnection, and heartbeat.
#[derive(Clone)]
pub struct ConnectionManager {
    /// Watch channel sender for state changes (enables reconnection detection)
    state_tx: watch::Sender<ConnectionState>,
    /// Watch channel receiver for state changes (for use in checking the current state)
    state_rx: watch::Receiver<ConnectionState>,
    /// Sender channel for outgoing messages
    sender_tx: mpsc::UnboundedSender<String>,
    /// Broadcast sender for incoming messages
    broadcast_tx: broadcast::Sender<WsMessage>,
}

impl ConnectionManager {
    /// Create a new connection manager and start the connection loop.
    ///
    /// The `interest` tracker is used to determine which message types to deserialize.
    /// Only messages that have active consumers will be fully parsed.
    pub fn new(endpoint: String, config: Config, interest: &Arc<InterestTracker>) -> Result<Self> {
        let (sender_tx, sender_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        let (state_tx, state_rx) = watch::channel(ConnectionState::Disconnected);

        // Spawn connection task
        let connection_config = config;
        let connection_endpoint = endpoint;
        let broadcast_tx_clone = broadcast_tx.clone();
        let connection_interest = Arc::clone(interest);
        let state_tx_clone = state_tx.clone();

        tokio::spawn(async move {
            Self::connection_loop(
                connection_endpoint,
                connection_config,
                sender_rx,
                broadcast_tx_clone,
                connection_interest,
                state_tx_clone,
            )
            .await;
        });

        Ok(Self {
            state_tx,
            state_rx,
            sender_tx,
            broadcast_tx,
        })
    }

    /// Main connection loop with automatic reconnection.
    async fn connection_loop(
        endpoint: String,
        config: Config,
        mut sender_rx: mpsc::UnboundedReceiver<String>,
        broadcast_tx: broadcast::Sender<WsMessage>,
        interest: Arc<InterestTracker>,
        state_tx: watch::Sender<ConnectionState>,
    ) {
        let mut attempt = 0_u32;
        let mut backoff: backoff::ExponentialBackoff = config.reconnect.clone().into();

        loop {
            let state_rx = state_tx.subscribe();

            _ = state_tx.send(ConnectionState::Connecting);

            // Attempt connection (with or without proxy)
            let connect_result = if let Some(proxy_url) = &config.proxy {
                connect_via_proxy(&endpoint, proxy_url).await
            } else {
                connect_async(&endpoint)
                    .await
                    .map(|(ws, _)| ws)
                    .map_err(|e| Error::with_source(Kind::WebSocket, WsError::Connection(e)))
            };

            match connect_result {
                Ok(ws_stream) => {
                    attempt = 0;
                    backoff.reset();
                    _ = state_tx.send(ConnectionState::Connected {
                        since: Instant::now(),
                    });

                    // Handle connection
                    if let Err(e) = Self::handle_connection(
                        ws_stream,
                        &mut sender_rx,
                        &broadcast_tx,
                        state_rx,
                        config.clone(),
                        &interest,
                    )
                    .await
                    {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Error handling connection: {e:?}");
                        #[cfg(not(feature = "tracing"))]
                        let _ = &e;
                    }
                }
                Err(e) => {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("Unable to connect: {e:?}");
                    #[cfg(not(feature = "tracing"))]
                    let _ = &e;
                    attempt = attempt.saturating_add(1);
                }
            }

            // Check if we should stop reconnecting
            if let Some(max) = config.reconnect.max_attempts
                && attempt >= max
            {
                _ = state_tx.send(ConnectionState::Disconnected);
                break;
            }

            // Update state and wait with exponential backoff
            _ = state_tx.send(ConnectionState::Reconnecting { attempt });

            if let Some(duration) = backoff.next_backoff() {
                sleep(duration).await;
            }
        }
    }

    /// Handle an active WebSocket connection.
    async fn handle_connection(
        ws_stream: WsStream,
        sender_rx: &mut mpsc::UnboundedReceiver<String>,
        broadcast_tx: &broadcast::Sender<WsMessage>,
        state_rx: watch::Receiver<ConnectionState>,
        config: Config,
        interest: &Arc<InterestTracker>,
    ) -> Result<()> {
        let (mut write, mut read) = ws_stream.split();

        // Channel to notify heartbeat loop when PONG is received
        let (pong_tx, pong_rx) = watch::channel(Instant::now());

        let (ping_tx, mut ping_rx) = mpsc::unbounded_channel();

        let heartbeat_handle = tokio::spawn(async move {
            Self::heartbeat_loop(ping_tx, state_rx, &config, pong_rx).await;
        });

        loop {
            tokio::select! {
                // Handle incoming messages
                Some(msg) = read.next() => {
                    match msg {
                        Ok(Message::Text(text)) if text == "PONG" => {
                            _ = pong_tx.send(Instant::now());
                        }
                        Ok(Message::Text(text)) => {
                            #[cfg(feature = "tracing")]
                            tracing::trace!(%text, "Received WebSocket text message");

                            // Only deserialize message types that have active consumers
                            match parse_if_interested(text.as_bytes(), &interest.get()) {
                                Ok(messages) => {
                                    for message in messages {
                                        #[cfg(feature = "tracing")]
                                        tracing::trace!(?message, "Parsed WebSocket message");
                                        _ = broadcast_tx.send(message);
                                    }

                                }
                                Err(e) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::warn!(%text, error = %e, "Failed to parse WebSocket message");
                                    #[cfg(not(feature = "tracing"))]
                                    let _ = (&text, &e);
                                }
                            }
                        }
                        Ok(Message::Close(_)) => {
                            heartbeat_handle.abort();
                            return Err(Error::with_source(
                                Kind::WebSocket,
                                WsError::ConnectionClosed,
                            ))
                        }
                        Err(e) => {
                            heartbeat_handle.abort();
                            return Err(Error::with_source(
                                Kind::WebSocket,
                                WsError::Connection(e),
                            ));
                        }
                        _ => {
                            // Ignore binary frames and unsolicited PONG replies.
                        }
                    }
                }

                // Handle outgoing messages from subscriptions
                Some(text) = sender_rx.recv() => {
                    if write.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }

                // Handle PING requests from heartbeat loop
                Some(()) = ping_rx.recv() => {
                    if write.send(Message::Text("PING".into())).await.is_err() {
                        break;
                    }
                }

                // Check if connection is still active
                else => {
                    break;
                }
            }
        }

        // Cleanup
        heartbeat_handle.abort();

        Ok(())
    }

    /// Heartbeat loop that sends PING messages and monitors PONG responses.
    async fn heartbeat_loop(
        ping_tx: mpsc::UnboundedSender<()>,
        state_rx: watch::Receiver<ConnectionState>,
        config: &Config,
        mut pong_rx: watch::Receiver<Instant>,
    ) {
        let mut ping_interval = interval(config.heartbeat_interval);

        loop {
            ping_interval.tick().await;

            // Check if still connected
            if !state_rx.borrow().is_connected() {
                break;
            }

            // Mark current PONG state as seen before sending PING
            // This prevents changed() from returning immediately due to a stale PONG
            drop(pong_rx.borrow_and_update());

            // Send PING request to message loop
            let ping_sent = Instant::now();
            if ping_tx.send(()).is_err() {
                // Message loop has terminated
                break;
            }

            // Wait for PONG within timeout
            let pong_result = timeout(config.heartbeat_timeout, pong_rx.changed()).await;

            match pong_result {
                Ok(Ok(())) => {
                    let last_pong = *pong_rx.borrow_and_update();
                    if last_pong < ping_sent {
                        #[cfg(feature = "tracing")]
                        tracing::debug!(
                            "PONG received but older than last PING, connection may be stale"
                        );
                        break;
                    }
                }
                Ok(Err(_)) => {
                    // Channel closed, connection is terminating
                    break;
                }
                Err(_) => {
                    // Timeout waiting for PONG
                    #[cfg(feature = "tracing")]
                    tracing::warn!(
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
        let mut v = serde_json::to_value(message)?;

        // Only expose credentials when serializing on the wire, otherwise do not include
        // credentials in other serialization contexts
        if let Some(creds) = message.auth.as_ref() {
            let auth = json!({
                "apiKey": creds.key.to_string(),
                "secret": creds.secret.expose_secret(),
                "passphrase": creds.passphrase.expose_secret(),
            });

            if let Value::Object(ref mut obj) = v {
                obj.insert("auth".to_owned(), auth);
            }
        }

        let json = serde_json::to_string(&v)?;
        self.sender_tx
            .send(json)
            .map_err(|_e| WsError::ConnectionClosed)?;
        Ok(())
    }

    /// Get the current connection state.
    #[must_use]
    pub fn state(&self) -> ConnectionState {
        *self.state_rx.borrow()
    }

    /// Subscribe to incoming messages.
    ///
    /// Each call returns a new independent receiver. Multiple subscribers can
    /// receive messages concurrently without blocking each other.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
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

/// Connect to a WebSocket endpoint via a proxy.
///
/// Supports:
/// - SOCKS5 proxies: `socks5://[user:pass@]host:port`
/// - HTTP CONNECT proxies: `http://[user:pass@]host:port`
async fn connect_via_proxy(endpoint: &str, proxy_url: &str) -> Result<WsStream> {
    let proxy = Url::parse(proxy_url)
        .map_err(|e| Error::validation(format!("invalid proxy URL '{proxy_url}': {e}")))?;

    let target = Url::parse(endpoint)
        .map_err(|e| Error::validation(format!("invalid WebSocket URL '{endpoint}': {e}")))?;

    let target_host = target
        .host_str()
        .ok_or_else(|| Error::validation("WebSocket URL missing host"))?;
    let target_port = target.port_or_known_default().unwrap_or(443);

    let stream = match proxy.scheme() {
        "socks5" | "socks5h" => connect_socks5(&proxy, target_host, target_port).await?,
        "http" | "https" => connect_http_tunnel(&proxy, target_host, target_port).await?,
        scheme => {
            return Err(Error::validation(format!(
                "unsupported proxy scheme '{scheme}', expected socks5 or http"
            )));
        }
    };

    // Upgrade the TCP connection to WebSocket with TLS
    let (ws_stream, _) = client_async_tls(endpoint, stream)
        .await
        .map_err(|e| Error::with_source(Kind::WebSocket, WsError::Connection(e)))?;

    Ok(ws_stream)
}

/// Connect through a SOCKS5 proxy.
async fn connect_socks5(proxy: &Url, target_host: &str, target_port: u16) -> Result<TcpStream> {
    let proxy_host = proxy
        .host_str()
        .ok_or_else(|| Error::validation("SOCKS5 proxy URL missing host"))?;
    let proxy_port = proxy.port().unwrap_or(1080);
    let proxy_addr = format!("{proxy_host}:{proxy_port}");

    let stream = if proxy.username().is_empty() {
        tokio_socks::tcp::Socks5Stream::connect(proxy_addr.as_str(), (target_host, target_port))
            .await
            .map_err(|e| Error::validation(format!("SOCKS5 connection failed: {e}")))?
    } else {
        let password = proxy.password().unwrap_or("");
        tokio_socks::tcp::Socks5Stream::connect_with_password(
            proxy_addr.as_str(),
            (target_host, target_port),
            proxy.username(),
            password,
        )
        .await
        .map_err(|e| Error::validation(format!("SOCKS5 connection failed: {e}")))?
    };

    Ok(stream.into_inner())
}

/// Connect through an HTTP CONNECT tunnel.
async fn connect_http_tunnel(
    proxy: &Url,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream> {
    let proxy_host = proxy
        .host_str()
        .ok_or_else(|| Error::validation("HTTP proxy URL missing host"))?;
    let proxy_port = proxy.port().unwrap_or(8080);

    // Connect to the proxy
    let mut stream = TcpStream::connect((proxy_host, proxy_port))
        .await
        .map_err(|e| Error::validation(format!("failed to connect to HTTP proxy: {e}")))?;

    // Build CONNECT request
    let mut connect_request = format!(
        "CONNECT {target_host}:{target_port} HTTP/1.1\r\n\
         Host: {target_host}:{target_port}\r\n"
    );

    // Add proxy authentication if provided
    if !proxy.username().is_empty() {
        let credentials = format!("{}:{}", proxy.username(), proxy.password().unwrap_or(""));
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        write!(connect_request, "Proxy-Authorization: Basic {encoded}\r\n")
            .map_err(|e| Error::validation(format!("failed to format CONNECT request: {e}")))?;
    }

    connect_request.push_str("\r\n");

    // Send CONNECT request
    stream
        .write_all(connect_request.as_bytes())
        .await
        .map_err(|e| Error::validation(format!("failed to send CONNECT request: {e}")))?;

    // Read response
    let mut buf = [0_u8; 1024];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| Error::validation(format!("failed to read CONNECT response: {e}")))?;

    let response = String::from_utf8_lossy(&buf[..n]);

    // Check for successful tunnel establishment (HTTP 200)
    if !response.starts_with("HTTP/1.1 200") && !response.starts_with("HTTP/1.0 200") {
        return Err(Error::validation(format!(
            "HTTP CONNECT tunnel failed: {}",
            response.lines().next().unwrap_or("unknown error")
        )));
    }

    Ok(stream)
}
