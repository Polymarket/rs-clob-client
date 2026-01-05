//! Comprehensive RTDS (Real-Time Data Socket) endpoint explorer.
//!
//! This example dynamically tests all RTDS streaming endpoints by:
//! 1. Subscribing to Binance crypto prices (all symbols and filtered)
//! 2. Subscribing to Chainlink price feeds
//! 3. Subscribing to comment events
//! 4. Showing connection state and subscription count
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example rtds_crypto_prices --features rtds,tracing
//! ```

use std::time::Duration;

use futures::StreamExt as _;
use polymarket_client_sdk::rtds::Client;
use polymarket_client_sdk::rtds::types::response::CommentType;
use tokio::time::timeout;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    // Show connection state
    let state = client.connection_state();
    info!(endpoint = "connection_state", state = ?state);

    // Subscribe to all crypto prices from Binance
    info!(
        stream = "crypto_prices",
        "Subscribing to Binance prices (all symbols)"
    );
    match client.subscribe_crypto_prices(None) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut count = 0;

            while let Ok(Some(result)) = timeout(Duration::from_secs(5), stream.next()).await {
                match result {
                    Ok(price) => {
                        info!(
                            stream = "crypto_prices",
                            symbol = %price.symbol.to_uppercase(),
                            value = %price.value,
                            timestamp = %price.timestamp
                        );
                        count += 1;
                        if count >= 5 {
                            break;
                        }
                    }
                    Err(e) => debug!(stream = "crypto_prices", error = %e),
                }
            }
            info!(stream = "crypto_prices", received = count);
        }
        Err(e) => debug!(stream = "crypto_prices", error = %e),
    }

    // Subscribe to specific crypto symbols
    let symbols = vec!["btcusdt".to_owned(), "ethusdt".to_owned()];
    info!(
        stream = "crypto_prices_filtered",
        symbols = ?symbols,
        "Subscribing to specific symbols"
    );
    match client.subscribe_crypto_prices(Some(symbols.clone())) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut count = 0;

            while let Ok(Some(result)) = timeout(Duration::from_secs(5), stream.next()).await {
                match result {
                    Ok(price) => {
                        info!(
                            stream = "crypto_prices_filtered",
                            symbol = %price.symbol.to_uppercase(),
                            value = %price.value
                        );
                        count += 1;
                        if count >= 3 {
                            break;
                        }
                    }
                    Err(e) => debug!(stream = "crypto_prices_filtered", error = %e),
                }
            }
            info!(stream = "crypto_prices_filtered", received = count);
        }
        Err(e) => debug!(stream = "crypto_prices_filtered", error = %e),
    }

    // Subscribe to Chainlink price feeds
    info!(
        stream = "chainlink_prices",
        "Subscribing to Chainlink price feeds"
    );
    match client.subscribe_chainlink_prices(None) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut count = 0;

            while let Ok(Some(result)) = timeout(Duration::from_secs(5), stream.next()).await {
                match result {
                    Ok(price) => {
                        info!(
                            stream = "chainlink_prices",
                            symbol = %price.symbol,
                            value = %price.value,
                            timestamp = %price.timestamp
                        );
                        count += 1;
                        if count >= 3 {
                            break;
                        }
                    }
                    Err(e) => debug!(stream = "chainlink_prices", error = %e),
                }
            }
            info!(stream = "chainlink_prices", received = count);
        }
        Err(e) => debug!(stream = "chainlink_prices", error = %e),
    }

    // Subscribe to comments (unauthenticated)
    info!(stream = "comments", "Subscribing to comment events");
    match client.subscribe_comments(None) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut count = 0;

            // Comments may be infrequent, use shorter timeout
            while let Ok(Some(result)) = timeout(Duration::from_secs(3), stream.next()).await {
                match result {
                    Ok(comment) => {
                        info!(
                            stream = "comments",
                            id = %comment.id,
                            parent_type = ?comment.parent_entity_type,
                            parent_id = %comment.parent_entity_id
                        );
                        count += 1;
                        if count >= 3 {
                            break;
                        }
                    }
                    Err(e) => debug!(stream = "comments", error = %e),
                }
            }
            if count > 0 {
                info!(stream = "comments", received = count);
            } else {
                debug!(stream = "comments", "no comments received within timeout");
            }
        }
        Err(e) => debug!(stream = "comments", error = %e),
    }

    // Subscribe to specific comment type
    info!(
        stream = "comments_created",
        comment_type = ?CommentType::CommentCreated,
        "Subscribing to created comments only"
    );
    match client.subscribe_comments(Some(CommentType::CommentCreated)) {
        Ok(stream) => {
            let mut stream = Box::pin(stream);
            let mut count = 0;

            while let Ok(Some(result)) = timeout(Duration::from_secs(3), stream.next()).await {
                match result {
                    Ok(comment) => {
                        info!(
                            stream = "comments_created",
                            id = %comment.id,
                            parent_id = %comment.parent_entity_id
                        );
                        count += 1;
                        if count >= 2 {
                            break;
                        }
                    }
                    Err(e) => debug!(stream = "comments_created", error = %e),
                }
            }
            if count > 0 {
                info!(stream = "comments_created", received = count);
            } else {
                debug!(
                    stream = "comments_created",
                    "no created comments received within timeout"
                );
            }
        }
        Err(e) => debug!(stream = "comments_created", error = %e),
    }

    // Show final subscription count
    let sub_count = client.subscription_count();
    info!(endpoint = "subscription_count", count = sub_count);

    Ok(())
}
