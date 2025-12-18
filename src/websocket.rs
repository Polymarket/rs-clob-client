//! WebSocket client implementations for Polymarket CLOB API
//!
//! This module provides WebSocket connectivity for both market data (unauthenticated)
//! and user-specific events (authenticated).
//!
//! # Market WebSocket
//!
//! The market WebSocket provides real-time order book updates and price changes:
//!
//! ```no_run
//! use polymarket_client_sdk::clob::{Client, Config};
//! use polymarket_client_sdk::websocket::MarketWebSocket;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = Client::new("https://clob.polymarket.com", Config::default())?;
//!     let mut ws = client.market_websocket().await?;
//!     
//!     // Subscribe to market updates for specific assets
//!     ws.subscribe(&["asset_id_1", "asset_id_2"]).await?;
//!     
//!     // Receive messages
//!     while let Some(msg) = ws.next().await {
//!         println!("Received: {:?}", msg);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! # User WebSocket
//!
//! The user WebSocket provides authenticated, user-specific events like trades and orders:
//!
//! ```no_run
//! use polymarket_client_sdk::clob::{Client, Config};
//! use polymarket_client_sdk::websocket::UserWebSocket;
//! use alloy::signers::local::LocalSigner;
//! use std::str::FromStr;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = Client::new("https://clob.polymarket.com", Config::default())?;
//!     let signer = LocalSigner::from_str("private_key")?;
//!     
//!     let authenticated_client = client
//!         .authentication_builder(&signer)
//!         .authenticate()
//!         .await?;
//!     
//!     let mut ws = authenticated_client.user_websocket().await?;
//!     
//!     // Receive user events
//!     while let Some(msg) = ws.next().await {
//!         println!("Received: {:?}", msg);
//!     }
//!     
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt, stream::SplitSink, stream::SplitStream};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use tracing::debug;

use crate::Result;
use crate::auth::Credentials;
use crate::error::Error;
use crate::types::{MarketWebSocketMessage, UserWebSocketMessage};

/// Default WebSocket URL for market data
const DEFAULT_MARKET_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

/// Default WebSocket URL for user events
const DEFAULT_USER_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/user";

/// Maximum reconnection backoff duration in seconds
const MAX_BACKOFF_SECS: u64 = 30;

/// Initial reconnection backoff duration in seconds
const INITIAL_BACKOFF_SECS: u64 = 1;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, Message>;
type WsSource = SplitStream<WsStream>;

/// Market WebSocket client for receiving real-time market data (unauthenticated)
///
/// Provides access to order book updates, price changes, and last trade prices
/// for subscribed assets.
pub struct MarketWebSocket {
    url: String,
    writer: Arc<Mutex<Option<WsSink>>>,
    reader: Arc<Mutex<Option<WsSource>>>,
    subscribed_assets: Arc<Mutex<Vec<String>>>,
    auto_reconnect: bool,
}

impl MarketWebSocket {
    /// Creates a new market WebSocket client with the default URL
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection cannot be established
    pub async fn new() -> Result<Self> {
        Self::with_url(DEFAULT_MARKET_WS_URL).await
    }

    /// Creates a new market WebSocket client with a custom URL
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket endpoint URL
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection cannot be established
    pub async fn with_url(url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(url).await.map_err(|e| {
            Error::validation(format!("Failed to connect to market WebSocket: {e}"))
        })?;

        let (writer, reader) = ws_stream.split();

        Ok(Self {
            url: url.to_owned(),
            writer: Arc::new(Mutex::new(Some(writer))),
            reader: Arc::new(Mutex::new(Some(reader))),
            subscribed_assets: Arc::new(Mutex::new(Vec::new())),
            auto_reconnect: true,
        })
    }

    /// Subscribes to market updates for the specified asset IDs
    ///
    /// # Arguments
    ///
    /// * `asset_ids` - List of asset IDs to subscribe to
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription message cannot be sent
    pub async fn subscribe(&mut self, asset_ids: &[&str]) -> Result<()> {
        let asset_ids_owned: Vec<String> = asset_ids.iter().map(|s| (*s).to_owned()).collect();

        let subscription_message = json!({
            "assets_ids": asset_ids_owned
        });

        let mut writer = self.writer.lock().await;
        if let Some(ws) = writer.as_mut() {
            ws.send(Message::Text(subscription_message.to_string().into()))
                .await
                .map_err(|e| Error::validation(format!("Failed to send subscription: {e}")))?;

            drop(writer);

            // Update subscribed assets list
            let mut subscribed = self.subscribed_assets.lock().await;
            subscribed.extend(asset_ids_owned);
        } else {
            return Err(Error::validation("WebSocket not connected"));
        }

        Ok(())
    }

