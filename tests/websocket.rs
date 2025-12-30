#![cfg(feature = "ws")]
#![allow(
    clippy::unwrap_used,
    reason = "Do not need additional syntax for setting up tests"
)]

mod common;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt as _, StreamExt as _};
use polymarket_client_sdk::clob::ws::{Client, Config, WsMessage};
use polymarket_client_sdk::types::Address;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

/// Mock WebSocket server.
struct MockWsServer {
    addr: SocketAddr,
    /// Broadcast messages to ALL connected clients
    message_tx: broadcast::Sender<String>,
    /// Receives subscription requests from clients
    subscription_rx: mpsc::UnboundedReceiver<String>,
}

impl MockWsServer {
    /// Start a mock WebSocket server on a random port.
    async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Broadcast channel for sending to ALL clients
        let (message_tx, _) = broadcast::channel::<String>(100);
        let (subscription_tx, subscription_rx) = mpsc::unbounded_channel::<String>();

        let broadcast_tx = message_tx.clone();

        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };

                let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await else {
                    continue;
                };

                let (mut write, mut read) = ws_stream.split();
                let sub_tx = subscription_tx.clone();
                let mut msg_rx = broadcast_tx.subscribe();

                // Spawn a task to handle this connection
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            // Handle incoming messages from client
                            msg = read.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) if text != "PING" => {
                                        drop(sub_tx.send(text.to_string()));
                                    }
                                    Some(Ok(_)) => {}
                                    _ => break,
                                }
                            }
                            // Handle outgoing messages to client
                            msg = msg_rx.recv() => {
                                match msg {
                                    Ok(text) => {
                                        if write.send(Message::Text(text.into())).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                    }
                });
            }
        });

        Self {
            addr,
            message_tx,
            subscription_rx,
        }
    }

    fn ws_url(&self, path: &str) -> String {
        format!("ws://{}{}", self.addr, path)
    }

    /// Send a message to all connected clients.
    fn send(&self, message: &str) {
        drop(self.message_tx.send(message.to_owned()));
    }

    /// Receive the next subscription request.
    async fn recv_subscription(&mut self) -> Option<String> {
        timeout(Duration::from_secs(2), self.subscription_rx.recv())
            .await
            .ok()
            .flatten()
    }
}

/// Example payloads from CLOB documentation.
/// <https://docs.polymarket.com/developers/CLOB/websocket/market-channel>
/// <https://docs.polymarket.com/developers/CLOB/websocket/user-channel>
mod payloads {
    use serde_json::{Value, json};

    pub const ASSET_ID: &str =
        "65818619657568813474341868652308942079804919287380422192892211131408793125422";

    pub const OTHER_ASSET_ID: &str =
        "99999999999999999999999999999999999999999999999999999999999999999";
    pub const MARKET: &str = "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af";

    pub fn book() -> Value {
        json!({
            "event_type": "book",
            "asset_id": ASSET_ID,
            "market": MARKET,
            "bids": [
                { "price": ".48", "size": "30" },
                { "price": ".49", "size": "20" },
                { "price": ".50", "size": "15" }
            ],
            "asks": [
                { "price": ".52", "size": "25" },
                { "price": ".53", "size": "60" },
                { "price": ".54", "size": "10" }
            ],
            "timestamp": "123456789000",
            "hash": "0x1234567890abcdef"
        })
    }

    pub fn price_change_batch(asset_id: &str) -> Value {
        json!({
            "market": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
            "price_changes": [
                {
                    "asset_id": asset_id,
                    "price": "0.5",
                    "size": "200",
                    "side": "BUY",
                    "hash": "56621a121a47ed9333273e21c83b660cff37ae50",
                    "best_bid": "0.5",
                    "best_ask": "1"
                }
            ],
            "timestamp": "1757908892351",
            "event_type": "price_change"
        })
    }

    pub fn tick_size_change() -> Value {
        json!({
            "event_type": "tick_size_change",
            "asset_id": ASSET_ID,
            "market": MARKET,
            "old_tick_size": "0.01",
            "new_tick_size": "0.001",
            "timestamp": "100000000"
        })
    }

