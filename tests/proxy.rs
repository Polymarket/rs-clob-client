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
