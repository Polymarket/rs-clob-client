#![allow(
    clippy::unwrap_used,
    reason = "Do not need additional syntax for setting up tests, and https://github.com/rust-lang/rust-clippy/issues/13981"
)]

//! Tests for HTTP and WebSocket proxy support.
//!
//! These tests verify:
//! - Client construction with proxy configuration
//! - Invalid proxy URL rejection
//! - WebSocket Config proxy builder

mod gamma_proxy {
    #[cfg(feature = "gamma")]
    mod tests {
        use polymarket_client_sdk::gamma::Client;

        #[test]
        fn new_with_proxy_none_should_succeed() {
            Client::new_with_proxy("https://gamma-api.polymarket.com", None).unwrap();
        }

        #[test]
        fn new_with_proxy_http_should_succeed() {
            Client::new_with_proxy(
                "https://gamma-api.polymarket.com",
                Some("http://proxy.example.com:8080"),
            )
            .unwrap();
        }

        #[test]
        fn new_with_proxy_socks5_should_succeed() {
            Client::new_with_proxy(
                "https://gamma-api.polymarket.com",
                Some("socks5://127.0.0.1:1080"),
            )
            .unwrap();
        }

        #[test]
        fn new_with_proxy_with_auth_should_succeed() {
            Client::new_with_proxy(
                "https://gamma-api.polymarket.com",
                Some("http://user:pass@proxy.example.com:8080"),
            )
            .unwrap();
        }

        #[test]
        fn new_with_proxy_invalid_url_should_fail() {
            let result = Client::new_with_proxy(
                "https://gamma-api.polymarket.com",
                Some("not a valid proxy url"),
            );
            let err = result.unwrap_err();
            assert!(err.to_string().contains("invalid proxy URL"));
        }
    }
}

mod data_proxy {
    #[cfg(feature = "data")]
    mod tests {
        use polymarket_client_sdk::data::Client;

        #[test]
        fn new_with_proxy_none_should_succeed() {
            Client::new_with_proxy("https://data-api.polymarket.com", None).unwrap();
        }

        #[test]
        fn new_with_proxy_http_should_succeed() {
            Client::new_with_proxy(
                "https://data-api.polymarket.com",
                Some("http://proxy.example.com:8080"),
            )
            .unwrap();
        }

        #[test]
        fn new_with_proxy_socks5_should_succeed() {
            Client::new_with_proxy(
                "https://data-api.polymarket.com",
                Some("socks5://127.0.0.1:1080"),
            )
            .unwrap();
        }

        #[test]
        fn new_with_proxy_with_auth_should_succeed() {
            Client::new_with_proxy(
                "https://data-api.polymarket.com",
                Some("http://user:pass@proxy.example.com:8080"),
            )
            .unwrap();
        }

        #[test]
        fn new_with_proxy_invalid_url_should_fail() {
            let result = Client::new_with_proxy(
                "https://data-api.polymarket.com",
                Some("not a valid proxy url"),
            );
            let err = result.unwrap_err();
            assert!(err.to_string().contains("invalid proxy URL"));
        }
    }
}

mod bridge_proxy {
    #[cfg(feature = "bridge")]
    mod tests {
        use polymarket_client_sdk::bridge::Client;

        #[test]
        fn new_with_proxy_none_should_succeed() {
            Client::new_with_proxy("https://bridge.polymarket.com", None).unwrap();
        }

        #[test]
        fn new_with_proxy_http_should_succeed() {
            Client::new_with_proxy(
                "https://bridge.polymarket.com",
                Some("http://proxy.example.com:8080"),
            )
            .unwrap();
        }

        #[test]
        fn new_with_proxy_socks5_should_succeed() {
            Client::new_with_proxy(
                "https://bridge.polymarket.com",
                Some("socks5://127.0.0.1:1080"),
            )
            .unwrap();
        }

