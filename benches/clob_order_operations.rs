//! Benchmarks for CLOB order creation and signing operations
//!
//! This benchmark suite focuses on the hot path operations for creating and signing orders:
//! - Limit order building (price validation, decimal conversion, order struct creation)
//! - Order signing (EIP-712 domain construction and cryptographic signing)
//! - Order serialization (converting `SignedOrder` to JSON for API submission)

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::PrivateKeySigner;
use criterion::{Criterion, criterion_group, criterion_main};
use polymarket_client_sdk::POLYGON;
use polymarket_client_sdk::auth::Normal;
use polymarket_client_sdk::auth::state::Authenticated;
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::{OrderType, Side, TickSize};
use polymarket_client_sdk::types::{Decimal, U256};
use rust_decimal_macros::dec;

const TOKEN_ID: &str =
    "15871154585880608648532107628464183779895785213830018178010423617714102767076";

// Dummy private key for benchmarking (DO NOT USE IN PRODUCTION)
const BENCH_PRIVATE_KEY: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000001";

/// Helper to create an authenticated client with cached tick size and fee rate
async fn setup_client() -> (Client<Authenticated<Normal>>, PrivateKeySigner) {
    let token_id = U256::from_str(TOKEN_ID).expect("valid token ID");
    let signer = PrivateKeySigner::from_str(BENCH_PRIVATE_KEY)
        .expect("valid key")
        .with_chain_id(Some(POLYGON));

    let client = Client::default()
        .authentication_builder(&signer)
        .authenticate()
        .await
        .expect("authentication succeeds");

    // Pre-cache tick size and fee rate to avoid HTTP requests during benchmarking
    client.set_tick_size(token_id, TickSize::Hundredth);
    client.set_fee_rate_bps(token_id, 0);
    client.set_neg_risk(token_id, false);

    (client, signer)
}

/// Benchmark limit order building
fn bench_order_building(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let (client, _) = runtime.block_on(setup_client());
    let token_id = U256::from_str(TOKEN_ID).expect("valid token ID");

    let mut group = c.benchmark_group("clob_order_operations/order_building");

    group.bench_function("BUY", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let order_builder = client
                    .limit_order()
                    .order_type(OrderType::GTC)
                    .token_id(token_id)
                    .side(Side::Buy)
                    .price(dec!(0.50))
                    .size(Decimal::ONE_HUNDRED);

                std::hint::black_box(order_builder.build().await.expect("build succeeds"))
            })
        });
    });

    group.bench_function("SELL", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let order_builder = client
                    .limit_order()
                    .order_type(OrderType::GTC)
                    .token_id(token_id)
                    .side(Side::Sell)
                    .price(dec!(0.50))
                    .size(Decimal::ONE_HUNDRED);

                std::hint::black_box(order_builder.build().await.expect("build succeeds"))
            })
        });
    });

    group.finish();
}

/// Benchmark order signing
fn bench_order_signing(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let (client, signer) = runtime.block_on(setup_client());
    let token_id = U256::from_str(TOKEN_ID).expect("valid token ID");

    let mut group = c.benchmark_group("clob_order_operations/order_signing");

    let signable_order = runtime.block_on(async {
        client
            .limit_order()
            .token_id(token_id)
            .side(Side::Buy)
            .price(dec!(0.50))
            .size(dec!(100.0))
            .build()
            .await
            .expect("build succeeds")
    });

    group.bench_function("limit_order", |b| {
        b.iter(|| {
            runtime.block_on(async {
                std::hint::black_box(
                    client
                        .sign(&signer, signable_order.clone())
                        .await
                        .expect("sign succeeds"),
                )
            })
        });
    });

    group.finish();
}

/// Benchmark order serialization
fn bench_order_serializing(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let token_id = U256::from_str(TOKEN_ID).expect("valid token ID");

    let mut group = c.benchmark_group("clob_order_operations/order_serializing");

    let signed_order = runtime.block_on(async {
        let (client, signer) = setup_client().await;

        let signable = client
            .limit_order()
            .token_id(token_id)
            .side(Side::Buy)
            .price(dec!(0.50))
            .size(dec!(100.0))
            .build()
            .await
            .expect("build succeeds");

        client.sign(&signer, signable).await.expect("sign succeeds")
    });

    group.bench_function("to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(std::hint::black_box(&signed_order))
                .expect("serialization succeeds");
            std::hint::black_box(json)
        });
    });

    group.finish();
}

criterion_group!(
    order_operations_benches,
    bench_order_building,
    bench_order_signing,
    bench_order_serializing,
);

criterion_main!(order_operations_benches);
