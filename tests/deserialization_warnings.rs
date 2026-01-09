//! Integration tests for deserialization warning system.
//!
//! Tests that the library correctly emits warnings/errors for:
//! 1. Unknown fields in API responses
//! 2. Type mismatch errors
//! 3. Unknown enum variants

use std::future::Future;
use std::sync::{Arc, Mutex};

use httpmock::MockServer;
use polymarket_client_sdk::clob::{Client as ClobClient, Config};
use polymarket_client_sdk::data::Client as DataClient;
use polymarket_client_sdk::types::address;
use serde_json::json;
use tracing_subscriber::layer::SubscriberExt as _;

/// Captures tracing output for assertion.
fn with_captured_logs<F, Fut, R>(f: F) -> (R, String)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    let logs: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = Arc::clone(&logs);

    let layer = tracing_subscriber::fmt::layer()
        .with_writer(move || {
            struct CaptureWriter(Arc<Mutex<Vec<String>>>);
            impl std::io::Write for CaptureWriter {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    if let Ok(s) = std::str::from_utf8(buf) {
                        self.0.lock().expect("lock").push(s.to_owned());
                    }
                    Ok(buf.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }
            CaptureWriter(Arc::clone(&logs_clone))
        })
        .with_ansi(false);

    let subscriber = tracing_subscriber::registry().with(layer);

    let result = tracing::subscriber::with_default(subscriber, || {
        // We need to block on the future within the subscriber context
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(f()))
    });

    let captured = logs.lock().expect("lock").join("");

    (result, captured)
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_field_emits_warning() {
    let server = MockServer::start();
    let client = ClobClient::new(&server.base_url(), Config::default()).unwrap();

    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path("/midpoint")
            .query_param("token_id", "123");
        then.status(200).json_body(json!({
            "mid": "0.55",
            "new_api_field": "some_value",
        }));
    });

    let (result, logs) = with_captured_logs(|| async {
        client
            .midpoint(
                &polymarket_client_sdk::clob::types::request::MidpointRequest::builder()
                    .token_id("123")
                    .build(),
            )
            .await
    });

    mock.assert();
    assert!(result.is_ok(), "request should succeed");
    assert!(
        logs.contains("unknown field"),
        "expected 'unknown field' warning, got: {logs}"
    );
    assert!(
        logs.contains("new_api_field"),
        "expected field name in warning, got: {logs}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn type_mismatch_emits_error() {
    let server = MockServer::start();
    let client = ClobClient::new(&server.base_url(), Config::default()).unwrap();

    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path("/midpoint")
            .query_param("token_id", "456");
        // Send an object where a decimal string is expected
        then.status(200)
            .json_body(json!({ "mid": { "invalid": "type" } }));
    });

    let (result, logs) = with_captured_logs(|| async {
        client
            .midpoint(
                &polymarket_client_sdk::clob::types::request::MidpointRequest::builder()
                    .token_id("456")
                    .build(),
            )
            .await
    });

    mock.assert();
    assert!(result.is_err(), "request should fail due to type mismatch");
    assert!(
        logs.contains("deserialization failed"),
        "expected 'deserialization failed' error, got: {logs}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_enum_variant_emits_warning() {
    let server = MockServer::start();
    let client = DataClient::new(&server.base_url()).unwrap();

    let user_addr = address!("0x1234567890123456789012345678901234567890");

    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path("/trades")
            .query_param("user", "0x1234567890123456789012345678901234567890");
        then.status(200).json_body(json!([{
            "proxyWallet": "0x1234567890123456789012345678901234567890",
            "side": "NEW_SIDE_TYPE",
            "asset": "12345",
            "conditionId": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",
            "size": "10.0",
            "price": "0.50",
            "timestamp": 1_700_000_000,
            "title": "Test Market",
            "slug": "test-market",
            "icon": "https://example.com/icon.png",
            "eventSlug": "test-event",
            "outcome": "Yes",
            "outcomeIndex": 0,
            "transactionHash": "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd"
        }]));
    });

    let (result, logs) = with_captured_logs(|| async {
        client
            .trades(
                &polymarket_client_sdk::data::types::request::TradesRequest::builder()
                    .user(user_addr)
                    .build(),
            )
            .await
    });

    mock.assert();
    assert!(result.is_ok(), "request should succeed");
    assert!(
        logs.contains("unknown enum variant"),
        "expected 'unknown enum variant' warning, got: {logs}"
    );
    assert!(
        logs.contains("NEW_SIDE_TYPE"),
        "expected unknown value in warning, got: {logs}"
    );
}