        #[test]
        fn new_with_proxy_with_auth_should_succeed() {
            Client::new_with_proxy(
                "https://bridge.polymarket.com",
                Some("http://user:pass@proxy.example.com:8080"),
            )
            .unwrap();
        }

        #[test]
        fn new_with_proxy_invalid_url_should_fail() {
            let result = Client::new_with_proxy(
                "https://bridge.polymarket.com",
                Some("not a valid proxy url"),
            );
            let err = result.unwrap_err();
            assert!(err.to_string().contains("invalid proxy URL"));
        }
    }
}

mod clob_proxy {
    use polymarket_client_sdk::clob::{Client, Config};

    #[test]
    fn client_with_proxy_config_should_succeed() {
        let config = Config::builder()
            .proxy("http://proxy.example.com:8080")
            .build();
        Client::new("https://clob.polymarket.com", config).unwrap();
    }

    #[test]
    fn client_with_socks5_proxy_should_succeed() {
        let config = Config::builder().proxy("socks5://127.0.0.1:1080").build();
        Client::new("https://clob.polymarket.com", config).unwrap();
    }

    #[test]
    fn client_with_proxy_auth_should_succeed() {
        let config = Config::builder()
            .proxy("http://user:pass@proxy.example.com:8080")
            .build();
        Client::new("https://clob.polymarket.com", config).unwrap();
    }

    #[test]
    fn client_with_invalid_proxy_should_fail() {
        let config = Config::builder().proxy("not a valid proxy url").build();
        let result = Client::new("https://clob.polymarket.com", config);
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid proxy URL"));
    }
}

#[cfg(feature = "ws")]
mod ws_proxy {
    use polymarket_client_sdk::clob::ws::Config;

    #[test]
    fn default_config_has_no_proxy() {
        let config = Config::default();
        assert!(config.proxy.is_none());
    }

    #[test]
    fn with_proxy_sets_proxy_url() {
        let config = Config::with_proxy("http://proxy.example.com:8080");
        assert_eq!(
            config.proxy,
            Some("http://proxy.example.com:8080".to_owned())
        );
    }

    #[test]
    fn with_proxy_socks5_sets_proxy_url() {
        let config = Config::with_proxy("socks5://127.0.0.1:1080");
        assert_eq!(config.proxy, Some("socks5://127.0.0.1:1080".to_owned()));
    }

    #[test]
    fn with_proxy_preserves_other_defaults() {
        let default_config = Config::default();
        let proxy_config = Config::with_proxy("http://proxy:8080");

        assert_eq!(
            proxy_config.heartbeat_interval,
            default_config.heartbeat_interval
        );
        assert_eq!(
            proxy_config.heartbeat_timeout,
            default_config.heartbeat_timeout
        );
    }
}

#[cfg(feature = "ws")]
mod ws_proxy_connection {
    use std::net::SocketAddr;
    use std::time::Duration;

    use futures_util::{SinkExt as _, StreamExt as _};
    use polymarket_client_sdk::clob::ws::{Client, Config};
    use serde_json::json;
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    use tokio::net::TcpListener;
    use tokio::time::timeout;
    use tokio_tungstenite::tungstenite::Message;

    const ASSET_ID: &str = "123456789";

    /// Mock WebSocket server for proxy tests.
    struct MockWsServer {
        addr: SocketAddr,
    }