    /// Receives the next market message from the WebSocket
    ///
    /// Returns `None` if the connection is closed or an error occurs
    ///
    /// # Errors
    ///
    /// Returns an error if message parsing fails
    pub async fn next(&mut self) -> Option<Result<Vec<MarketWebSocketMessage>>> {
        loop {
            let mut reader = self.reader.lock().await;
            let reader_ref = reader.as_mut()?;

            match reader_ref.next().await {
                Some(Ok(Message::Text(text))) => {
                    drop(reader);

                    match self.parse_market_messages(&text) {
                        Ok(msgs) => return Some(Ok(msgs)),
                        Err(e) => return Some(Err(e)),
                    }
                }
                Some(Ok(Message::Ping(data))) => {
                    drop(reader);
                    let mut writer = self.writer.lock().await;
                    if let Some(ws) = writer.as_mut() {
                        let _ = ws.send(Message::Pong(data)).await;
                    }
                }
                Some(Ok(Message::Close(_))) => {
                    drop(reader);
                    if self.auto_reconnect {
                        let _ = self.reconnect().await;
                        continue;
                    }
                    return None;
                }
                Some(Err(_)) => {
                    drop(reader);
                    if self.auto_reconnect {
                        let _ = self.reconnect().await;
                        continue;
                    }
                    return None;
                }
                None => {
                    drop(reader);
                    if self.auto_reconnect {
                        let _ = self.reconnect().await;
                        continue;
                    }
                    return None;
                }
                _ => {
                    // Ignore other message types
                    drop(reader);
                }
            }
        }
    }

    /// Closes the WebSocket connection
    ///
    /// # Errors
    ///
    /// Returns an error if the close message cannot be sent
    pub async fn close(&mut self) -> Result<()> {
        self.auto_reconnect = false;
        let mut writer = self.writer.lock().await;
        if let Some(ws) = writer.as_mut() {
            ws.close()
                .await
                .map_err(|e| Error::validation(format!("Failed to close WebSocket: {e}")))?;
        }
        Ok(())
    }

    async fn reconnect(&mut self) -> Result<()> {
        let mut backoff = INITIAL_BACKOFF_SECS;

        loop {
            sleep(Duration::from_secs(backoff)).await;

            match connect_async(&self.url).await {
                Ok((ws_stream, _)) => {
                    let (writer, reader) = ws_stream.split();

                    *self.writer.lock().await = Some(writer);
                    *self.reader.lock().await = Some(reader);

                    // Re-subscribe to previously subscribed assets
                    let subscribed = self.subscribed_assets.lock().await.clone();
                    if !subscribed.is_empty() {
                        let refs: Vec<&str> = subscribed.iter().map(String::as_str).collect();
                        let _ = self.subscribe(&refs).await;
                    }

                    return Ok(());
                }
                Err(_) => {
                    backoff = (backoff * 2).min(MAX_BACKOFF_SECS);
                }
            }
        }
    }

    fn parse_market_messages(&self, text: &str) -> Result<Vec<MarketWebSocketMessage>> {
        debug!(message = %text, "market websocket message received");

        let value: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| Error::validation(format!("Failed to parse market message: {e}")))?;

        match value {
            serde_json::Value::Array(arr) => {
                let mut out = Vec::with_capacity(arr.len());
                for v in arr {
                    let msg: MarketWebSocketMessage = serde_json::from_value(v).map_err(|e| {
                        Error::validation(format!("Failed to parse market message: {e}"))
                    })?;
                    out.push(msg);
                }
                Ok(out)
            }
            other => {
                let msg: MarketWebSocketMessage = serde_json::from_value(other).map_err(|e| {
                    Error::validation(format!("Failed to parse market message: {e}"))
                })?;
                Ok(vec![msg])
            }
        }
    }
}

/// User WebSocket client for receiving authenticated user events
///
/// Provides access to trade executions, order updates, and other user-specific events.
pub struct UserWebSocket {
    url: String,
    credentials: Credentials,
    writer: Arc<Mutex<Option<WsSink>>>,
    reader: Arc<Mutex<Option<WsSource>>>,
    auto_reconnect: bool,
}

impl UserWebSocket {
    /// Creates a new user WebSocket client with the default URL
    ///
    /// # Arguments
    ///
    /// * `credentials` - API credentials for authentication
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection cannot be established or authentication fails
    pub async fn new(credentials: Credentials) -> Result<Self> {
        Self::with_url(DEFAULT_USER_WS_URL, credentials).await
    }

