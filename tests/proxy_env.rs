//! Tests for proxy environment variable support.

#[cfg(any(feature = "ws", feature = "rtds"))]
mod env_var_tests {
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F, R>(vars: &[(&str, &str)], f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _lock = ENV_LOCK.lock().expect("env lock poisoned");

        // Clear all proxy-related env vars
        let all_vars = [
            "all_proxy",
            "ALL_PROXY",
            "https_proxy",
            "HTTPS_PROXY",
            "http_proxy",
            "HTTP_PROXY",
            "no_proxy",
            "NO_PROXY",
        ];
        for var in &all_vars {
            // SAFETY: We hold a mutex lock to ensure no concurrent access to env vars
            unsafe { std::env::remove_var(var) };
        }

        // Set the requested vars
        for (key, value) in vars {
            // SAFETY: We hold a mutex lock to ensure no concurrent access to env vars
            unsafe { std::env::set_var(key, value) };
        }

        let result = f();

        // Clean up
        for var in &all_vars {
            // SAFETY: We hold a mutex lock to ensure no concurrent access to env vars
            unsafe { std::env::remove_var(var) };
        }

        result
    }

    #[test]
    fn returns_none_when_no_env_vars_set() {
        with_env(&[], || {
            assert_eq!(polymarket_client_sdk::proxy::from_env("example.com"), None);
        });
    }

    #[test]
    fn returns_all_proxy_lowercase() {
        with_env(&[("all_proxy", "socks5://proxy:1080")], || {
            assert_eq!(
                polymarket_client_sdk::proxy::from_env("example.com"),
                Some("socks5://proxy:1080".into())
            );
        });
    }

    #[test]
    fn returns_all_proxy_uppercase() {
        with_env(&[("ALL_PROXY", "socks5://proxy:1080")], || {
            assert_eq!(
                polymarket_client_sdk::proxy::from_env("example.com"),
                Some("socks5://proxy:1080".into())
            );
        });
    }

    #[test]
    fn lowercase_takes_precedence_over_uppercase() {
        with_env(
            &[
                ("all_proxy", "socks5://lower:1080"),
                ("ALL_PROXY", "socks5://upper:1080"),
            ],
            || {
                assert_eq!(
                    polymarket_client_sdk::proxy::from_env("example.com"),
                    Some("socks5://lower:1080".into())
                );
            },
        );
    }

    #[test]
    fn all_proxy_takes_precedence_over_https_proxy() {
        with_env(
            &[
                ("ALL_PROXY", "socks5://all:1080"),
                ("HTTPS_PROXY", "http://https:8080"),
            ],
            || {
                assert_eq!(
                    polymarket_client_sdk::proxy::from_env("example.com"),
                    Some("socks5://all:1080".into())
                );
            },
        );
    }

    #[test]
    fn https_proxy_takes_precedence_over_http_proxy() {
        with_env(
            &[
                ("HTTPS_PROXY", "http://https:8080"),
                ("HTTP_PROXY", "http://http:8080"),
            ],
            || {
                assert_eq!(
                    polymarket_client_sdk::proxy::from_env("example.com"),
                    Some("http://https:8080".into())
                );
            },
        );
    }

    #[test]
    fn returns_http_proxy_as_fallback() {
        with_env(&[("HTTP_PROXY", "http://proxy:8080")], || {
            assert_eq!(
                polymarket_client_sdk::proxy::from_env("example.com"),
                Some("http://proxy:8080".into())
            );
        });
    }

    #[test]
    fn no_proxy_exact_match() {
        with_env(
            &[
                ("ALL_PROXY", "socks5://proxy:1080"),
                ("NO_PROXY", "localhost,example.com"),
            ],
            || {
                assert_eq!(polymarket_client_sdk::proxy::from_env("localhost"), None);
                assert_eq!(polymarket_client_sdk::proxy::from_env("example.com"), None);
                assert_eq!(
                    polymarket_client_sdk::proxy::from_env("other.com"),
                    Some("socks5://proxy:1080".into())
                );
            },
        );
    }

    #[test]
    fn no_proxy_suffix_match_with_dot() {
        with_env(
            &[
                ("ALL_PROXY", "socks5://proxy:1080"),
                ("NO_PROXY", ".example.com"),
            ],
            || {
                // Suffix match: foo.example.com should match
                assert_eq!(
                    polymarket_client_sdk::proxy::from_env("foo.example.com"),
                    None
                );
                // Exact domain (without leading dot) should also match
                assert_eq!(polymarket_client_sdk::proxy::from_env("example.com"), None);
                // Different domain should not match
                assert_eq!(
                    polymarket_client_sdk::proxy::from_env("notexample.com"),
                    Some("socks5://proxy:1080".into())
                );
            },
        );
    }

    #[test]
    fn no_proxy_suffix_match_without_dot() {
        with_env(
            &[
                ("ALL_PROXY", "socks5://proxy:1080"),
                ("NO_PROXY", "example.com"),
            ],
            || {
                // Exact match
                assert_eq!(polymarket_client_sdk::proxy::from_env("example.com"), None);
                // Subdomain should also match (implicit suffix)
                assert_eq!(
                    polymarket_client_sdk::proxy::from_env("foo.example.com"),
                    None
                );
                // Different domain should not match
                assert_eq!(
                    polymarket_client_sdk::proxy::from_env("notexample.com"),
                    Some("socks5://proxy:1080".into())
                );
            },
        );
    }

    #[test]
    fn no_proxy_wildcard_disables_all() {
        with_env(
            &[("ALL_PROXY", "socks5://proxy:1080"), ("NO_PROXY", "*")],
            || {
                assert_eq!(polymarket_client_sdk::proxy::from_env("any.host.com"), None);
                assert_eq!(polymarket_client_sdk::proxy::from_env("localhost"), None);
                assert_eq!(polymarket_client_sdk::proxy::from_env("192.168.1.1"), None);
            },
        );
    }

    #[test]
    fn no_proxy_handles_whitespace() {
        with_env(
            &[
                ("ALL_PROXY", "socks5://proxy:1080"),
                ("NO_PROXY", " localhost , example.com "),
            ],
            || {
                assert_eq!(polymarket_client_sdk::proxy::from_env("localhost"), None);
                assert_eq!(polymarket_client_sdk::proxy::from_env("example.com"), None);
            },
        );
    }

    #[test]
    fn no_proxy_lowercase_takes_precedence() {
        with_env(
            &[
                ("ALL_PROXY", "socks5://proxy:1080"),
                ("no_proxy", "localhost"),
                ("NO_PROXY", "example.com"),
            ],
            || {
                assert_eq!(polymarket_client_sdk::proxy::from_env("localhost"), None);
                // NO_PROXY is not used because no_proxy is set
                assert_eq!(
                    polymarket_client_sdk::proxy::from_env("example.com"),
                    Some("socks5://proxy:1080".into())
                );
            },
        );
    }
}