    impl MockWsServer {
        async fn start() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            tokio::spawn(async move {
                while let Ok((stream, _)) = listener.accept().await {
                    let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await else {
                        continue;
                    };

                    let (mut write, mut read) = ws_stream.split();

                    tokio::spawn(async move {
                        while let Some(Ok(msg)) = read.next().await {
                            if let Message::Text(text) = msg {
                                if text.as_str() == "PING" {
                                    continue;
                                }
                                // Echo back a book message for any subscription
                                let response = json!({
                                    "event_type": "book",
                                    "asset_id": ASSET_ID,
                                    "market": "0xabc",
                                    "bids": [{"price": "0.5", "size": "100"}],
                                    "asks": [{"price": "0.6", "size": "100"}],
                                    "timestamp": "1234567890",
                                    "hash": "0xdef"
                                });
                                if write
                                    .send(Message::Text(response.to_string().into()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                    });
                }
            });

            Self { addr }
        }

        fn base_url(&self) -> String {
            format!("ws://{}", self.addr)
        }
    }

    /// Mock SOCKS5 proxy server.
    struct MockSocks5Proxy {
        addr: SocketAddr,
    }

    impl MockSocks5Proxy {
        async fn start() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            tokio::spawn(async move {
                while let Ok((mut client, _)) = listener.accept().await {
                    tokio::spawn(async move {
                        let mut buf = [0_u8; 256];

                        // Read SOCKS5 greeting: version + num_methods + methods
                        let n = client.read(&mut buf).await.unwrap();
                        if n < 2 || buf[0] != 0x05 {
                            return;
                        }

                        // Respond: version 5, no auth required
                        client.write_all(&[0x05, 0x00]).await.unwrap();

                        // Read connect request
                        let n = client.read(&mut buf).await.unwrap();
                        if n < 7 || buf[0] != 0x05 || buf[1] != 0x01 {
                            return;
                        }

                        // Parse target address
                        let (target_host, target_port) = match buf[3] {
                            0x01 => {
                                // IPv4
                                let ip = format!("{}.{}.{}.{}", buf[4], buf[5], buf[6], buf[7]);
                                let port = u16::from_be_bytes([buf[8], buf[9]]);
                                (ip, port)
                            }
                            0x03 => {
                                // Domain name
                                let len = buf[4] as usize;
                                let domain = String::from_utf8_lossy(&buf[5..5 + len]).to_string();
                                let port = u16::from_be_bytes([buf[5 + len], buf[6 + len]]);
                                (domain, port)
                            }
                            _ => return,
                        };

                        // Connect to target
                        let Ok(target) =
                            tokio::net::TcpStream::connect(format!("{target_host}:{target_port}"))
                                .await
                        else {
                            // Connection refused response
                            client
                                .write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                                .await
                                .ok();
                            return;
                        };

                        // Success response
                        client
                            .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                            .await
                            .unwrap();

                        // Bidirectional forwarding
                        let (mut client_read, mut client_write) = client.into_split();
                        let (mut target_read, mut target_write) = target.into_split();

                        let c2t = tokio::spawn(async move {
                            tokio::io::copy(&mut client_read, &mut target_write)
                                .await
                                .ok();
                        });

                        let t2c = tokio::spawn(async move {
                            tokio::io::copy(&mut target_read, &mut client_write)
                                .await
                                .ok();
                        });

                        tokio::select! {
                            _ = c2t => {}
                            _ = t2c => {}
                        }
                    });
                }
            });

            Self { addr }
        }

        fn url(&self) -> String {
            format!("socks5://{}", self.addr)
        }
    }

    /// Mock HTTP CONNECT proxy server.
    struct MockHttpProxy {
        addr: SocketAddr,
    }

    impl MockHttpProxy {
        async fn start() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            tokio::spawn(async move {
                while let Ok((mut client, _)) = listener.accept().await {
                    tokio::spawn(async move {
                        let mut buf = [0_u8; 1024];

                        // Read HTTP CONNECT request
                        let n = client.read(&mut buf).await.unwrap();
                        let request = String::from_utf8_lossy(&buf[..n]);

                        // Parse CONNECT request
                        let first_line = request.lines().next().unwrap_or("");
                        if !first_line.starts_with("CONNECT ") {
                            client
                                .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                                .await
                                .ok();
                            return;
                        }

                        // Extract host:port
                        let parts: Vec<&str> = first_line.split_whitespace().collect();
                        if parts.len() < 2 {
                            client
                                .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                                .await
                                .ok();
                            return;
                        }
                        let target_addr = parts[1];

                        // Connect to target
                        let Ok(target) = tokio::net::TcpStream::connect(target_addr).await else {
                            client
                                .write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")
                                .await
                                .ok();
                            return;
                        };

                        // Send success response
                        client
                            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                            .await
                            .unwrap();

                        // Bidirectional forwarding
                        let (mut client_read, mut client_write) = client.into_split();
                        let (mut target_read, mut target_write) = target.into_split();

                        let c2t = tokio::spawn(async move {
                            tokio::io::copy(&mut client_read, &mut target_write)
                                .await
                                .ok();
                        });

                        let t2c = tokio::spawn(async move {
                            tokio::io::copy(&mut target_read, &mut client_write)
                                .await
                                .ok();
                        });

                        tokio::select! {
                            _ = c2t => {}
                            _ = t2c => {}
                        }
                    });
                }
            });

            Self { addr }
        }

        fn url(&self) -> String {
            format!("http://{}", self.addr)
        }
    }