    pub fn last_trade_price(asset_id: &str) -> Value {
        json!({
            "asset_id": asset_id,
            "event_type": "last_trade_price",
            "fee_rate_bps": "0",
            "market": "0x6a67b9d828d53862160e470329ffea5246f338ecfffdf2cab45211ec578b0347",
            "price": "0.456",
            "side": "BUY",
            "size": "219.217767",
            "timestamp": "1750428146322"
        })
    }

    pub fn trade() -> Value {
        json!({
            "asset_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426",
            "event_type": "trade",
            "id": "28c4d2eb-bbea-40e7-a9f0-b2fdb56b2c2e",
            "last_update": "1672290701",
            "maker_orders": [
                {
                    "asset_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426",
                    "matched_amount": "10",
                    "order_id": "0xff354cd7ca7539dfa9c28d90943ab5779a4eac34b9b37a757d7b32bdfb11790b",
                    "outcome": "YES",
                    "owner": "9180014b-33c8-9240-a14b-bdca11c0a465",
                    "price": "0.57"
                }
            ],
            "market": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
            "matchtime": "1672290701",
            "outcome": "YES",
            "owner": "9180014b-33c8-9240-a14b-bdca11c0a465",
            "price": "0.57",
            "side": "BUY",
            "size": "10",
            "status": "MATCHED",
            "taker_order_id": "0x06bc63e346ed4ceddce9efd6b3af37c8f8f440c92fe7da6b2d0f9e4ccbc50c42",
            "timestamp": "1672290701",
            "trade_owner": "9180014b-33c8-9240-a14b-bdca11c0a465",
            "type": "TRADE"
        })
    }

    pub fn order() -> Value {
        json!({
            "asset_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426",
            "associate_trades": null,
            "event_type": "order",
            "id": "0xff354cd7ca7539dfa9c28d90943ab5779a4eac34b9b37a757d7b32bdfb11790b",
            "market": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
            "order_owner": "9180014b-33c8-9240-a14b-bdca11c0a465",
            "original_size": "10",
            "outcome": "YES",
            "owner": "9180014b-33c8-9240-a14b-bdca11c0a465",
            "price": "0.57",
            "side": "SELL",
            "size_matched": "0",
            "timestamp": "1672290687",
            "type": "PLACEMENT"
        })
    }
}

mod market_channel {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::payloads::OTHER_ASSET_ID;

    #[tokio::test]
    async fn subscribe_orderbook_receives_book_updates() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![payloads::ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Verify subscription request was sent
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(sub_request.contains("\"type\":\"market\""));
        assert!(sub_request.contains(payloads::ASSET_ID));

        // Send book update from docs
        server.send(&payloads::book().to_string());

        // Receive and verify
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let book = result.unwrap().unwrap().unwrap();

        assert_eq!(book.asset_id, payloads::ASSET_ID);
        assert_eq!(book.market, payloads::MARKET);
        assert_eq!(book.bids.len(), 3);
        assert_eq!(book.asks.len(), 3);
        assert_eq!(book.bids[0].price, dec!(0.48));
        assert_eq!(book.bids[0].size, dec!(30));
        assert_eq!(book.asks[0].price, dec!(0.52));
        assert_eq!(book.hash, Some("0x1234567890abcdef".to_owned()));
    }

    #[tokio::test]
    async fn subscribe_prices_receives_price_changes() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let asset_id =
            "71321045679252212594626385532706912750332728571942532289631379312455583992563";
        let stream = client.subscribe_prices(vec![asset_id.to_owned()]).unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        server.send(&payloads::price_change_batch(asset_id).to_string());

        // Receive and verify
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let price = result.unwrap().unwrap().unwrap();

