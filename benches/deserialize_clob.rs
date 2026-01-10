/// Comprehensive benchmarks for CLOB API deserialization.
///
/// This module benchmarks ALL deserialization types for the Central Limit Order Book API,
/// with special focus on hot trading paths: order placement, orderbook updates, trades,
/// and cancellations.
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use polymarket_client_sdk::clob::types::response::{
    ApiKeysResponse, BalanceAllowanceResponse, BanStatusResponse, CancelOrdersResponse,
    FeeRateResponse, LastTradePriceResponse, MarketResponse, MidpointResponse, NegRiskResponse,
    NotificationResponse, OpenOrderResponse, OrderBookSummaryResponse, PostOrderResponse,
    PriceHistoryResponse, PriceResponse, SpreadResponse, TickSizeResponse, TradeResponse,
};

fn bench_orderbook(c: &mut Criterion) {
    let mut group = c.benchmark_group("clob/orderbook");

    let orderbook_small = r#"{
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "timestamp": "1234567890123",
        "bids": [{"price": "0.55", "size": "100.0"}],
        "asks": [{"price": "0.56", "size": "150.0"}],
        "min_order_size": "10.0",
        "neg_risk": false,
        "tick_size": "0.01"
    }"#;

    let orderbook_medium = r#"{
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "timestamp": "1234567890123",
        "hash": "abc123def456",
        "bids": [
            {"price": "0.55", "size": "100.0"},
            {"price": "0.54", "size": "200.0"},
            {"price": "0.53", "size": "300.0"},
            {"price": "0.52", "size": "400.0"},
            {"price": "0.51", "size": "500.0"},
            {"price": "0.50", "size": "600.0"},
            {"price": "0.49", "size": "700.0"},
            {"price": "0.48", "size": "800.0"},
            {"price": "0.47", "size": "900.0"},
            {"price": "0.46", "size": "1000.0"}
        ],
        "asks": [
            {"price": "0.56", "size": "150.0"},
            {"price": "0.57", "size": "175.0"},
            {"price": "0.58", "size": "200.0"},
            {"price": "0.59", "size": "225.0"},
            {"price": "0.60", "size": "250.0"},
            {"price": "0.61", "size": "275.0"},
            {"price": "0.62", "size": "300.0"},
            {"price": "0.63", "size": "325.0"},
            {"price": "0.64", "size": "350.0"},
            {"price": "0.65", "size": "375.0"}
        ],
        "min_order_size": "10.0",
        "neg_risk": false,
        "tick_size": "0.01"
    }"#;

    // Benchmark with different orderbook depths
    for (name, json) in [
        ("small_1_level", orderbook_small),
        ("medium_10_levels", orderbook_medium),
    ] {
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("OrderBookSummaryResponse", name),
            &json,
            |b, json| {
                b.iter(|| {
                    let _: OrderBookSummaryResponse =
                        serde_json::from_str(std::hint::black_box(json))
                            .expect("Deserialization should succeed");
                });
            },
        );
    }

    group.finish();
}