    #[tokio::test]
    async fn websocket_receives_messages_via_socks5_proxy() {
        let ws_server = MockWsServer::start().await;
        let proxy = MockSocks5Proxy::start().await;

        let config = Config::with_proxy(proxy.url());
        let client = Client::new(&ws_server.base_url(), config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Wait for a message
        let result = timeout(Duration::from_secs(5), stream.next()).await;

        assert!(result.is_ok(), "Should receive message via SOCKS5 proxy");
        let book = result.unwrap().unwrap().unwrap();
        assert_eq!(book.asset_id, ASSET_ID);
    }

    #[tokio::test]
    async fn websocket_receives_messages_via_http_proxy() {
        let ws_server = MockWsServer::start().await;
        let proxy = MockHttpProxy::start().await;

        let config = Config::with_proxy(proxy.url());
        let client = Client::new(&ws_server.base_url(), config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Wait for a message
        let result = timeout(Duration::from_secs(5), stream.next()).await;

        assert!(result.is_ok(), "Should receive message via HTTP proxy");
        let book = result.unwrap().unwrap().unwrap();
        assert_eq!(book.asset_id, ASSET_ID);
    }

    /// Mock SOCKS5 proxy that requires authentication.
    struct MockSocks5ProxyWithAuth {
        addr: SocketAddr,
    }

    impl MockSocks5ProxyWithAuth {
        async fn start(expected_user: &'static str, expected_pass: &'static str) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            tokio::spawn(async move {
                while let Ok((mut client, _)) = listener.accept().await {
                    tokio::spawn(async move {
                        let mut buf = [0_u8; 256];

                        // Read SOCKS5 greeting
                        let n = client.read(&mut buf).await.unwrap();
                        if n < 2 || buf[0] != 0x05 {
                            return;
                        }

                        // Respond: version 5, username/password auth required (0x02)
                        client.write_all(&[0x05, 0x02]).await.unwrap();

                        // Read username/password auth
                        let n = client.read(&mut buf).await.unwrap();
                        if n < 3 || buf[0] != 0x01 {
                            return;
                        }

                        let ulen = buf[1] as usize;
                        let username = String::from_utf8_lossy(&buf[2..2 + ulen]).to_string();
                        let plen = buf[2 + ulen] as usize;
                        let password =
                            String::from_utf8_lossy(&buf[3 + ulen..3 + ulen + plen]).to_string();

                        if username != expected_user || password != expected_pass {
                            // Auth failed
                            client.write_all(&[0x01, 0x01]).await.unwrap();
                            return;
                        }

                        // Auth success
                        client.write_all(&[0x01, 0x00]).await.unwrap();

                        // Read connect request
                        let n = client.read(&mut buf).await.unwrap();
                        if n < 7 || buf[0] != 0x05 || buf[1] != 0x01 {
                            return;
                        }

                        // Parse target address
                        let (target_host, target_port) = match buf[3] {
                            0x01 => {
                                let ip = format!("{}.{}.{}.{}", buf[4], buf[5], buf[6], buf[7]);
                                let port = u16::from_be_bytes([buf[8], buf[9]]);
                                (ip, port)
                            }
                            0x03 => {
                                let len = buf[4] as usize;
                                let domain = String::from_utf8_lossy(&buf[5..5 + len]).to_string();
                                let port = u16::from_be_bytes([buf[5 + len], buf[6 + len]]);
                                (domain, port)
                            }
                            _ => return,
                        };

                        // Connect to target
                        let Ok(target) =
                            tokio::net::TcpStream::connect(format!("{target_host}:{target_port}"))
                                .await
                        else {
                            client
                                .write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                                .await
                                .ok();
                            return;
                        };

                        // Success response
                        client
                            .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                            .await
                            .unwrap();

                        // Bidirectional forwarding
                        let (mut client_read, mut client_write) = client.into_split();
                        let (mut target_read, mut target_write) = target.into_split();

                        let c2t = tokio::spawn(async move {
                            tokio::io::copy(&mut client_read, &mut target_write)
                                .await
                                .ok();
                        });

                        let t2c = tokio::spawn(async move {
                            tokio::io::copy(&mut target_read, &mut client_write)
                                .await
                                .ok();
                        });

                        tokio::select! {
                            _ = c2t => {}
                            _ = t2c => {}
                        }
                    });
                }
            });

            Self { addr }
        }

        fn url_with_auth(&self, user: &str, pass: &str) -> String {
            format!("socks5://{user}:{pass}@{}", self.addr)
        }
    }

    /// Mock HTTP proxy that requires authentication.
    struct MockHttpProxyWithAuth {
        addr: SocketAddr,
    }

    impl MockHttpProxyWithAuth {
        async fn start(expected_user: &'static str, expected_pass: &'static str) -> Self {
            use base64::Engine as _;

            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let expected_auth = base64::engine::general_purpose::STANDARD
                .encode(format!("{expected_user}:{expected_pass}"));

            tokio::spawn(async move {
                while let Ok((mut client, _)) = listener.accept().await {
                    let expected_auth = expected_auth.clone();
                    tokio::spawn(async move {
                        let mut buf = [0_u8; 1024];

                        let n = client.read(&mut buf).await.unwrap();
                        let request = String::from_utf8_lossy(&buf[..n]);

                        // Check for Proxy-Authorization header
                        let auth_ok = request.lines().any(|line| {
                            line.starts_with("Proxy-Authorization: Basic ")
                                && line
                                    .trim_start_matches("Proxy-Authorization: Basic ")
                                    .trim()
                                    == expected_auth
                        });

                        if !auth_ok {
                            client
                                .write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\n\r\n")
                                .await
                                .ok();
                            return;
                        }

                        // Parse CONNECT request
                        let first_line = request.lines().next().unwrap_or("");
                        if !first_line.starts_with("CONNECT ") {
                            client
                                .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                                .await
                                .ok();
                            return;
                        }

                        let parts: Vec<&str> = first_line.split_whitespace().collect();
                        if parts.len() < 2 {
                            client
                                .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                                .await
                                .ok();
                            return;
                        }
                        let target_addr = parts[1];

                        let Ok(target) = tokio::net::TcpStream::connect(target_addr).await else {
                            client
                                .write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")
                                .await
                                .ok();
                            return;
                        };

                        client
                            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                            .await
                            .unwrap();

                        let (mut client_read, mut client_write) = client.into_split();
                        let (mut target_read, mut target_write) = target.into_split();

                        let c2t = tokio::spawn(async move {
                            tokio::io::copy(&mut client_read, &mut target_write)
                                .await
                                .ok();
                        });

                        let t2c = tokio::spawn(async move {
                            tokio::io::copy(&mut target_read, &mut client_write)
                                .await
                                .ok();
                        });

                        tokio::select! {
                            _ = c2t => {}
                            _ = t2c => {}
                        }
                    });
                }
            });

            Self { addr }
        }

        fn url_with_auth(&self, user: &str, pass: &str) -> String {
            format!("http://{user}:{pass}@{}", self.addr)
        }
    }

    /// Mock HTTP proxy that always returns 403 Forbidden.
    struct MockHttpProxyForbidden {
        addr: SocketAddr,
    }

    impl MockHttpProxyForbidden {
        async fn start() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            tokio::spawn(async move {
                while let Ok((mut client, _)) = listener.accept().await {
                    tokio::spawn(async move {
                        let mut buf = [0_u8; 1024];
                        let _: Result<usize, _> = client.read(&mut buf).await;
                        client
                            .write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n")
                            .await
                            .ok();
                    });
                }
            });

            Self { addr }
        }

        fn url(&self) -> String {
            format!("http://{}", self.addr)
        }
    }

