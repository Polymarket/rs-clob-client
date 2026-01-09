//! Comprehensive CLOB API endpoint explorer (unauthenticated).
//!
//! This example dynamically tests all unauthenticated CLOB API endpoints by:
//! 1. Fetching markets to discover real token IDs and condition IDs
//! 2. Using those IDs for subsequent price, orderbook, and trade queries
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example unauthenticated --features tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=clob.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example unauthenticated --features tracing
//! ```

use std::collections::HashMap;
use std::fs::File;

use futures_util::StreamExt as _;
use polymarket_client_sdk::clob::types::Side;
use polymarket_client_sdk::clob::types::request::{
    LastTradePriceRequest, MidpointRequest, OrderBookSummaryRequest, PriceRequest, SpreadRequest,
};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::types::{B256, Decimal, U256};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

/// Finds a market with an active orderbook by streaming through all markets.
///
/// Returns a tuple of (`token_id`, `condition_id`) from a market that:
/// - Has orderbook enabled (`enable_order_book` = true)
/// - Is active and not closed
/// - Is accepting orders
/// - Has tokens with non-zero prices
///
/// This ensures subsequent price/midpoint/orderbook API calls will succeed.
async fn find_market_with_orderbook(client: &Client) -> anyhow::Result<(U256, B256)> {
    info!("Searching for a market with an active orderbook...");

    let mut stream = Box::pin(client.stream_data(Client::markets));

    while let Some(maybe_market) = stream.next().await {
        match maybe_market {
            Ok(market) => {
                if market.enable_order_book
                    && market.active
                    && !market.closed
                    && !market.archived
                    && market.accepting_orders
                    && !market.tokens.is_empty()
                    && market.tokens.iter().any(|t| t.price > Decimal::ZERO)
                {
                    let condition_id = market
                        .condition_id
                        .ok_or_else(|| anyhow::anyhow!("Market missing condition_id"))?;
                    let token_id = market
                        .tokens
                        .first()
                        .map(|t| t.token_id)
                        .ok_or_else(|| anyhow::anyhow!("Market has no tokens"))?;

                    let request = MidpointRequest::builder().token_id(token_id).build();
                    if client.midpoint(&request).await.is_ok() {
                        info!(
                            condition_id = %condition_id,
                            token_id = %token_id,
                            question = %market.question,
                            "Found market with active orderbook"
                        );

                        return Ok((token_id, condition_id));
                    }
                }
            }
            Err(e) => {
                error!(error = ?e, "Error fetching market");
            }
        }
    }

    Err(anyhow::anyhow!(
        "No active markets with orderbooks found after searching all markets"
    ))
}

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

    let client = Client::new("https://clob.polymarket.com", Config::default())?;

    // Health check endpoints
    match client.ok().await {
        Ok(_) => info!(endpoint = "ok", status = "healthy"),
        Err(e) => error!(endpoint = "ok", error = %e),
    }

    match client.server_time().await {
        Ok(time) => info!(endpoint = "server_time", timestamp = %time),
        Err(e) => error!(endpoint = "server_time", error = %e),
    }

    let (token_id, condition_id) = match find_market_with_orderbook(&client).await {
        Ok((tid, cid)) => (Some(tid), Some(cid)),
        Err(e) => {
            error!("Failed to find market with orderbook: {}", e);
            (None, None)
        }
    };

    if let Some(cid) = &condition_id {
        match client.market(&cid.to_string()).await {
            Ok(market) => info!(
                endpoint = "market",
                condition_id = %cid,
                question = %market.question,
                active = market.active
            ),
            Err(e) => error!(endpoint = "market", condition_id = %cid, error = %e),
        }
    }

    match client.sampling_markets(None).await {
        Ok(page) => info!(
            endpoint = "sampling_markets",
            count = page.data.len(),
            has_next = !page.next_cursor.is_empty()
        ),
        Err(e) => error!(endpoint = "sampling_markets", error = %e),
    }

    match client.simplified_markets(None).await {
        Ok(page) => info!(
            endpoint = "simplified_markets",
            count = page.data.len(),
            has_next = !page.next_cursor.is_empty()
        ),
        Err(e) => error!(endpoint = "simplified_markets", error = %e),
    }

    match client.sampling_simplified_markets(None).await {
        Ok(page) => info!(
            endpoint = "sampling_simplified_markets",
            count = page.data.len(),
            has_next = !page.next_cursor.is_empty()
        ),
        Err(e) => error!(endpoint = "sampling_simplified_markets", error = %e),
    }

    if let Some(token_id) = token_id {
        let midpoint_request = MidpointRequest::builder().token_id(token_id).build();
        match client.midpoint(&midpoint_request).await {
            Ok(midpoint) => info!(endpoint = "midpoint", token_id = %token_id, mid = %midpoint.mid),
            Err(e) => error!(endpoint = "midpoint", token_id = %token_id, error = %e),
        }

        match client.midpoints(&[midpoint_request]).await {
            Ok(midpoints) => info!(endpoint = "midpoints", count = midpoints.midpoints.len()),
            Err(e) => error!(endpoint = "midpoints", error = %e),
        }

        let buy_price_request = PriceRequest::builder()
            .token_id(token_id)
            .side(Side::Buy)
            .build();
        match client.price(&buy_price_request).await {
            Ok(price) => info!(
                endpoint = "price",
                token_id = %token_id,
                side = "buy",
                price = %price.price
            ),
            Err(e) => error!(endpoint = "price", token_id = %token_id, side = "buy", error = %e),
        }

        let sell_price_request = PriceRequest::builder()
            .token_id(token_id)
            .side(Side::Sell)
            .build();
        match client.price(&sell_price_request).await {
            Ok(price) => info!(
                endpoint = "price",
                token_id = %token_id,
                side = "sell",
                price = %price.price
            ),
            Err(e) => error!(endpoint = "price", token_id = %token_id, side = "sell", error = %e),
        }

        match client
            .prices(&[buy_price_request, sell_price_request])
            .await
        {
            Ok(prices) => info!(
                endpoint = "prices",
                count = prices.prices.as_ref().map_or(0, HashMap::len)
            ),
            Err(e) => error!(endpoint = "prices", error = %e),
        }

        let spread_request = SpreadRequest::builder().token_id(token_id).build();
        match client.spread(&spread_request).await {
            Ok(spread) => info!(
                endpoint = "spread",
                token_id = %token_id,
                spread = %spread.spread
            ),
            Err(e) => error!(endpoint = "spread", token_id = %token_id, error = %e),
        }

        match client.spreads(&[spread_request]).await {
            Ok(spreads) => info!(
                endpoint = "spreads",
                count = spreads.spreads.as_ref().map_or(0, HashMap::len)
            ),
            Err(e) => error!(endpoint = "spreads", error = %e),
        }

        match client.tick_size(token_id).await {
            Ok(tick_size) => info!(
                endpoint = "tick_size",
                token_id = %token_id,
                tick_size = %tick_size.minimum_tick_size
            ),
            Err(e) => error!(endpoint = "tick_size", token_id = %token_id, error = %e),
        }

        match client.neg_risk(token_id).await {
            Ok(neg_risk) => info!(
                endpoint = "neg_risk",
                token_id = %token_id,
                neg_risk = neg_risk.neg_risk
            ),
            Err(e) => error!(endpoint = "neg_risk", token_id = %token_id, error = %e),
        }

        match client.fee_rate_bps(token_id).await {
            Ok(fee_rate) => info!(
                endpoint = "fee_rate_bps",
                token_id = %token_id,
                base_fee = fee_rate.base_fee
            ),
            Err(e) => error!(endpoint = "fee_rate_bps", token_id = %token_id, error = %e),
        }

        let order_book_request = OrderBookSummaryRequest::builder()
            .token_id(token_id)
            .build();
        match client.order_book(&order_book_request).await {
            Ok(book) => {
                let hash = book.hash().unwrap_or_default();
                info!(
                    endpoint = "order_book",
                    token_id = %token_id,
                    bids = book.bids.len(),
                    asks = book.asks.len(),
                    hash = %hash
                );
            }
            Err(e) => error!(endpoint = "order_book", token_id = %token_id, error = %e),
        }

        match client.order_books(&[order_book_request]).await {
            Ok(books) => info!(endpoint = "order_books", count = books.len()),
            Err(e) => error!(endpoint = "order_books", error = %e),
        }

        let last_trade_request = LastTradePriceRequest::builder().token_id(token_id).build();
        match client.last_trade_price(&last_trade_request).await {
            Ok(last_trade) => info!(
                endpoint = "last_trade_price",
                token_id = %token_id,
                price = %last_trade.price
            ),
            Err(e) => error!(endpoint = "last_trade_price", token_id = %token_id, error = %e),
        }

        match client.last_trades_prices(&[last_trade_request]).await {
            Ok(prices) => info!(endpoint = "last_trade_prices", count = prices.len()),
            Err(e) => error!(endpoint = "last_trade_prices", error = %e),
        }
    } else {
        warn!(
            endpoint = "price_queries",
            "skipped - no token_id discovered"
        );
    }

    Ok(())
}
