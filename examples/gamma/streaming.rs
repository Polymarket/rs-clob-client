//! Gamma API streaming endpoint explorer.
//!
//! This example demonstrates streaming data from Gamma API endpoints using offset-based
//! pagination. It shows how to:
//! 1. Stream all active events with their markets
//! 2. Stream markets filtered by tags
//! 3. Stream teams from sports data
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example gamma-streaming --features gamma,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=gamma_streaming.log RUST_LOG=info cargo run --example gamma-streaming --features gamma,tracing
//! ```

use std::fs::File;

use futures::StreamExt as _;
use polymarket_client_sdk::gamma::{
    Client,
    types::request::{EventsRequest, MarketsRequest, TeamsRequest},
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

    // Stream 1: Active events
    // stream_active_events(&client).await?;

    // Stream 2: Markets
    stream_markets(&client).await?;

    // Stream 3: Teams
    // stream_teams(&client).await?;

    Ok(())
}

/// Streams all active events from the Gamma API.
async fn stream_active_events(client: &Client) -> anyhow::Result<()> {
    info!(stream = "events", "starting event stream");

    let mut stream = client
        .stream_data(
            |c, limit, offset| {
                let request = EventsRequest::builder()
                    .active(true)
                    .ascending(false)
                    .limit(limit)
                    .offset(offset)
                    .build();
                async move { c.events(&request).await }
            },
            1000, // page size
        )
        .boxed();

    let mut count = 0_u32;

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => {
                count += 1;

                // Log every 100th event to avoid flooding logs
                if count % 1000 == 1
                    && let Some(title) = &event.title
                {
                    info!(
                        stream = "events",
                        count = count,
                        id = %event.id,
                        title = %title,
                        active = ?event.active,
                        markets_count = event.markets.as_ref().map(std::vec::Vec::len)
                    );
                }
            }
            Err(e) => {
                warn!(stream = "events", error = %e, "stream error");
                break;
            }
        }
    }

    info!(stream = "events", total_events = count, "stream completed");

    Ok(())
}

/// Streams markets from the Gamma API.
async fn stream_markets(client: &Client) -> anyhow::Result<()> {
    info!(stream = "markets", "starting market stream");

    let mut stream = client
        .stream_data(
            |c, limit, offset| {
                let request = MarketsRequest::builder()
                    // .start_date_min(Utc::now() - TimeDelta::weeks(1))
                    // .end_date_max(Utc::now() + TimeDelta::days(1))
                    .ascending(false)
                    .closed(false)
                    .limit(limit)
                    .offset(offset)
                    .build();
                async move { c.markets(&request).await }
            },
            1000,
        )
        .boxed();

    let mut count = 0_u32;

    while let Some(result) = stream.next().await {
        match result {
            Ok(market) => {
                count += 1;

                // Log every 10th market to avoid flooding logs
                if count % 100 == 1
                    && let Some(question) = &market.question
                {
                    info!(
                        stream = "markets",
                        count = count,
                        id = %market.id,
                        nr = ?market.neg_risk_market_id,
                        clob = ?market.clob_token_ids,
                        question = %question,
                        active = ?market.active,
                        volume = ?market.volume_num
                    );
                }
            }
            Err(e) => {
                warn!(stream = "markets", error = %e, "stream error");
                break;
            }
        }
    }

    info!(
        stream = "markets",
        total_markets = count,
        "stream completed"
    );

    Ok(())
}

/// Streams teams from the Gamma API.
async fn stream_teams(client: &Client) -> anyhow::Result<()> {
    info!(stream = "teams", "starting team stream");

    let mut stream = client
        .stream_data(
            |c, limit, offset| {
                let request = TeamsRequest::builder()
                    .league(vec!["nfl".to_owned()])
                    .limit(limit)
                    .offset(offset)
                    .build();
                async move { c.teams(&request).await }
            },
            50, // page size
        )
        .boxed();

    let mut count = 0_u32;

    while let Some(result) = stream.next().await {
        match result {
            Ok(team) => {
                count += 1;

                // Log every 10th team to avoid flooding logs
                if count % 10 == 1
                    && let Some(name) = &team.name
                {
                    info!(
                        stream = "teams",
                        count = count,
                        id = team.id,
                        name = %name,
                        league = ?team.league,
                        abbreviation = ?team.abbreviation
                    );
                }
            }
            Err(e) => {
                warn!(stream = "teams", error = %e, "stream error");
                break;
            }
        }
    }

    info!(stream = "teams", total_teams = count, "stream completed");

    Ok(())
}