        assert_eq!(price.price_changes[0].asset_id, asset_id);
        assert_eq!(price.price_changes[0].price, dec!(0.5));
        assert_eq!(price.price_changes[0].size, Some(dec!(200)));
        assert_eq!(price.price_changes[0].best_bid, Some(dec!(0.5)));
        assert_eq!(price.price_changes[0].best_ask, Some(dec!(1)));
    }

    #[tokio::test]
    async fn filters_messages_by_asset_id() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let subscribed_asset = payloads::ASSET_ID;

        let stream = client
            .subscribe_orderbook(vec![subscribed_asset.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send message for non-subscribed asset (should be filtered)
        let mut other_book = payloads::book();
        other_book["asset_id"] = serde_json::Value::String(OTHER_ASSET_ID.to_owned());
        server.send(&other_book.to_string());

        // Send message for subscribed asset
        server.send(&payloads::book().to_string());

        // Should receive only the subscribed asset's message
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let book = result.unwrap().unwrap().unwrap();
        assert_eq!(book.asset_id, subscribed_asset);
    }

    #[tokio::test]
    async fn subscribe_midpoints_calculates_midpoint() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let stream = client
            .subscribe_midpoints(vec![payloads::ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send book with bids at 0.48, 0.49, 0.50 and asks at 0.52, 0.53, 0.54
        // Best bid = 0.48, best ask = 0.52 (from payloads::book())
        // Midpoint = (0.48 + 0.52) / 2 = 0.50
        server.send(&payloads::book().to_string());

        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let midpoint = result.unwrap().unwrap().unwrap();

        assert_eq!(midpoint.asset_id, payloads::ASSET_ID);
        assert_eq!(midpoint.market, payloads::MARKET);
        assert_eq!(midpoint.midpoint, dec!(0.50));
    }

    #[tokio::test]
    async fn subscribe_midpoints_skips_empty_orderbook() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let stream = client
            .subscribe_midpoints(vec![payloads::ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send book with no bids (should be skipped)
        let empty_book = json!({
            "event_type": "book",
            "asset_id": payloads::ASSET_ID,
            "market": payloads::MARKET,
            "bids": [],
            "asks": [{ "price": ".52", "size": "25" }],
            "timestamp": "123456789000"
        });
        server.send(&empty_book.to_string());

        // Send valid book
        server.send(&payloads::book().to_string());

        // Should only receive the valid midpoint (empty book skipped)
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let midpoint = result.unwrap().unwrap().unwrap();
        assert_eq!(midpoint.midpoint, dec!(0.50));
    }
}

mod user_channel {
    use polymarket_client_sdk::auth::Credentials;
    use polymarket_client_sdk::clob::types::Side;
    use rust_decimal_macros::dec;
    use tokio::time::sleep;

    use super::*;
    use crate::{
        common::{API_KEY, PASSPHRASE, SECRET},
        payloads::OTHER_ASSET_ID,
    };

    fn test_credentials() -> Credentials {
        Credentials::new(API_KEY, SECRET.to_owned(), PASSPHRASE.to_owned())
    }

    #[tokio::test]
    async fn subscribe_user_events_receives_orders() {
        let mut server = MockWsServer::start().await;
        let base_endpoint = format!("ws://{}", server.addr);

        let config = Config::default();
        let client = Client::new(&base_endpoint, config)
            .unwrap()
            .authenticate(test_credentials(), Address::ZERO)
            .unwrap();

        // Wait for connections to establish
        sleep(Duration::from_millis(100)).await;

        let stream = client.subscribe_user_events(vec![]).unwrap();
        let mut stream = Box::pin(stream);

        // Verify subscription request contains auth
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(sub_request.contains("\"type\":\"user\""));
        assert!(sub_request.contains("\"auth\""));
        assert!(sub_request.contains("\"apiKey\""));

        // Send order message from docs
        server.send(&payloads::order().to_string());

        // Receive and verify
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        match result.unwrap().unwrap().unwrap() {
            WsMessage::Order(order) => {
                assert_eq!(
                    order.id,
                    "0xff354cd7ca7539dfa9c28d90943ab5779a4eac34b9b37a757d7b32bdfb11790b"
                );
                assert_eq!(
                    order.market,
                    "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af"
                );
                assert_eq!(order.price, dec!(0.57));
                assert_eq!(order.side, Side::Sell);
                assert_eq!(order.original_size, Some(dec!(10)));
                assert_eq!(order.size_matched, Some(dec!(0)));
                assert_eq!(order.outcome, Some("YES".to_owned()));
                assert_eq!(order.msg_type, Some("PLACEMENT".to_owned()));
            }
            other => panic!("Expected Order, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn subscribe_user_events_receives_trades() {
        let mut server = MockWsServer::start().await;
        let base_endpoint = format!("ws://{}", server.addr);

        let config = Config::default();
        let client = Client::new(&base_endpoint, config)
            .unwrap()
            .authenticate(test_credentials(), Address::ZERO)
            .unwrap();

        // Wait for connections to establish
        sleep(Duration::from_millis(100)).await;

        let stream = client.subscribe_user_events(vec![]).unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send trade message from docs
        server.send(&payloads::trade().to_string());

        // Receive and verify
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        match result.unwrap().unwrap().unwrap() {
            WsMessage::Trade(trade) => {
                assert_eq!(trade.id, "28c4d2eb-bbea-40e7-a9f0-b2fdb56b2c2e");
                assert_eq!(
                    trade.market,
                    "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af"
                );
                assert_eq!(trade.price, dec!(0.57));
                assert_eq!(trade.size, dec!(10));
                assert_eq!(trade.side, Side::Buy);
                assert_eq!(trade.status, "MATCHED");
                assert_eq!(trade.outcome, Some("YES".to_owned()));
                assert_eq!(trade.maker_orders.len(), 1);
                assert_eq!(trade.maker_orders[0].matched_amount, dec!(10));
                assert_eq!(trade.maker_orders[0].price, dec!(0.57));
                assert_eq!(
                    trade.taker_order_id,
                    Some(
                        "0x06bc63e346ed4ceddce9efd6b3af37c8f8f440c92fe7da6b2d0f9e4ccbc50c42"
                            .to_owned()
                    )
                );
            }
            other => panic!("Expected Trade, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn subscribe_orders_filters_to_orders_only() {
        let mut server = MockWsServer::start().await;
        let base_endpoint = format!("ws://{}", server.addr);

        let config = Config::default();
        let client = Client::new(&base_endpoint, config)
            .unwrap()
            .authenticate(test_credentials(), Address::ZERO)
            .unwrap();

        // Wait for connections to establish
        sleep(Duration::from_millis(100)).await;

        let stream = client.subscribe_orders(vec![]).unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send a trade (should be filtered)
        server.send(&payloads::trade().to_string());

        // Send an order
        server.send(&payloads::order().to_string());

        // Should only receive the order
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let order = result.unwrap().unwrap().unwrap();
        assert_eq!(
            order.id,
            "0xff354cd7ca7539dfa9c28d90943ab5779a4eac34b9b37a757d7b32bdfb11790b"
        );
    }

    #[tokio::test]
    async fn subscribe_trades_filters_to_trades_only() {
        let mut server = MockWsServer::start().await;
        let base_endpoint = format!("ws://{}", server.addr);

        let config = Config::default();
        let client = Client::new(&base_endpoint, config)
            .unwrap()
            .authenticate(test_credentials(), Address::ZERO)
            .unwrap();

        // Wait for connections to establish
        sleep(Duration::from_millis(100)).await;

        let stream = client.subscribe_trades(vec![]).unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send an order (should be filtered)
        server.send(&payloads::order().to_string());

        // Send a trade
        server.send(&payloads::trade().to_string());

        // Should only receive the trade
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let trade = result.unwrap().unwrap().unwrap();
        assert_eq!(trade.id, "28c4d2eb-bbea-40e7-a9f0-b2fdb56b2c2e");
    }

    #[tokio::test]
    async fn multiplexing_does_not_send_duplicate_subscription() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::ASSET_ID;

        // First subscription - should send request
        let _stream1 = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        let sub1 = server.recv_subscription().await.unwrap();
        assert!(sub1.contains(asset_id));

        // Second subscription to SAME asset - should NOT send request (multiplexed)
        let _stream2 = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();

        // Third subscription to DIFFERENT asset - should send request
        let _stream3 = client
            .subscribe_orderbook(vec![OTHER_ASSET_ID.to_owned()])
            .unwrap();

        // The next message we receive should be for other_asset only
        let sub2 = server.recv_subscription().await.unwrap();
        assert!(
            sub2.contains(OTHER_ASSET_ID),
            "Should receive subscription for new asset"
        );
        assert!(
            !sub2.contains(asset_id),
            "Should NOT contain duplicate of already-subscribed asset"
        );
    }

    #[tokio::test]
    async fn deauthenticate_returns_to_unauthenticated_state() {
        let mut server = MockWsServer::start().await;
        let base_endpoint = format!("ws://{}", server.addr);

        let config = Config::default();
        let client = Client::new(&base_endpoint, config)
            .unwrap()
            .authenticate(test_credentials(), Address::ZERO)
            .unwrap();

        // Wait for connection to establish
        sleep(Duration::from_millis(100)).await;

        // Deauthenticate should succeed and return unauthenticated client
        let unauth_client = client.deauthenticate().unwrap();

        // Should still be able to subscribe to market data
        let stream = unauth_client
            .subscribe_orderbook(vec![payloads::ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        server.send(&payloads::book().to_string());

        let result = timeout(Duration::from_secs(2), stream.next()).await;
        result.unwrap().unwrap().unwrap();
    }
}

mod reconnection {
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    /// Mock WebSocket server that can simulate disconnections and send messages.
    struct ReconnectableMockServer {
        addr: SocketAddr,
        subscription_rx: mpsc::UnboundedReceiver<String>,
        message_tx: broadcast::Sender<String>,
        disconnect_signal: Arc<AtomicBool>,
    }

    impl ReconnectableMockServer {
        async fn start() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let (message_tx, _) = broadcast::channel::<String>(100);
            let (subscription_tx, subscription_rx) = mpsc::unbounded_channel::<String>();
            let disconnect_signal = Arc::new(AtomicBool::new(false));

            let broadcast_tx = message_tx.clone();
            let disconnect = Arc::clone(&disconnect_signal);

            tokio::spawn(async move {
                loop {
                    let Ok((stream, _)) = listener.accept().await else {
                        break;
                    };

                    let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await else {
                        continue;
                    };

                    let (mut write, mut read) = ws_stream.split();
                    let sub_tx = subscription_tx.clone();
                    let mut msg_rx = broadcast_tx.subscribe();
                    let disconnect_clone = Arc::clone(&disconnect);

                    tokio::spawn(async move {
                        loop {
                            if disconnect_clone.load(Ordering::SeqCst) {
                                break;
                            }

                            tokio::select! {
                                msg = read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) if text != "PING" => {
                                            drop(sub_tx.send(text.to_string()));
                                        }
                                        Some(Ok(_)) => {}
                                        _ => break,
                                    }
                                }
                                msg = msg_rx.recv() => {
                                    match msg {
                                        Ok(text) => {
                                            if write.send(Message::Text(text.into())).await.is_err() {
                                                break;
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                                () = tokio::time::sleep(Duration::from_millis(50)) => {
                                    if disconnect_clone.load(Ordering::SeqCst) {
                                        break;
                                    }
                                }
                            }
                        }
                    });
                }
            });

            Self {
                addr,
                subscription_rx,
                message_tx,
                disconnect_signal,
            }
        }

        fn ws_url(&self, path: &str) -> String {
            format!("ws://{}{}", self.addr, path)
        }

        fn disconnect_all(&self) {
            self.disconnect_signal.store(true, Ordering::SeqCst);
        }

        fn allow_reconnect(&self) {
            self.disconnect_signal.store(false, Ordering::SeqCst);
        }

        fn send(&self, message: &str) {
            drop(self.message_tx.send(message.to_owned()));
        }

        async fn recv_subscription(&mut self) -> Option<String> {
            timeout(Duration::from_secs(2), self.subscription_rx.recv())
                .await
                .ok()
                .flatten()
        }
    }

    fn config() -> Config {
        let mut config = Config::default();
        config.reconnect.max_attempts = Some(5);
        config.reconnect.initial_backoff = Duration::from_millis(50);
        config.reconnect.max_backoff = Duration::from_millis(200);
        config
    }

    #[tokio::test]
    async fn resubscribes_and_receives_messages_after_reconnect() {
        let mut server = ReconnectableMockServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, config()).unwrap();

        let asset_id = payloads::ASSET_ID;
        let stream = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Verify initial subscription
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(sub_request.contains(asset_id));

        // Verify we can receive messages before disconnect
        server.send(&payloads::book().to_string());
        let msg1 = timeout(Duration::from_secs(2), stream.next()).await;
        assert!(msg1.is_ok(), "Should receive message before disconnect");

        // Simulate disconnect
        server.disconnect_all();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Allow reconnection and wait for re-subscription
        server.allow_reconnect();

        // Wait for re-subscription request (proves reconnection happened)
        let resub = server.recv_subscription().await;
        assert!(
            resub.is_some(),
            "Should receive re-subscription after reconnect"
        );
        assert!(resub.unwrap().contains(asset_id));

        // Send message after reconnection and verify it's received
        server.send(&payloads::book().to_string());
        let msg2 = timeout(Duration::from_secs(2), stream.next()).await;
        assert!(
            msg2.is_ok(),
            "Should receive message after reconnection - proves subscription is active"
        );
    }

    #[tokio::test]
    async fn resubscribes_all_assets_after_reconnect() {
        let mut server = ReconnectableMockServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, config()).unwrap();

        let asset1 = payloads::ASSET_ID;
        let asset2 = payloads::OTHER_ASSET_ID;

        // Subscribe to both assets
        let _stream1 = client.subscribe_orderbook(vec![asset1.to_owned()]).unwrap();
        let _: Option<String> = server.recv_subscription().await;

        let _stream2 = client.subscribe_orderbook(vec![asset2.to_owned()]).unwrap();
        let sub2 = server.recv_subscription().await.unwrap();
        assert!(sub2.contains(asset2));

        // Disconnect and reconnect
        server.disconnect_all();
        tokio::time::sleep(Duration::from_millis(100)).await;
        server.allow_reconnect();

        // Verify re-subscription contains BOTH assets
        let resub = server.recv_subscription().await;
        assert!(resub.is_some(), "Should receive re-subscription");
        let resub_str = resub.unwrap();
        assert!(
            resub_str.contains(asset1) && resub_str.contains(asset2),
            "Re-subscription should contain all tracked assets, got: {resub_str}"
        );
    }
}

mod unsubscribe {
    use super::*;
    use crate::payloads::OTHER_ASSET_ID;

    #[tokio::test]
    async fn unsubscribe_sends_request_when_refcount_reaches_zero() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::ASSET_ID;

        // Subscribe once
        let _stream = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        let sub = server.recv_subscription().await.unwrap();
        assert!(sub.contains(asset_id));

        // Unsubscribe - should send unsubscribe request since refcount goes to 0
        client
            .unsubscribe_orderbook(&[asset_id.to_owned()])
            .unwrap();

        let unsub = server.recv_subscription().await.unwrap();
        assert!(
            unsub.contains("\"operation\":\"unsubscribe\""),
            "Should send unsubscribe request, got: {unsub}"
        );
        assert!(unsub.contains(asset_id));
    }

    #[tokio::test]
    async fn unsubscribe_does_not_send_request_when_refcount_above_zero() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::ASSET_ID;

        // Subscribe twice to same asset
        let _stream1 = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        let _: Option<String> = server.recv_subscription().await;

        let _stream2 = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        // Second subscribe should not send (multiplexed)

        // Unsubscribe once - refcount goes from 2 to 1, should NOT send request
        client
            .unsubscribe_orderbook(&[asset_id.to_owned()])
            .unwrap();

        // Subscribe to different asset to verify server is still responsive
        let _stream3 = client
            .subscribe_orderbook(vec![OTHER_ASSET_ID.to_owned()])
            .unwrap();

        let next_msg = server.recv_subscription().await.unwrap();
        // Should be a subscribe for OTHER_ASSET_ID, not an unsubscribe for ASSET_ID
        assert!(
            next_msg.contains(OTHER_ASSET_ID),
            "Should receive subscribe for new asset, not unsubscribe. Got: {next_msg}"
        );
        assert!(
            !next_msg.contains("\"operation\":\"unsubscribe\""),
            "Should not have sent unsubscribe yet"
        );
    }

    #[tokio::test]
    async fn multiple_streams_unsubscribe_independently() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::ASSET_ID;

        // Subscribe three times
        let _stream1 = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        let _: Option<String> = server.recv_subscription().await;

        let _stream2 = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        let _stream3 = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();

        // Unsubscribe twice - still one stream left
        client
            .unsubscribe_orderbook(&[asset_id.to_owned()])
            .unwrap();
        client
            .unsubscribe_orderbook(&[asset_id.to_owned()])
            .unwrap();

        // Third unsubscribe - now refcount hits 0, should send request
        client
            .unsubscribe_orderbook(&[asset_id.to_owned()])
            .unwrap();

        let unsub = server.recv_subscription().await.unwrap();
        assert!(
            unsub.contains("\"operation\":\"unsubscribe\""),
            "Should send unsubscribe when last stream unsubscribes, got: {unsub}"
        );
    }

    #[tokio::test]
    async fn resubscribe_after_full_unsubscribe() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::ASSET_ID;

        // Subscribe
        let _stream1 = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        let sub1 = server.recv_subscription().await.unwrap();
        assert!(sub1.contains(asset_id));

        // Fully unsubscribe
        client
            .unsubscribe_orderbook(&[asset_id.to_owned()])
            .unwrap();
        let unsub = server.recv_subscription().await.unwrap();
        assert!(unsub.contains("\"operation\":\"unsubscribe\""));

        // Re-subscribe should send a new subscription request
        let stream2 = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        let mut stream2 = Box::pin(stream2);

        let sub2 = server.recv_subscription().await.unwrap();
        assert!(
            sub2.contains("\"type\":\"market\""),
            "Should send new subscribe request after full unsubscribe"
        );
        assert!(sub2.contains(asset_id));

        // Verify stream works
        server.send(&payloads::book().to_string());
        let result = timeout(Duration::from_secs(2), stream2.next()).await;
        assert!(
            result.is_ok(),
            "Should receive messages on re-subscribed stream"
        );
    }

    #[tokio::test]
    async fn unsubscribe_empty_asset_ids_returns_error() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        // Subscribe to something first
        let _stream = client
            .subscribe_orderbook(vec![payloads::ASSET_ID.to_owned()])
            .unwrap();
        let _: Option<String> = server.recv_subscription().await;

        // Unsubscribe with empty array should error
        let result = client.unsubscribe_orderbook(&[]);
        assert!(result.is_err(), "Should return error for empty asset_ids");
    }

    #[tokio::test]
    async fn unsubscribe_nonexistent_subscription_is_noop() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::ASSET_ID;
        let nonexistent_asset = OTHER_ASSET_ID;

        // Subscribe to one asset
        let _stream = client
            .subscribe_orderbook(vec![asset_id.to_owned()])
            .unwrap();
        let _: Option<String> = server.recv_subscription().await;

        // Unsubscribe from asset we never subscribed to - should be no-op
        client
            .unsubscribe_orderbook(&[nonexistent_asset.to_owned()])
            .unwrap();

        // Subscribe to another asset to verify server didn't receive unsubscribe
        let _stream2 = client
            .subscribe_orderbook(vec![nonexistent_asset.to_owned()])
            .unwrap();

        let next_msg = server.recv_subscription().await.unwrap();
        // Should be a subscribe, not an unsubscribe
        assert!(
            next_msg.contains("\"type\":\"market\""),
            "Should receive subscribe, not unsubscribe for non-existent sub. Got: {next_msg}"
        );
    }
}

