//! Gamma API streaming endpoint explorer.
//!
//! This example demonstrates streaming data from Gamma API endpoints using offset-based
//! pagination and single-call endpoints. It covers all response types:
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example gamma_streaming --features gamma,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=gamma_streaming.log RUST_LOG=info cargo run --example gamma_streaming --features gamma,tracing
//! ```

use std::fs::File;

use futures::StreamExt as _;
use polymarket_client_sdk::gamma::{
    Client,
    types::request::{EventsRequest, MarketsRequest},
};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Ok(path) = std::env::var("LOG_FILE") {
        let file = File::create(path)?;
        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file)
                    .with_ansi(false),
            )
            .init();
    } else {
        tracing_subscriber::fmt::init();
    }

    let client = Client::default();

    stream_events(&client).await?;
    stream_markets(&client).await?;

    Ok(())
}

/// Streams events from the Gamma API.
async fn stream_events(client: &Client) -> anyhow::Result<()> {
    info!(stream = "events", "starting stream");

    let mut stream = client
        .stream_data(
            |c, limit, offset| {
                let request = EventsRequest::builder()
                    .active(true)
                    .limit(limit)
                    .offset(offset)
                    .build();
                async move { c.events(&request).await }
            },
            100,
        )
        .take(100)
        .boxed();

    let mut count = 0_u32;

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                count += 1;
                info!(stream = "events", count, "{event:?}");
            }
            Err(e) => {
                warn!(stream = "events", error = %e, "stream error");
                break;
            }
        }
    }

    info!(stream = "events", total = count, "stream completed");
    Ok(())
}

/// Streams markets from the Gamma API.
async fn stream_markets(client: &Client) -> anyhow::Result<()> {
    info!(stream = "markets", "starting stream");

    let mut stream = client
        .stream_data(
            |c, limit, offset| {
                let request = MarketsRequest::builder()
                    .closed(false)
                    .limit(limit)
                    .offset(offset)
                    .build();
                async move { c.markets(&request).await }
            },
            100,
        )
        .take(100)
        .boxed();

    let mut count = 0_u32;

    while let Some(result) = stream.next().await {
        match result {
            Ok(market) => {
                count += 1;
                info!(stream = "markets", count, "{market:?}");
            }
            Err(e) => {
                warn!(stream = "markets", error = %e, "stream error");
                break;
            }
        }
    }

    info!(stream = "markets", total = count, "stream completed");
    Ok(())
}
