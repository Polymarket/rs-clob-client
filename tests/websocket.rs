#![cfg(all(feature = "clob", feature = "ws"))]
#![allow(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    reason = "Do not need additional syntax for setting up tests"
)]

mod common;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt as _, StreamExt as _};
use polymarket_client_sdk::clob::ws::{Client, WsMessage};
use polymarket_client_sdk::types::{Address, U256, b256};
use polymarket_client_sdk::ws::config::Config;
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
pub mod payloads {
    use std::str::FromStr as _;

    use polymarket_client_sdk::types::{B256, U256, b256};
    use serde_json::{Value, json};

    pub const ASSET_ID_STR: &str =
        "65818619657568813474341868652308942079804919287380422192892211131408793125422";

    pub const OTHER_ASSET_ID_STR: &str =
        "99999999999999999999999999999999999999999999999999999999999999999";
    pub const MARKET_STR: &str =
        "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af";
    pub const MARKET: B256 =
        b256!("bd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af");

    #[must_use]
    pub fn asset_id() -> U256 {
        U256::from_str(ASSET_ID_STR).unwrap()
    }

    #[must_use]
    pub fn other_asset_id() -> U256 {
        U256::from_str(OTHER_ASSET_ID_STR).unwrap()
    }

    #[must_use]
    pub fn book() -> Value {
        json!({
            "event_type": "book",
            "asset_id": ASSET_ID_STR,
            "market": MARKET_STR,
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

    #[must_use]
    pub fn price_change_batch(asset_id: U256) -> Value {
        json!({
            "market": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
            "price_changes": [
                {
                    "asset_id": asset_id.to_string(),
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

    #[must_use]
    pub fn tick_size_change() -> Value {
        json!({
            "event_type": "tick_size_change",
            "asset_id": ASSET_ID_STR,
            "market": MARKET_STR,
            "old_tick_size": "0.01",
            "new_tick_size": "0.001",
            "timestamp": "100000000"
        })
    }

    #[must_use]
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

    #[must_use]
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
            "market": MARKET_STR,
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

    #[must_use]
    pub fn order() -> Value {
        json!({
            "asset_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426",
            "associate_trades": null,
            "event_type": "order",
            "id": "0xff354cd7ca7539dfa9c28d90943ab5779a4eac34b9b37a757d7b32bdfb11790b",
            "market": MARKET_STR,
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
    use std::str::FromStr as _;

    use rust_decimal_macros::dec;

    use super::*;
    use crate::payloads::OTHER_ASSET_ID_STR;

    #[tokio::test]
    async fn subscribe_orderbook_receives_book_updates() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Verify subscription request was sent
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(sub_request.contains("\"type\":\"market\""));
        assert!(sub_request.contains(&payloads::asset_id().to_string()));

        // Send book update from docs
        server.send(&payloads::book().to_string());

        // Receive and verify
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let book = result.unwrap().unwrap().unwrap();

        assert_eq!(book.asset_id, payloads::asset_id());
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

        let asset_id = U256::from_str(
            "71321045679252212594626385532706912750332728571942532289631379312455583992563",
        )
        .unwrap();
        let stream = client.subscribe_prices(vec![asset_id]).unwrap();
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
    async fn subscribe_tick_size_change_receives_updates() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let stream = client
            .subscribe_tick_size_change(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Verify subscription request was sent
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(sub_request.contains("\"type\":\"market\""));
        assert!(sub_request.contains(&payloads::asset_id().to_string()));

        // Send tick size change event
        server.send(&payloads::tick_size_change().to_string());

        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let tsc = result.unwrap().unwrap().unwrap();

        assert_eq!(tsc.asset_id, payloads::asset_id());
        assert_eq!(tsc.market, payloads::MARKET);
        assert_eq!(tsc.old_tick_size, dec!(0.01));
        assert_eq!(tsc.new_tick_size, dec!(0.001));
        assert_eq!(tsc.timestamp, 100_000_000);
    }

    #[tokio::test]
    async fn filters_messages_by_asset_id() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let config = Config::default();
        let client = Client::new(&endpoint, config).unwrap();

        let subscribed_asset = payloads::asset_id();

        let stream = client.subscribe_orderbook(vec![subscribed_asset]).unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send message for non-subscribed asset (should be filtered)
        let mut other_book = payloads::book();
        other_book["asset_id"] = serde_json::Value::String(OTHER_ASSET_ID_STR.to_owned());
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
            .subscribe_midpoints(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send book with bids at 0.48, 0.49, 0.50 and asks at 0.52, 0.53, 0.54
        // Best bid = 0.48, best ask = 0.52 (from payloads::book())
        // Midpoint = (0.48 + 0.52) / 2 = 0.50
        server.send(&payloads::book().to_string());

        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let midpoint = result.unwrap().unwrap().unwrap();

        assert_eq!(midpoint.asset_id, payloads::asset_id());
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
            .subscribe_midpoints(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send book with no bids (should be skipped)
        let empty_book = json!({
            "event_type": "book",
            "asset_id": payloads::asset_id(),
            "market": payloads::MARKET_STR,
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
    use polymarket_client_sdk::clob::ws::types::response::{OrderMessageType, TradeMessageStatus};
    use rust_decimal_macros::dec;
    use tokio::time::sleep;

    use super::*;
    use crate::common::{API_KEY, PASSPHRASE, SECRET};
    use crate::payloads::OTHER_ASSET_ID_STR;

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
                assert_eq!(order.market, payloads::MARKET);
                assert_eq!(order.price, dec!(0.57));
                assert_eq!(order.side, Side::Sell);
                assert_eq!(order.original_size, Some(dec!(10)));
                assert_eq!(order.size_matched, Some(dec!(0)));
                assert_eq!(order.outcome, Some("YES".to_owned()));
                assert_eq!(order.msg_type, Some(OrderMessageType::Placement));
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
                assert_eq!(trade.market, payloads::MARKET);
                assert_eq!(trade.price, dec!(0.57));
                assert_eq!(trade.size, dec!(10));
                assert_eq!(trade.side, Side::Buy);
                assert_eq!(trade.status, TradeMessageStatus::Matched);
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

        let asset_id = payloads::asset_id();

        // First subscription - should send request
        let _stream1 = client.subscribe_orderbook(vec![asset_id]).unwrap();
        let sub1 = server.recv_subscription().await.unwrap();
        assert!(sub1.contains(&asset_id.to_string()));

        // Second subscription to SAME asset - should NOT send request (multiplexed)
        let _stream2 = client.subscribe_orderbook(vec![asset_id]).unwrap();

        // Third subscription to DIFFERENT asset - should send request
        let _stream3 = client
            .subscribe_orderbook(vec![payloads::other_asset_id()])
            .unwrap();

        // The next message we receive should be for other_asset only
        let sub2 = server.recv_subscription().await.unwrap();
        assert!(
            sub2.contains(OTHER_ASSET_ID_STR),
            "Should receive subscription for new asset"
        );
        assert!(
            !sub2.contains(&asset_id.to_string()),
            "Should NOT contain duplicate of already-subscribed asset"
        );
    }

    #[tokio::test]
    async fn unsubscribe_user_events_sends_request() {
        let mut server = MockWsServer::start().await;
        let base_endpoint = format!("ws://{}", server.addr);

        let config = Config::default();
        let client = Client::new(&base_endpoint, config)
            .unwrap()
            .authenticate(test_credentials(), Address::ZERO)
            .unwrap();

        // Wait for connections to establish
        sleep(Duration::from_millis(100)).await;

        let market = payloads::MARKET;

        // Subscribe to user events for a specific market
        let _stream = client.subscribe_user_events(vec![market]).unwrap();
        let _: Option<String> = server.recv_subscription().await;

        // Unsubscribe from user events
        client.unsubscribe_user_events(&[market]).unwrap();

        let unsub = server.recv_subscription().await.unwrap();
        assert!(
            unsub.contains("\"operation\":\"unsubscribe\""),
            "Should send unsubscribe request, got: {unsub}"
        );
        assert!(unsub.contains(&market.to_string()));
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
            .subscribe_orderbook(vec![payloads::asset_id()])
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

        let asset_id = payloads::asset_id();
        let stream = client.subscribe_orderbook(vec![asset_id]).unwrap();
        let mut stream = Box::pin(stream);

        // Verify initial subscription
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(sub_request.contains(&asset_id.to_string()));

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
        assert!(resub.unwrap().contains(&asset_id.to_string()));

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

        let asset1 = payloads::asset_id();
        let asset2 = payloads::other_asset_id();

        // Subscribe to both assets
        let _stream1 = client.subscribe_orderbook(vec![asset1]).unwrap();
        let _: Option<String> = server.recv_subscription().await;

        let _stream2 = client.subscribe_orderbook(vec![asset2]).unwrap();
        let sub2 = server.recv_subscription().await.unwrap();
        assert!(sub2.contains(&asset2.to_string()));

        // Disconnect and reconnect
        server.disconnect_all();
        tokio::time::sleep(Duration::from_millis(100)).await;
        server.allow_reconnect();

        // Verify re-subscription contains BOTH assets
        let resub = server.recv_subscription().await;
        assert!(resub.is_some(), "Should receive re-subscription");
        let resub_str = resub.unwrap();
        assert!(
            resub_str.contains(&asset1.to_string()) && resub_str.contains(&asset2.to_string()),
            "Re-subscription should contain all tracked assets, got: {resub_str}"
        );
    }

    #[tokio::test]
    async fn preserves_custom_features_after_reconnect() {
        let mut server = ReconnectableMockServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, config()).unwrap();

        let asset_id = payloads::asset_id();

        // Subscribe with custom features enabled (e.g., best_bid_ask)
        let _stream = client.subscribe_best_bid_ask(vec![asset_id]).unwrap();

        // Verify initial subscription has custom_feature_enabled
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(
            sub_request.contains("\"custom_feature_enabled\":true"),
            "Initial subscription should have custom_feature_enabled, got: {sub_request}"
        );

        // Disconnect and reconnect
        server.disconnect_all();
        tokio::time::sleep(Duration::from_millis(100)).await;
        server.allow_reconnect();

        // Verify re-subscription ALSO has custom_feature_enabled
        let resub = server.recv_subscription().await;
        assert!(resub.is_some(), "Should receive re-subscription");
        let resub_str = resub.unwrap();
        assert!(
            resub_str.contains("\"custom_feature_enabled\":true"),
            "Re-subscription should preserve custom_feature_enabled, got: {resub_str}"
        );
    }

    /// Test that mirrors the exact usage pattern from GitHub issue #185.
    /// <https://github.com/Polymarket/rs-clob-client/issues/185>
    #[tokio::test]
    async fn best_bid_ask_stream_continues_after_reconnect() {
        let mut server = ReconnectableMockServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, config()).unwrap();

        let asset_id = payloads::asset_id();

        // Exact pattern from issue #185:
        // let stream = client.subscribe_best_bid_ask(asset_ids)?;
        // let mut stream = Box::pin(stream);
        let stream = client.subscribe_best_bid_ask(vec![asset_id]).unwrap();
        let mut stream = Box::pin(stream);

        // Consume initial subscription request
        let _: Option<String> = server.recv_subscription().await;

        // Send best_bid_ask message before disconnect
        let best_bid_ask_msg = serde_json::json!({
            "event_type": "best_bid_ask",
            "market": payloads::MARKET_STR,
            "asset_id": asset_id.to_string(),
            "best_bid": "0.48",
            "best_ask": "0.52",
            "spread": "0.04",
            "timestamp": "1234567890000"
        });
        server.send(&best_bid_ask_msg.to_string());

        // Verify we receive message before disconnect (mirrors the issue's loop pattern)
        let msg1 = timeout(Duration::from_secs(2), stream.next()).await;
        assert!(
            msg1.is_ok() && msg1.unwrap().is_some(),
            "Should receive best_bid_ask message before disconnect"
        );

        // Simulate disconnect (what the user experienced)
        server.disconnect_all();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Allow reconnection
        server.allow_reconnect();

        // Wait for re-subscription (proves reconnection happened)
        let resub = server.recv_subscription().await;
        assert!(
            resub.is_some(),
            "Should receive re-subscription after reconnect"
        );
        assert!(
            resub.unwrap().contains("\"custom_feature_enabled\":true"),
            "Re-subscription must include custom_feature_enabled for best_bid_ask to work"
        );

        // Send best_bid_ask message AFTER reconnection
        let best_bid_ask_msg2 = serde_json::json!({
            "event_type": "best_bid_ask",
            "market": payloads::MARKET_STR,
            "asset_id": asset_id.to_string(),
            "best_bid": "0.50",
            "best_ask": "0.54",
            "spread": "0.04",
            "timestamp": "1234567891000"
        });
        server.send(&best_bid_ask_msg2.to_string());

        // THE FIX: This should now work - stream should receive message after reconnection
        // Before the fix, this would hang forever because the server wasn't sending
        // best_bid_ask messages (custom_feature_enabled was not included in re-subscription)
        let msg2 = timeout(Duration::from_secs(2), stream.next()).await;
        assert!(
            msg2.is_ok() && msg2.unwrap().is_some(),
            "Should receive best_bid_ask message after reconnection - this was the bug in issue #185"
        );
    }
}

mod unsubscribe {
    use super::*;
    use crate::payloads::OTHER_ASSET_ID_STR;

    #[tokio::test]
    async fn unsubscribe_sends_request_when_refcount_reaches_zero() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::asset_id();

        // Subscribe once
        let _stream = client.subscribe_orderbook(vec![asset_id]).unwrap();
        let sub = server.recv_subscription().await.unwrap();
        assert!(sub.contains(&asset_id.to_string()));

        // Unsubscribe - should send unsubscribe request since refcount goes to 0
        client.unsubscribe_orderbook(&[asset_id]).unwrap();

        let unsub = server.recv_subscription().await.unwrap();
        assert!(
            unsub.contains("\"operation\":\"unsubscribe\""),
            "Should send unsubscribe request, got: {unsub}"
        );
        assert!(unsub.contains(&asset_id.to_string()));
    }

    #[tokio::test]
    async fn unsubscribe_does_not_send_request_when_refcount_above_zero() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::asset_id();

        // Subscribe twice to same asset
        let _stream1 = client.subscribe_orderbook(vec![asset_id]).unwrap();
        let _: Option<String> = server.recv_subscription().await;

        let _stream2 = client.subscribe_orderbook(vec![asset_id]).unwrap();
        // Second subscribe should not send (multiplexed)

        // Unsubscribe once - refcount goes from 2 to 1, should NOT send request
        client.unsubscribe_orderbook(&[asset_id]).unwrap();

        // Subscribe to different asset to verify server is still responsive
        let _stream3 = client
            .subscribe_orderbook(vec![payloads::other_asset_id()])
            .unwrap();

        let next_msg = server.recv_subscription().await.unwrap();
        // Should be a subscribe for OTHER_ASSET_ID, not an unsubscribe for ASSET_ID
        assert!(
            next_msg.contains(OTHER_ASSET_ID_STR),
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

        let asset_id = payloads::asset_id();

        // Subscribe three times
        let _stream1 = client.subscribe_orderbook(vec![asset_id]).unwrap();
        let _: Option<String> = server.recv_subscription().await;

        let _stream2 = client.subscribe_orderbook(vec![asset_id]).unwrap();
        let _stream3 = client.subscribe_orderbook(vec![asset_id]).unwrap();

        // Unsubscribe twice - still one stream left
        client.unsubscribe_orderbook(&[asset_id]).unwrap();
        client.unsubscribe_orderbook(&[asset_id]).unwrap();

        // Third unsubscribe - now refcount hits 0, should send request
        client.unsubscribe_orderbook(&[asset_id]).unwrap();

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

        let asset_id = payloads::asset_id();

        // Subscribe
        let _stream1 = client.subscribe_orderbook(vec![asset_id]).unwrap();
        let sub1 = server.recv_subscription().await.unwrap();
        assert!(sub1.contains(&asset_id.to_string()));

        // Fully unsubscribe
        client.unsubscribe_orderbook(&[asset_id]).unwrap();
        let unsub = server.recv_subscription().await.unwrap();
        assert!(unsub.contains("\"operation\":\"unsubscribe\""));

        // Re-subscribe should send a new subscription request
        let stream2 = client.subscribe_orderbook(vec![asset_id]).unwrap();
        let mut stream2 = Box::pin(stream2);

        let sub2 = server.recv_subscription().await.unwrap();
        assert!(
            sub2.contains("\"type\":\"market\""),
            "Should send new subscribe request after full unsubscribe"
        );
        assert!(sub2.contains(&asset_id.to_string()));

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
            .subscribe_orderbook(vec![payloads::asset_id()])
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

        let asset_id = payloads::asset_id();
        let nonexistent_asset = payloads::other_asset_id();

        // Subscribe to one asset
        let _stream = client.subscribe_orderbook(vec![asset_id]).unwrap();
        let _: Option<String> = server.recv_subscription().await;

        // Unsubscribe from asset we never subscribed to - should be no-op
        client.unsubscribe_orderbook(&[nonexistent_asset]).unwrap();

        // Subscribe to another asset to verify server didn't receive unsubscribe
        let _stream2 = client.subscribe_orderbook(vec![nonexistent_asset]).unwrap();

        let next_msg = server.recv_subscription().await.unwrap();
        // Should be a subscribe, not an unsubscribe
        assert!(
            next_msg.contains("\"type\":\"market\""),
            "Should receive subscribe, not unsubscribe for non-existent sub. Got: {next_msg}"
        );
    }

    /// Stress test for concurrent subscribe/unsubscribe operations.
    ///
    /// This test verifies that the atomic reference counting in
    /// `SubscriptionManager` prevents race conditions when multiple
    /// tasks subscribe and unsubscribe to the same asset concurrently.
    ///
    /// The test creates N concurrent tasks that each subscribe and
    /// unsubscribe in a loop, then verifies that the final state is
    /// consistent (either fully subscribed or fully unsubscribed).
    #[tokio::test]
    async fn concurrent_subscribe_unsubscribe_maintains_consistency() {
        const NUM_TASKS: usize = 10;
        const ITERATIONS: usize = 50;

        let server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Arc::new(Client::new(&endpoint, Config::default()).unwrap());
        let asset_id = payloads::asset_id();

        // Spawn multiple tasks that race to subscribe and unsubscribe
        let mut handles = Vec::with_capacity(NUM_TASKS);
        for _ in 0..NUM_TASKS {
            let client = Arc::clone(&client);
            let handle = tokio::spawn(async move {
                for _ in 0..ITERATIONS {
                    // Subscribe
                    let _stream = client.subscribe_orderbook(vec![asset_id]).unwrap();

                    // Small yield to increase interleaving
                    tokio::task::yield_now().await;

                    // Unsubscribe
                    client.unsubscribe_orderbook(&[asset_id]).unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("task should not panic");
        }

        // Final verification: after all tasks complete, subscription count should be 0
        // This verifies no reference count corruption occurred during concurrent operations
        assert_eq!(
            client.subscription_count(),
            0,
            "All subscriptions should be cleaned up after concurrent operations"
        );
    }
}

mod client_state {
    use polymarket_client_sdk::clob::ws::ChannelType;

    use super::*;

    #[tokio::test]
    async fn is_connected_returns_false_before_subscription() {
        let server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        // Before any subscription, connection should not be established
        assert!(!client.is_connected(ChannelType::Market));
        assert!(!client.is_connected(ChannelType::User));
    }

    #[tokio::test]
    async fn is_connected_returns_true_after_subscription() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        // Subscribe to trigger connection
        let _stream = client
            .subscribe_orderbook(vec![payloads::asset_id()])
            .unwrap();
        let _: Option<String> = server.recv_subscription().await;

        // Now should be connected
        assert!(client.is_connected(ChannelType::Market));
        assert!(!client.is_connected(ChannelType::User));
    }

    #[tokio::test]
    async fn connection_state_is_connected_after_subscription() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        // Subscribe to trigger connection
        let _stream = client
            .subscribe_orderbook(vec![payloads::asset_id()])
            .unwrap();
        let _: Option<String> = server.recv_subscription().await;

        // Allow connection to establish
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Now should be connected
        assert!(client.connection_state(ChannelType::Market).is_connected());
        assert!(!client.connection_state(ChannelType::User).is_connected());
    }

    #[tokio::test]
    async fn subscription_count_increases_with_subscriptions() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let _stream1 = client
            .subscribe_orderbook(vec![payloads::asset_id()])
            .unwrap();
        let _: Option<String> = server.recv_subscription().await;

        assert_eq!(client.subscription_count(), 1);

        let _stream2 = client
            .subscribe_prices(vec![payloads::other_asset_id()])
            .unwrap();
        let _: Option<String> = server.recv_subscription().await;

        assert_eq!(client.subscription_count(), 2);
    }
}

mod unsubscribe_variants {
    use super::*;

    #[tokio::test]
    async fn unsubscribe_prices_sends_request() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::asset_id();

        // Subscribe via prices
        let _stream = client.subscribe_prices(vec![asset_id]).unwrap();
        let _: Option<String> = server.recv_subscription().await;

        // Unsubscribe via prices
        client.unsubscribe_prices(&[asset_id]).unwrap();

        let unsub = server.recv_subscription().await.unwrap();
        assert!(unsub.contains("\"operation\":\"unsubscribe\""));
        assert!(unsub.contains(&asset_id.to_string()));
    }

    #[tokio::test]
    async fn unsubscribe_tick_size_change_sends_request() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::asset_id();

        // Subscribe via tick size changes
        let _stream = client.subscribe_tick_size_change(vec![asset_id]).unwrap();
        let _: Option<String> = server.recv_subscription().await;

        // Unsubscribe via tick size changes
        client.unsubscribe_tick_size_change(&[asset_id]).unwrap();

        let unsub = server.recv_subscription().await.unwrap();
        assert!(unsub.contains("\"operation\":\"unsubscribe\""));
        assert!(unsub.contains(&asset_id.to_string()));
    }

    #[tokio::test]
    async fn unsubscribe_midpoints_sends_request() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let asset_id = payloads::asset_id();

        // Subscribe via midpoints
        let _stream = client.subscribe_midpoints(vec![asset_id]).unwrap();
        let _: Option<String> = server.recv_subscription().await;

        // Unsubscribe via midpoints
        client.unsubscribe_midpoints(&[asset_id]).unwrap();

        let unsub = server.recv_subscription().await.unwrap();
        assert!(unsub.contains("\"operation\":\"unsubscribe\""));
        assert!(unsub.contains(&asset_id.to_string()));
    }
}

mod custom_features {
    use rust_decimal_macros::dec;

    use super::*;

    pub fn best_bid_ask() -> serde_json::Value {
        json!({
            "event_type": "best_bid_ask",
            "market": payloads::MARKET_STR,
            "asset_id": payloads::asset_id(),
            "best_bid": "0.48",
            "best_ask": "0.52",
            "spread": "0.04",
            "timestamp": "1234567890000"
        })
    }

    pub fn new_market() -> serde_json::Value {
        json!({
            "event_type": "new_market",
            "id": "12345",
            "question": "Will it rain tomorrow?",
            "market": payloads::MARKET_STR,
            "slug": "will-it-rain-tomorrow",
            "description": "A test market",
            "assets_ids": [payloads::asset_id()],
            "outcomes": ["Yes", "No"],
            "timestamp": "1234567890000"
        })
    }

    pub fn market_resolved() -> serde_json::Value {
        json!({
            "event_type": "market_resolved",
            "id": "12345",
            "question": "Will it rain tomorrow?",
            "market": payloads::MARKET_STR,
            "slug": "will-it-rain-tomorrow",
            "description": "A test market",
            "assets_ids": [payloads::asset_id()],
            "outcomes": ["Yes", "No"],
            "winning_asset_id": payloads::asset_id(),
            "winning_outcome": "Yes",
            "timestamp": "1234567890000"
        })
    }

    #[tokio::test]
    async fn subscribe_best_bid_ask_receives_updates() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let stream = client
            .subscribe_best_bid_ask(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Verify subscription with custom_feature_enabled
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(sub_request.contains("\"type\":\"market\""));
        assert!(sub_request.contains("\"custom_feature_enabled\":true"));

        // Send best_bid_ask message
        server.send(&best_bid_ask().to_string());

        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let bba = result.unwrap().unwrap().unwrap();

        assert_eq!(bba.asset_id, payloads::asset_id());
        assert_eq!(bba.market, payloads::MARKET);
        assert_eq!(bba.best_bid, dec!(0.48));
        assert_eq!(bba.best_ask, dec!(0.52));
        assert_eq!(bba.spread, dec!(0.04));
    }

    #[tokio::test]
    async fn subscribe_new_markets_receives_updates() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let stream = client
            .subscribe_new_markets(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Verify subscription with custom_feature_enabled
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(sub_request.contains("\"custom_feature_enabled\":true"));

        // Send new_market message
        server.send(&new_market().to_string());

        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let nm = result.unwrap().unwrap().unwrap();

        assert_eq!(nm.id, "12345");
        assert_eq!(nm.question, "Will it rain tomorrow?");
        assert_eq!(nm.market, payloads::MARKET);
        assert_eq!(nm.slug, "will-it-rain-tomorrow");
        assert_eq!(nm.asset_ids, vec![payloads::asset_id()]);
        assert_eq!(nm.outcomes, vec!["Yes", "No"]);
    }

    #[tokio::test]
    async fn subscribe_market_resolutions_receives_updates() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let stream = client
            .subscribe_market_resolutions(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Verify subscription with custom_feature_enabled
        let sub_request = server.recv_subscription().await.unwrap();
        assert!(sub_request.contains("\"custom_feature_enabled\":true"));

        // Send market_resolved message
        server.send(&market_resolved().to_string());

        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let mr = result.unwrap().unwrap().unwrap();

        assert_eq!(mr.id, "12345");
        assert_eq!(mr.question, "Will it rain tomorrow?");
        assert_eq!(mr.market, payloads::MARKET);
        assert_eq!(mr.slug, "will-it-rain-tomorrow");
        assert_eq!(mr.asset_ids, vec![payloads::asset_id()]);
    }

    #[tokio::test]
    async fn subscribe_best_bid_ask_filters_other_messages() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let stream = client
            .subscribe_best_bid_ask(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send a book message (should be filtered out)
        server.send(&payloads::book().to_string());

        // Send best_bid_ask message
        server.send(&best_bid_ask().to_string());

        // Should only receive best_bid_ask
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let bba = result.unwrap().unwrap().unwrap();
        assert_eq!(bba.best_bid, dec!(0.48));
    }

    #[tokio::test]
    async fn subscribe_new_markets_filters_other_messages() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let stream = client
            .subscribe_new_markets(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send a book message (should be filtered out)
        server.send(&payloads::book().to_string());

        // Send a best_bid_ask message (should also be filtered out)
        server.send(&best_bid_ask().to_string());

        // Send new_market message
        server.send(&new_market().to_string());

        // Should only receive new_market
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let nm = result.unwrap().unwrap().unwrap();
        assert_eq!(nm.id, "12345");
    }

    #[tokio::test]
    async fn subscribe_market_resolutions_filters_other_messages() {
        let mut server = MockWsServer::start().await;
        let endpoint = server.ws_url("/ws/market");

        let client = Client::new(&endpoint, Config::default()).unwrap();

        let stream = client
            .subscribe_market_resolutions(vec![payloads::asset_id()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send a book message (should be filtered out)
        server.send(&payloads::book().to_string());

        // Send a new_market message (should also be filtered out)
        server.send(&new_market().to_string());

        // Send market_resolved message
        server.send(&market_resolved().to_string());

        // Should only receive market_resolved
        let result = timeout(Duration::from_secs(2), stream.next()).await;
        let mr = result.unwrap().unwrap().unwrap();
        assert_eq!(mr.id, "12345");
    }
}

mod message_parsing {
    use std::str::FromStr as _;

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
            .subscribe_orderbook(vec![payloads::asset_id()])
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

        let asset_a_str =
            "71321045679252212594626385532706912750332728571942532289631379312455583992563";
        let asset_b_str =
            "88888888888888888888888888888888888888888888888888888888888888888888888888888";
        let asset_a = U256::from_str(asset_a_str).unwrap();
        let asset_b = U256::from_str(asset_b_str).unwrap();

        let stream = client.subscribe_prices(vec![asset_a, asset_b]).unwrap();
        let mut stream = Box::pin(stream);

        let _: Option<String> = server.recv_subscription().await;

        // Send batch price change with two assets
        let batch_msg = json!({
            "market": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
            "price_changes": [
                {
                    "asset_id": asset_a_str,
                    "price": "0.5",
                    "size": "200",
                    "side": "BUY",
                    "hash": "56621a121a47ed9333273e21c83b660cff37ae50",
                    "best_bid": "0.5",
                    "best_ask": "1"
                },
                {
                    "asset_id": asset_b_str,
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

        assert_eq!(tsc.asset_id, payloads::asset_id());
        assert_eq!(tsc.market, payloads::MARKET);
        assert_eq!(tsc.old_tick_size, dec!(0.01));
        assert_eq!(tsc.new_tick_size, dec!(0.001));
        assert_eq!(tsc.timestamp, 100_000_000);
    }

    #[test]
    fn parses_last_trade_price() {
        let asset_id_str =
            "114122071509644379678018727908709560226618148003371446110114509806601493071694";
        let asset_id = U256::from_str(asset_id_str).unwrap();
        let payload = payloads::last_trade_price(asset_id_str).to_string();
        let ltp: LastTradePrice = serde_json::from_str(&payload).unwrap();

        assert_eq!(ltp.asset_id, asset_id);
        assert_eq!(
            ltp.market,
            b256!("6a67b9d828d53862160e470329ffea5246f338ecfffdf2cab45211ec578b0347")
        );
        assert_eq!(ltp.price, dec!(0.456));
        assert_eq!(ltp.side, Some(Side::Buy));
        assert_eq!(ltp.timestamp, 1_750_428_146_322);
    }
}