    /// Creates a new user WebSocket client with a custom URL
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket endpoint URL
    /// * `credentials` - API credentials for authentication
    ///
    /// # Errors
    ///
    /// Returns an error if the WebSocket connection cannot be established or authentication fails
    pub async fn with_url(url: &str, credentials: Credentials) -> Result<Self> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| Error::validation(format!("Failed to connect to user WebSocket: {e}")))?;

        let (mut writer, reader) = ws_stream.split();

        // Send authentication message
        let auth_message = json!({
            "type": "user",
            "auth": {
                "apiKey": credentials.key,
                "secret": credentials.secret.reveal(),
                "passphrase": credentials.passphrase.reveal()
            }
        });

        writer
            .send(Message::Text(auth_message.to_string().into()))
            .await
            .map_err(|e| Error::validation(format!("Failed to authenticate: {e}")))?;

        Ok(Self {
            url: url.to_owned(),
            credentials,
            writer: Arc::new(Mutex::new(Some(writer))),
            reader: Arc::new(Mutex::new(Some(reader))),
            auto_reconnect: true,
        })
    }

    /// Receives the next user event from the WebSocket
    ///
    /// Returns `None` if the connection is closed or an error occurs
    ///
    /// # Errors
    ///
    /// Returns an error if message parsing fails
    pub async fn next(&mut self) -> Option<Result<UserWebSocketMessage>> {
        loop {
            let mut reader = self.reader.lock().await;
            let reader_ref = reader.as_mut()?;

            match reader_ref.next().await {
                Some(Ok(Message::Text(text))) => {
                    drop(reader);

                    match self.parse_user_message(&text) {
                        Ok(msg) => return Some(Ok(msg)),
                        Err(e) => return Some(Err(e)),
                    }
                }
                Some(Ok(Message::Ping(data))) => {
                    drop(reader);
                    let mut writer = self.writer.lock().await;
                    if let Some(ws) = writer.as_mut() {
                        let _ = ws.send(Message::Pong(data)).await;
                    }
                }
                Some(Ok(Message::Close(_))) => {
                    drop(reader);
                    if self.auto_reconnect {
                        let _ = self.reconnect().await;
                        continue;
                    }
                    return None;
                }
                Some(Err(_)) => {
                    drop(reader);
                    if self.auto_reconnect {
                        let _ = self.reconnect().await;
                        continue;
                    }
                    return None;
                }
                None => {
                    drop(reader);
                    if self.auto_reconnect {
                        let _ = self.reconnect().await;
                        continue;
                    }
                    return None;
                }
                _ => {
                    // Ignore other message types
                    drop(reader);
                }
            }
        }
    }

    /// Closes the WebSocket connection
    ///
    /// # Errors
    ///
    /// Returns an error if the close message cannot be sent
    pub async fn close(&mut self) -> Result<()> {
        self.auto_reconnect = false;
        let mut writer = self.writer.lock().await;
        if let Some(ws) = writer.as_mut() {
            ws.close()
                .await
                .map_err(|e| Error::validation(format!("Failed to close WebSocket: {e}")))?;
        }
        Ok(())
    }

    async fn reconnect(&mut self) -> Result<()> {
        let mut backoff = INITIAL_BACKOFF_SECS;

        loop {
            sleep(Duration::from_secs(backoff)).await;

            match connect_async(&self.url).await {
                Ok((ws_stream, _)) => {
                    let (mut writer, reader) = ws_stream.split();

                    // Re-authenticate
                    let auth_message = json!({
                        "type": "user",
                        "auth": {
                            "apiKey": self.credentials.key,
                            "secret": self.credentials.secret.reveal(),
                            "passphrase": self.credentials.passphrase.reveal()
                        }
                    });

                    if writer
                        .send(Message::Text(auth_message.to_string().into()))
                        .await
                        .is_err()
                    {
                        backoff = (backoff * 2).min(MAX_BACKOFF_SECS);
                        continue;
                    }

                    *self.writer.lock().await = Some(writer);
                    *self.reader.lock().await = Some(reader);

                    return Ok(());
                }
                Err(_) => {
                    backoff = (backoff * 2).min(MAX_BACKOFF_SECS);
                }
            }
        }
    }

    fn parse_user_message(&self, text: &str) -> Result<UserWebSocketMessage> {
        serde_json::from_str(text)
            .map_err(|e| Error::validation(format!("Failed to parse user message: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_should_be_defined() {
        assert!(!DEFAULT_MARKET_WS_URL.is_empty());
        assert!(!DEFAULT_USER_WS_URL.is_empty());
        assert_eq!(MAX_BACKOFF_SECS, 30);
        assert_eq!(INITIAL_BACKOFF_SECS, 1);
    }
}