mod message_parsing {
    use polymarket_client_sdk::clob::types::Side;
    use polymarket_client_sdk::clob::ws::{LastTradePrice, TickSizeChange};
    use rust_decimal_macros::dec;

    use super::*;

    #[tokio::test]
    async fn parses_book_with_hash() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![payloads::ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        server.send(&payloads::book().to_string());

        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let book = result.unwrap().unwrap().unwrap();

        // Verify all fields from docs example
        assert_eq!(book.timestamp, 123_456_789_000);
        assert_eq!(book.hash, Some("0x1234567890abcdef".to_owned()));
        assert_eq!(book.bids[1].price, dec!(0.49));
        assert_eq!(book.bids[1].size, dec!(20));
        assert_eq!(book.asks[2].price, dec!(0.54));
        assert_eq!(book.asks[2].size, dec!(10));
    }

    #[tokio::test]
    async fn parses_batch_price_changes() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let asset_a =
            "71321045679252212594626385532706912750332728571942532289631379312455583992563";
        let asset_b =
            "88888888888888888888888888888888888888888888888888888888888888888888888888888";

        let stream = client
            .subscribe_prices(vec![asset_a.to_owned(), asset_b.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send batch price change with two assets
        let batch_msg = json!({
            "market": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
            "price_changes": [
                {
                    "asset_id": asset_a,
                    "price": "0.5",
                    "size": "200",
                    "side": "BUY",
                    "hash": "56621a121a47ed9333273e21c83b660cff37ae50",
                    "best_bid": "0.5",
                    "best_ask": "1"
                },
                {
                    "asset_id": asset_b,
                    "price": "0.75",
                    "side": "SELL"
                }
            ],
            "timestamp": "1757908892351",
            "event_type": "price_change"
        });
        server.send(&batch_msg.to_string());

        // Should receive two price changes
        let result1 = timeout(Duration::from_secs(2), stream.next()).await;
        let prices = result1.unwrap().unwrap().unwrap();
        assert_eq!(prices.price_changes[0].asset_id, asset_a);
        assert_eq!(prices.price_changes[0].price, dec!(0.5));
        assert_eq!(prices.price_changes[0].size, Some(dec!(200)));
        assert_eq!(
            prices.price_changes[0].hash,
            Some("56621a121a47ed9333273e21c83b660cff37ae50".to_owned())
        );

        assert_eq!(prices.price_changes[1].asset_id, asset_b);
        assert_eq!(prices.price_changes[1].price, dec!(0.75));
        assert!(prices.price_changes[1].size.is_none());
    }

    #[test]
    fn parses_tick_size_change() {
        let payload = payloads::tick_size_change().to_string();
        let tsc: TickSizeChange = serde_json::from_str(&payload).unwrap();

        assert_eq!(tsc.asset_id, payloads::ASSET_ID);
        assert_eq!(tsc.market, payloads::MARKET);
        assert_eq!(tsc.old_tick_size, dec!(0.01));
        assert_eq!(tsc.new_tick_size, dec!(0.001));
        assert_eq!(tsc.timestamp, 100_000_000);
    }

    #[test]
    fn parses_last_trade_price() {
        let asset_id =
            "114122071509644379678018727908709560226618148003371446110114509806601493071694";
        let payload = payloads::last_trade_price(asset_id).to_string();
        let ltp: LastTradePrice = serde_json::from_str(&payload).unwrap();

        assert_eq!(ltp.asset_id, asset_id);
        assert_eq!(
            ltp.market,
            "0x6a67b9d828d53862160e470329ffea5246f338ecfffdf2cab45211ec578b0347"
        );
        assert_eq!(ltp.price, dec!(0.456));
        assert_eq!(ltp.side, Some(Side::Buy));
        assert_eq!(ltp.timestamp, 1_750_428_146_322);
    }
}
