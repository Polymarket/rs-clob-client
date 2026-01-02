//! Comprehensive CLOB API unauthenticated endpoint explorer.
//!
//! This example dynamically tests all unauthenticated CLOB API endpoints by:
//! 1. Using Gamma API to find active markets with volume (ensures orderbooks exist)
//! 2. Extracting token IDs from those markets
//! 3. Using those IDs for CLOB price, orderbook, and spread queries
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info cargo run --example unauthenticated --features gamma,tracing
//! ```

use std::collections::HashMap;

use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::request::{
    LastTradePriceRequest, MidpointRequest, OrderBookSummaryRequest, PriceHistoryRequest,
    PriceRequest, SpreadRequest,
};
use polymarket_client_sdk::clob::types::{Interval, Side};
use polymarket_client_sdk::gamma::Client as GammaClient;
use polymarket_client_sdk::gamma::types::request::MarketsRequest;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();
    let gamma = GammaClient::default();

    // Health check
    match client.ok().await {
        Ok(s) => info!(endpoint = "ok", result = %s),
        Err(e) => debug!(endpoint = "ok", error = %e),
    }

    // Server time
    match client.server_time().await {
        Ok(t) => info!(endpoint = "server_time", timestamp = %t),
        Err(e) => debug!(endpoint = "server_time", error = %e),
    }

    // Geoblock check
    match client.check_geoblock().await {
        Ok(g) => info!(
            endpoint = "check_geoblock",
            blocked = g.blocked,
            country = %g.country,
            region = %g.region
        ),
        Err(e) => debug!(endpoint = "check_geoblock", error = %e),
    }

    // Use Gamma API to get active markets sorted by volume (high-volume markets have orderbooks)
    let gamma_markets = gamma
        .markets(
            &MarketsRequest::builder()
                .closed(false)
                .limit(20)
                .order("volume".to_owned())
                .ascending(false)
                .build(),
        )
        .await;

    // Extract condition_id and token_ids from active Gamma markets
    let (condition_id, token_ids): (Option<String>, Vec<String>) = match &gamma_markets {
        Ok(markets) => {
            info!(endpoint = "gamma_markets", count = markets.len());
            // Find a market with condition_id and clob_token_ids
            let market_with_tokens = markets.iter().find(|m| {
                m.condition_id.is_some() && m.clob_token_ids.as_ref().is_some_and(|t| !t.is_empty())
            });

            if let Some(m) = market_with_tokens {
                let cid = m.condition_id.clone();
                let tids = m.clob_token_ids.clone().unwrap_or_default();
                if let Some(c) = &cid {
                    info!(
                        condition_id = %c,
                        token_count = tids.len(),
                        "Found active market with tokens"
                    );
                }
                (cid, tids)
            } else {
                debug!("No active markets with clob_token_ids found");
                (None, Vec::new())
            }
        }
        Err(e) => {
            debug!(endpoint = "gamma_markets", error = %e);
            (None, Vec::new())
        }
    };

    // Fetch CLOB markets
    match client.markets(None).await {
        Ok(page) => info!(endpoint = "markets", count = page.data.len()),
        Err(e) => debug!(endpoint = "markets", error = %e),
    }

    // Additional market endpoints
    match client.sampling_markets(None).await {
        Ok(page) => info!(endpoint = "sampling_markets", count = page.data.len()),
        Err(e) => debug!(endpoint = "sampling_markets", error = %e),
    }

    match client.simplified_markets(None).await {
        Ok(page) => info!(endpoint = "simplified_markets", count = page.data.len()),
        Err(e) => debug!(endpoint = "simplified_markets", error = %e),
    }

    match client.sampling_simplified_markets(None).await {
        Ok(page) => info!(
            endpoint = "sampling_simplified_markets",
            count = page.data.len()
        ),
        Err(e) => debug!(endpoint = "sampling_simplified_markets", error = %e),
    }

    // Single market lookup (if we have a condition ID)
    if let Some(cid) = &condition_id {
        match client.market(cid).await {
            Ok(_) => info!(endpoint = "market", condition_id = %cid),
            Err(e) => debug!(endpoint = "market", condition_id = %cid, error = %e),
        }

        // Price history
        match client
            .price_history(
                &PriceHistoryRequest::builder()
                    .market(cid)
                    .time_range(Interval::OneDay)
                    .fidelity(60)
                    .build(),
            )
            .await
        {
            Ok(h) => {
                info!(endpoint = "price_history", condition_id = %cid, count = h.history.len());
            }
            Err(e) => debug!(endpoint = "price_history", condition_id = %cid, error = %e),
        }
    }

    // All prices (returns all token prices)
    match client.all_prices().await {
        Ok(prices) => {
            let count = prices.prices.as_ref().map_or(0, HashMap::len);
            info!(endpoint = "all_prices", count = count);
        }
        Err(e) => debug!(endpoint = "all_prices", error = %e),
    }

    // Token-based queries (if we have token IDs from active Gamma markets)
    if token_ids.is_empty() {
        debug!("skipping token-based queries - no token IDs from active markets");
    } else {
        let token_id = &token_ids[0];
        info!(token_id = %token_id, "Using token from active Gamma market");

        // Midpoint
        let midpoint_request = MidpointRequest::builder().token_id(token_id).build();
        match client.midpoint(&midpoint_request).await {
            Ok(m) => info!(endpoint = "midpoint", token_id = %token_id, mid = ?m.mid),
            Err(e) => debug!(endpoint = "midpoint", token_id = %token_id, error = %e),
        }

        // Midpoints (batch)
        let midpoint_requests: Vec<MidpointRequest> = token_ids
            .iter()
            .map(|tid| MidpointRequest::builder().token_id(tid).build())
            .collect();
        match client.midpoints(&midpoint_requests).await {
            Ok(m) => info!(endpoint = "midpoints", count = m.midpoints.len()),
            Err(e) => debug!(endpoint = "midpoints", error = %e),
        }

        // Price (buy and sell)
        for side in [Side::Buy, Side::Sell] {
            let price_request = PriceRequest::builder()
                .token_id(token_id)
                .side(side)
                .build();
            match client.price(&price_request).await {
                Ok(p) => {
                    info!(endpoint = "price", token_id = %token_id, side = ?side, price = %p.price);
                }
                Err(e) => {
                    debug!(endpoint = "price", token_id = %token_id, side = ?side, error = %e);
                }
            }
        }

        // Prices (batch)
        let price_requests: Vec<PriceRequest> = token_ids
            .iter()
            .map(|tid| {
                PriceRequest::builder()
                    .token_id(tid)
                    .side(Side::Buy)
                    .build()
            })
            .collect();
        match client.prices(&price_requests).await {
            Ok(p) => {
                let count = p.prices.as_ref().map_or(0, HashMap::len);
                info!(endpoint = "prices", count = count);
            }
            Err(e) => debug!(endpoint = "prices", error = %e),
        }

        // Spread
        let spread_request = SpreadRequest::builder().token_id(token_id).build();
        match client.spread(&spread_request).await {
            Ok(s) => info!(endpoint = "spread", token_id = %token_id, spread = %s.spread),
            Err(e) => debug!(endpoint = "spread", token_id = %token_id, error = %e),
        }

        // Spreads (batch)
        let spread_requests: Vec<SpreadRequest> = token_ids
            .iter()
            .map(|tid| SpreadRequest::builder().token_id(tid).build())
            .collect();
        match client.spreads(&spread_requests).await {
            Ok(s) => {
                let count = s.spreads.as_ref().map_or(0, HashMap::len);
                info!(endpoint = "spreads", count = count);
            }
            Err(e) => debug!(endpoint = "spreads", error = %e),
        }

        // Tick size
        match client.tick_size(token_id).await {
            Ok(t) => {
                info!(endpoint = "tick_size", token_id = %token_id, tick_size = %t.minimum_tick_size);
            }
            Err(e) => debug!(endpoint = "tick_size", token_id = %token_id, error = %e),
        }

        // Neg risk
        match client.neg_risk(token_id).await {
            Ok(n) => info!(endpoint = "neg_risk", token_id = %token_id, neg_risk = n.neg_risk),
            Err(e) => debug!(endpoint = "neg_risk", token_id = %token_id, error = %e),
        }

        // Fee rate
        match client.fee_rate_bps(token_id).await {
            Ok(f) => info!(endpoint = "fee_rate_bps", token_id = %token_id, base_fee = f.base_fee),
            Err(e) => debug!(endpoint = "fee_rate_bps", token_id = %token_id, error = %e),
        }

        // Order book
        let order_book_request = OrderBookSummaryRequest::builder()
            .token_id(token_id)
            .build();
        match client.order_book(&order_book_request).await {
            Ok(book) => {
                let hash = book.hash().unwrap_or_else(|_| "error".to_owned());
                info!(
                    endpoint = "order_book",
                    token_id = %token_id,
                    bids = book.bids.len(),
                    asks = book.asks.len(),
                    hash = %hash
                );
            }
            Err(e) => debug!(endpoint = "order_book", token_id = %token_id, error = %e),
        }

        // Order books (batch)
        let order_book_requests: Vec<OrderBookSummaryRequest> = token_ids
            .iter()
            .map(|tid| OrderBookSummaryRequest::builder().token_id(tid).build())
            .collect();
        match client.order_books(&order_book_requests).await {
            Ok(books) => info!(endpoint = "order_books", count = books.len()),
            Err(e) => debug!(endpoint = "order_books", error = %e),
        }

        // Last trade price
        let last_trade_request = LastTradePriceRequest::builder().token_id(token_id).build();
        match client.last_trade_price(&last_trade_request).await {
            Ok(p) => info!(endpoint = "last_trade_price", token_id = %token_id, price = %p.price),
            Err(e) => debug!(endpoint = "last_trade_price", token_id = %token_id, error = %e),
        }

        // Last trade prices (batch)
        let last_trade_requests: Vec<LastTradePriceRequest> = token_ids
            .iter()
            .map(|tid| LastTradePriceRequest::builder().token_id(tid).build())
            .collect();
        match client.last_trades_prices(&last_trade_requests).await {
            Ok(p) => info!(endpoint = "last_trades_prices", count = p.len()),
            Err(e) => debug!(endpoint = "last_trades_prices", error = %e),
        }
    }

    Ok(())
}