fn bench_orders(c: &mut Criterion) {
    let mut group = c.benchmark_group("clob/orders");

    let post_order = r#"{
        "makingAmount": "100.5",
        "takingAmount": "55.275",
        "orderID": "0x1234567890abcdef",
        "status": "LIVE",
        "success": true,
        "transactionsHashes": ["0x0000000000000000000000000000000000000000000000000000000000000001"],
        "trade_ids": ["trade_123", "trade_456"]
    }"#;
    group.throughput(Throughput::Bytes(post_order.len() as u64));
    group.bench_function("PostOrderResponse", |b| {
        b.iter(|| {
            let _: PostOrderResponse = serde_json::from_str(std::hint::black_box(post_order))
                .expect("Deserialization should succeed");
        });
    });

    let open_order = r#"{
        "id": "0x1234567890abcdef",
        "status": "LIVE",
        "owner": "550e8400-e29b-41d4-a716-446655440000",
        "maker_address": "0x1234567890123456789012345678901234567890",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "side": "BUY",
        "original_size": "100.0",
        "size_matched": "25.0",
        "price": "0.55",
        "associate_trades": ["trade_123"],
        "outcome": "Yes",
        "created_at": 1234567890,
        "expiration": "1234567890",
        "order_type": "GTC"
    }"#;
    group.throughput(Throughput::Bytes(open_order.len() as u64));
    group.bench_function("OpenOrderResponse", |b| {
        b.iter(|| {
            let _: OpenOrderResponse = serde_json::from_str(std::hint::black_box(open_order))
                .expect("Deserialization should succeed");
        });
    });

    let trade = r#"{
        "id": "trade_123",
        "taker_order_id": "0xabcdef1234567890",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "side": "BUY",
        "size": "25.0",
        "fee_rate_bps": "25",
        "price": "0.55",
        "status": "MATCHED",
        "match_time": "1234567890",
        "last_update": "1234567891",
        "outcome": "Yes",
        "bucket_index": 5,
        "owner": "550e8400-e29b-41d4-a716-446655440000",
        "maker_address": "0x1234567890123456789012345678901234567890",
        "maker_orders": [
            {
                "order_id": "0x111",
                "owner": "550e8400-e29b-41d4-a716-446655440000",
                "maker_address": "0x1234567890123456789012345678901234567890",
                "matched_amount": "0.2",
                "price": "0.55",
                "fee_rate_bps": "1",
                "asset_id": "123456789",
                "outcome": "Yes",
                "side": "BUY"
            },
            {
                "order_id": "0x222",
                "owner": "550e8400-e29b-41d4-a716-446655440000",
                "maker_address": "0x1234567890123456789012345678901234567890",
                "matched_amount": "0.2",
                "price": "0.55",
                "fee_rate_bps": "1",
                "asset_id": "123456789",
                "outcome": "Yes",
                "side": "BUY"
            }
        ],
        "transaction_hash": "0x0000000000000000000000000000000000000000000000000000000000000abc",
        "trader_side": "TAKER"
    }"#;
    group.throughput(Throughput::Bytes(trade.len() as u64));
    group.bench_function("TradeResponse", |b| {
        b.iter(|| {
            let _: TradeResponse = serde_json::from_str(std::hint::black_box(trade))
                .expect("Deserialization should succeed");
        });
    });

    let cancel = r#"{
        "canceled": ["0x123", "0x456", "0x789"],
        "notCanceled": {
            "0xabc": "Order already filled",
            "0xdef": "Order not found"
        }
    }"#;
    group.throughput(Throughput::Bytes(cancel.len() as u64));
    group.bench_function("CancelOrdersResponse", |b| {
        b.iter(|| {
            let _: CancelOrdersResponse = serde_json::from_str(std::hint::black_box(cancel))
                .expect("Deserialization should succeed");
        });
    });

    group.finish();
}

fn bench_market_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("clob/market_data");

    let market = r#"{
        "enable_order_book": true,
        "active": true,
        "closed": false,
        "archived": false,
        "accepting_orders": true,
        "accepting_order_timestamp": null,
        "minimum_order_size": "1.0",
        "minimum_tick_size": "0.01",
        "condition_id": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "question_id": "0x0000000000000000000000000000000000000000000000000000000000000002",
        "question": "Will X happen?",
        "description": "Test market for benchmarking",
        "market_slug": "test-market-2024",
        "end_date_iso": "2024-12-31T23:59:59Z",
        "game_start_time": null,
        "seconds_delay": 0,
        "fpmm": "0x1234567890123456789012345678901234567890",
        "maker_base_fee": "0.001",
        "taker_base_fee": "0.002",
        "notifications_enabled": true,
        "neg_risk": false,
        "neg_risk_market_id": "",
        "neg_risk_request_id": "",
        "icon": "https://polymarket.com/icon.png",
        "image": "https://polymarket.com/image.png",
        "rewards": {"rates": [], "min_size": "0", "max_spread": "0"},
        "is_50_50_outcome": true,
        "tokens": [
            {"token_id": "123456789", "outcome": "Yes", "price": "0.55", "winner": false},
            {"token_id": "987654321", "outcome": "No", "price": "0.45", "winner": false}
        ],
        "tags": ["politics", "2024"]
    }"#;
    group.throughput(Throughput::Bytes(market.len() as u64));
    group.bench_function("MarketResponse", |b| {
        b.iter(|| {
            let _: MarketResponse = serde_json::from_str(std::hint::black_box(market))
                .expect("Deserialization should succeed");
        });
    });

    group.finish();
}