    #[tokio::test]
    async fn websocket_connects_via_socks5_proxy_with_auth() {
        let ws_server = MockWsServer::start().await;
        let proxy = MockSocks5ProxyWithAuth::start("testuser", "testpass").await;

        let config = Config::with_proxy(proxy.url_with_auth("testuser", "testpass"));
        let client = Client::new(&ws_server.base_url(), config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let result = timeout(Duration::from_secs(5), stream.next()).await;

        assert!(
            result.is_ok(),
            "Should receive message via authenticated SOCKS5 proxy"
        );
        let book = result.unwrap().unwrap().unwrap();
        assert_eq!(book.asset_id, ASSET_ID);
    }

    #[tokio::test]
    async fn websocket_connects_via_http_proxy_with_auth() {
        let ws_server = MockWsServer::start().await;
        let proxy = MockHttpProxyWithAuth::start("testuser", "testpass").await;

        let config = Config::with_proxy(proxy.url_with_auth("testuser", "testpass"));
        let client = Client::new(&ws_server.base_url(), config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        let result = timeout(Duration::from_secs(5), stream.next()).await;

        assert!(
            result.is_ok(),
            "Should receive message via authenticated HTTP proxy"
        );
        let book = result.unwrap().unwrap().unwrap();
        assert_eq!(book.asset_id, ASSET_ID);
    }

    #[tokio::test]
    async fn websocket_fails_on_http_proxy_forbidden() {
        let ws_server = MockWsServer::start().await;
        let proxy = MockHttpProxyForbidden::start().await;

        let config = Config::with_proxy(proxy.url());
        let client = Client::new(&ws_server.base_url(), config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Should timeout because proxy returns 403 and connection fails
        let result = timeout(Duration::from_millis(500), stream.next()).await;

        assert!(
            result.is_err(),
            "Should timeout when HTTP proxy returns 403 Forbidden"
        );
    }

    #[tokio::test]
    async fn websocket_fails_on_unsupported_proxy_scheme() {
        let ws_server = MockWsServer::start().await;

        // Use ftp:// which is not supported
        let config = Config::with_proxy("ftp://proxy.example.com:21");
        let client = Client::new(&ws_server.base_url(), config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Should timeout because proxy scheme is unsupported
        let result = timeout(Duration::from_millis(500), stream.next()).await;

        assert!(
            result.is_err(),
            "Should timeout when proxy scheme is unsupported"
        );
    }

    #[tokio::test]
    async fn websocket_fails_on_unreachable_proxy() {
        let ws_server = MockWsServer::start().await;

        // Use a port that's not listening
        let config = Config::with_proxy("socks5://127.0.0.1:59999");
        let client = Client::new(&ws_server.base_url(), config).unwrap();

        let stream = client
            .subscribe_orderbook(vec![ASSET_ID.to_owned()])
            .unwrap();
        let mut stream = Box::pin(stream);

        // Should timeout because proxy is unreachable
        let result = timeout(Duration::from_millis(500), stream.next()).await;

        assert!(result.is_err(), "Should timeout when proxy is unreachable");
    }
}
