//! Comprehensive authenticated CLOB API endpoint explorer.
//!
//! This example tests authenticated CLOB API endpoints including:
//! 1. API key management and account status
//! 2. Market and limit order creation
//! 3. Order management (fetch, cancel)
//! 4. Balance and allowance operations
//! 5. Trades and rewards queries
//!
//! Run with tracing enabled:
//! ```sh
//! RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example authenticated --features clob,tracing
//! ```
//!
//! Optionally log to a file:
//! ```sh
//! LOG_FILE=authenticated.log RUST_LOG=info,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example authenticated --features clob,tracing
//! ```
//!
//! Requires `POLY_PRIVATE_KEY` environment variable to be set.

use std::fs::File;
use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use chrono::{TimeDelta, Utc};
use polymarket_client_sdk::clob::types::request::{
    BalanceAllowanceRequest, OrdersRequest, TradesRequest, UpdateBalanceAllowanceRequest,
    UserRewardsEarningRequest,
};
use polymarket_client_sdk::clob::types::{Amount, OrderType, Side};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::types::{Decimal, U256};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use rust_decimal_macros::dec;
use tracing::{error, info};
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

    let token_id = U256::from_str(
        "15871154585880608648532107628464183779895785213830018178010423617714102767076",
    )?;

    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need POLY_PRIVATE_KEY");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let config = Config::builder().use_server_time(true).build();
    let client = Client::new("https://clob.polymarket.com", config)?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    match client.api_keys().await {
        Ok(keys) => info!(endpoint = "api_keys", result = ?keys),
        Err(e) => error!(endpoint = "api_keys", error = %e),
    }

    match client.closed_only_mode().await {
        Ok(status) => info!(
            endpoint = "closed_only_mode",
            closed_only = status.closed_only
        ),
        Err(e) => error!(endpoint = "closed_only_mode", error = %e),
    }

    // Market order
    let market_order = client
        .market_order()
        .token_id(token_id)
        .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
        .side(Side::Buy)
        .build()
        .await?;
    let signed_order = client.sign(&signer, market_order).await?;
    match client.post_order(signed_order).await {
        Ok(r) => {
            info!(endpoint = "post_order", order_type = "market", order_id = %r.order_id, success = r.success);
        }
        Err(e) => error!(endpoint = "post_order", order_type = "market", error = %e),
    }

    // Limit order
    let limit_order = client
        .limit_order()
        .token_id(token_id)
        .order_type(OrderType::GTD)
        .expiration(Utc::now() + TimeDelta::days(2))
        .price(dec!(0.5))
        .size(Decimal::ONE_HUNDRED)
        .side(Side::Buy)
        .build()
        .await?;
    let signed_order = client.sign(&signer, limit_order).await?;
    match client.post_order(signed_order).await {
        Ok(r) => {
            info!(endpoint = "post_order", order_type = "limit", order_id = %r.order_id, success = r.success);
        }
        Err(e) => error!(endpoint = "post_order", order_type = "limit", error = %e),
    }

    match client.notifications().await {
        Ok(n) => info!(endpoint = "notifications", count = n.len()),
        Err(e) => error!(endpoint = "notifications", error = %e),
    }

    match client
        .balance_allowance(BalanceAllowanceRequest::default())
        .await
    {
        Ok(b) => info!(endpoint = "balance_allowance", result = ?b),
        Err(e) => error!(endpoint = "balance_allowance", error = %e),
    }

    match client
        .update_balance_allowance(UpdateBalanceAllowanceRequest::default())
        .await
    {
        Ok(b) => info!(endpoint = "update_balance_allowance", result = ?b),
        Err(e) => error!(endpoint = "update_balance_allowance", error = %e),
    }

    let order_id = "0xa1449ec0831c7d62f887c4653d0917f2445783ff30f0ca713d99c667fef17f2c";
    match client.order(order_id).await {
        Ok(o) => info!(endpoint = "order", order_id = %order_id, status = ?o.status),
        Err(e) => error!(endpoint = "order", order_id = %order_id, error = %e),
    }

    match client.orders(&OrdersRequest::default(), None).await {
        Ok(orders) => info!(endpoint = "orders", count = orders.data.len()),
        Err(e) => error!(endpoint = "orders", error = %e),
    }

    match client.cancel_order(order_id).await {
        Ok(r) => info!(endpoint = "cancel_order", order_id = %order_id, result = ?r),
        Err(e) => error!(endpoint = "cancel_order", order_id = %order_id, error = %e),
    }

    match client.cancel_orders(&[order_id]).await {
        Ok(r) => info!(endpoint = "cancel_orders", result = ?r),
        Err(e) => error!(endpoint = "cancel_orders", error = %e),
    }

    match client.cancel_all_orders().await {
        Ok(r) => info!(endpoint = "cancel_all_orders", result = ?r),
        Err(e) => error!(endpoint = "cancel_all_orders", error = %e),
    }

    match client.orders(&OrdersRequest::default(), None).await {
        Ok(orders) => info!(
            endpoint = "orders",
            after_cancel = true,
            count = orders.data.len()
        ),
        Err(e) => error!(endpoint = "orders", after_cancel = true, error = %e),
    }

    match client.trades(&TradesRequest::default(), None).await {
        Ok(trades) => info!(endpoint = "trades", count = trades.data.len()),
        Err(e) => error!(endpoint = "trades", error = %e),
    }

    match client
        .earnings_for_user_for_day(Utc::now().date_naive(), None)
        .await
    {
        Ok(e) => info!(endpoint = "earnings_for_user_for_day", result = ?e),
        Err(e) => error!(endpoint = "earnings_for_user_for_day", error = %e),
    }

    let request = UserRewardsEarningRequest::builder()
        .date(Utc::now().date_naive() - TimeDelta::days(30))
        .build();
    match client
        .user_earnings_and_markets_config(&request, None)
        .await
    {
        Ok(e) => info!(endpoint = "user_earnings_and_markets_config", result = ?e),
        Err(e) => error!(endpoint = "user_earnings_and_markets_config", error = %e),
    }

    match client.reward_percentages().await {
        Ok(r) => info!(endpoint = "reward_percentages", result = ?r),
        Err(e) => error!(endpoint = "reward_percentages", error = %e),
    }

    match client.current_rewards(None).await {
        Ok(r) => info!(endpoint = "current_rewards", result = ?r),
        Err(e) => error!(endpoint = "current_rewards", error = %e),
    }

    let market_id = "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1";
    match client.raw_rewards_for_market(market_id, None).await {
        Ok(r) => info!(endpoint = "raw_rewards_for_market", market_id = %market_id, result = ?r),
        Err(e) => error!(endpoint = "raw_rewards_for_market", market_id = %market_id, error = %e),
    }

    Ok(())
}