fn bench_pricing(c: &mut Criterion) {
    let mut group = c.benchmark_group("clob/pricing");

    let midpoint = r#"{"mid": "0.55"}"#;
    group.bench_function("MidpointResponse", |b| {
        b.iter(|| {
            let _: MidpointResponse = serde_json::from_str(std::hint::black_box(midpoint))
                .expect("Deserialization should succeed");
        });
    });

    let price = r#"{"price": "0.60"}"#;
    group.bench_function("PriceResponse", |b| {
        b.iter(|| {
            let _: PriceResponse = serde_json::from_str(std::hint::black_box(price))
                .expect("Deserialization should succeed");
        });
    });

    let spread = r#"{"spread": "0.05"}"#;
    group.bench_function("SpreadResponse", |b| {
        b.iter(|| {
            let _: SpreadResponse = serde_json::from_str(std::hint::black_box(spread))
                .expect("Deserialization should succeed");
        });
    });

    let tick_size = r#"{"minimum_tick_size": "0.01"}"#;
    group.bench_function("TickSizeResponse", |b| {
        b.iter(|| {
            let _: TickSizeResponse = serde_json::from_str(std::hint::black_box(tick_size))
                .expect("Deserialization should succeed");
        });
    });

    let neg_risk = r#"{"neg_risk": false}"#;
    group.bench_function("NegRiskResponse", |b| {
        b.iter(|| {
            let _: NegRiskResponse = serde_json::from_str(std::hint::black_box(neg_risk))
                .expect("Deserialization should succeed");
        });
    });

    let fee_rate = r#"{"base_fee": 25}"#;
    group.bench_function("FeeRateResponse", |b| {
        b.iter(|| {
            let _: FeeRateResponse = serde_json::from_str(std::hint::black_box(fee_rate))
                .expect("Deserialization should succeed");
        });
    });

    let last_trade_price = r#"{"price": "0.55", "side": "BUY"}"#;
    group.bench_function("LastTradePriceResponse", |b| {
        b.iter(|| {
            let _: LastTradePriceResponse =
                serde_json::from_str(std::hint::black_box(last_trade_price))
                    .expect("Deserialization should succeed");
        });
    });

    group.finish();
}

fn bench_account_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("clob/account");

    let balance = r#"{"balance": "1000.50", "allowance": "500.25"}"#;
    group.bench_function("BalanceAllowanceResponse", |b| {
        b.iter(|| {
            let _: BalanceAllowanceResponse = serde_json::from_str(std::hint::black_box(balance))
                .expect("Deserialization should succeed");
        });
    });

    let api_keys = r#"{"api_keys": ["key1", "key2", "key3"]}"#;
    group.bench_function("ApiKeysResponse", |b| {
        b.iter(|| {
            let _: ApiKeysResponse = serde_json::from_str(std::hint::black_box(api_keys))
                .expect("Deserialization should succeed");
        });
    });

    let ban_status = r#"{
        "closed_only": true
    }"#;
    group.bench_function("BanStatusResponse", |b| {
        b.iter(|| {
            let _: BanStatusResponse = serde_json::from_str(std::hint::black_box(ban_status))
                .expect("Deserialization should succeed");
        });
    });

    group.finish();
}

fn bench_additional_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("clob/additional");

    let notification = r#"{
        "type": 1,
        "owner": "550e8400-e29b-41d4-a716-446655440000",
        "payload": {
            "asset_id": "123456789",
            "condition_id": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "eventSlug": "test-event",
            "icon": "https://polymarket.com/icon.png",
            "image": "https://polymarket.com/image.png",
            "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "market_slug": "test-market",
            "matched_size": "25.0",
            "order_id": "0x123",
            "original_size": "100.0",
            "outcome": "Yes",
            "outcome_index": 0,
            "owner": "550e8400-e29b-41d4-a716-446655440000",
            "price": "0.55",
            "question": "Will X happen?",
            "remaining_size": "75.0",
            "seriesSlug": "test-series",
            "side": "BUY",
            "trade_id": "trade_123",
            "transaction_hash": "0x0000000000000000000000000000000000000000000000000000000000000abc",
            "type": "GTC"
        }
    }"#;
    group.bench_function("NotificationResponse", |b| {
        b.iter(|| {
            let _: NotificationResponse = serde_json::from_str(std::hint::black_box(notification))
                .expect("Deserialization should succeed");
        });
    });

    let price_history = r#"{
        "history": [
            {"t": 1234567890000, "p": "0.55"},
            {"t": 1234567891000, "p": "0.56"},
            {"t": 1234567892000, "p": "0.54"},
            {"t": 1234567893000, "p": "0.57"},
            {"t": 1234567894000, "p": "0.55"}
        ]
    }"#;
    group.bench_function("PriceHistoryResponse", |b| {
        b.iter(|| {
            let _: PriceHistoryResponse = serde_json::from_str(std::hint::black_box(price_history))
                .expect("Deserialization should succeed");
        });
    });

    group.finish();
}

criterion_group!(
    clob_benches,
    bench_orderbook,
    bench_orders,
    bench_market_data,
    bench_pricing,
    bench_account_data,
    bench_additional_types
);
criterion_main!(clob_benches);
