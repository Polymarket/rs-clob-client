/// Comprehensive benchmarks for CLOB WebSocket message deserialization.
///
/// This module benchmarks ALL WebSocket message types with special focus on the MOST CRITICAL
/// hot paths for live trading: orderbook updates, trade notifications, and order status updates.
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use polymarket_client_sdk::clob::ws::types::response::OrderBookLevel;
use polymarket_client_sdk::clob::ws::{
    BestBidAsk, BookUpdate, LastTradePrice, MakerOrder, MarketResolved, MidpointUpdate, NewMarket,
    OrderMessage, PriceChange, TickSizeChange, TradeMessage, WsMessage,
};

fn bench_ws_message(c: &mut Criterion) {
    let mut group = c.benchmark_group("websocket/ws_message");

    let book_msg = r#"{
        "event_type": "book",
        "asset_id": "123456789",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "bids": [{"price": "0.55", "size": "100.0"}],
        "asks": [{"price": "0.56", "size": "150.0"}]
    }"#;
    group.throughput(Throughput::Bytes(book_msg.len() as u64));
    group.bench_function("WsMessage::Book", |b| {
        b.iter(|| {
            let _: WsMessage = serde_json::from_str(std::hint::black_box(book_msg))
                .expect("Deserialization should succeed");
        });
    });

    let price_change_msg = r#"{
        "event_type": "price_change",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "price_changes": [{
            "asset_id": "123456789",
            "price": "0.65",
            "side": "BUY"
        }]
    }"#;
    group.throughput(Throughput::Bytes(price_change_msg.len() as u64));
    group.bench_function("WsMessage::PriceChange", |b| {
        b.iter(|| {
            let _: WsMessage = serde_json::from_str(std::hint::black_box(price_change_msg))
                .expect("Deserialization should succeed");
        });
    });

    let trade_msg = r#"{
        "event_type": "trade",
        "id": "trade_123",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "side": "BUY",
        "size": "25.0",
        "price": "0.55",
        "status": "MATCHED",
        "maker_orders": []
    }"#;
    group.throughput(Throughput::Bytes(trade_msg.len() as u64));
    group.bench_function("WsMessage::Trade", |b| {
        b.iter(|| {
            let _: WsMessage = serde_json::from_str(std::hint::black_box(trade_msg))
                .expect("Deserialization should succeed");
        });
    });

    let order_msg = r#"{
        "event_type": "order",
        "id": "0x123",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "side": "BUY",
        "price": "0.55"
    }"#;
    group.throughput(Throughput::Bytes(order_msg.len() as u64));
    group.bench_function("WsMessage::Order", |b| {
        b.iter(|| {
            let _: WsMessage = serde_json::from_str(std::hint::black_box(order_msg))
                .expect("Deserialization should succeed");
        });
    });

    group.finish();
}

fn bench_book_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("websocket/book_update");

    // BookUpdate - MOST CRITICAL HOT PATH
    // This is the highest frequency message in live trading
    // Deserialized on every orderbook tick (can be 10-100+ per second)

    let book_small = r#"{
        "asset_id": "123456789",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "bids": [{"price": "0.55", "size": "100.0"}],
        "asks": [{"price": "0.56", "size": "150.0"}]
    }"#;

    let book_medium = r#"{
        "asset_id": "123456789",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "hash": "abc123",
        "bids": [
            {"price": "0.55", "size": "100.0"},
            {"price": "0.54", "size": "200.0"},
            {"price": "0.53", "size": "300.0"},
            {"price": "0.52", "size": "400.0"},
            {"price": "0.51", "size": "500.0"}
        ],
        "asks": [
            {"price": "0.56", "size": "150.0"},
            {"price": "0.57", "size": "175.0"},
            {"price": "0.58", "size": "200.0"},
            {"price": "0.59", "size": "225.0"},
            {"price": "0.60", "size": "250.0"}
        ]
    }"#;

    let book_large = r#"{
        "asset_id": "123456789",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "hash": "abc123",
        "bids": [
            {"price": "0.55", "size": "100.0"}, {"price": "0.54", "size": "200.0"},
            {"price": "0.53", "size": "300.0"}, {"price": "0.52", "size": "400.0"},
            {"price": "0.51", "size": "500.0"}, {"price": "0.50", "size": "600.0"},
            {"price": "0.49", "size": "700.0"}, {"price": "0.48", "size": "800.0"},
            {"price": "0.47", "size": "900.0"}, {"price": "0.46", "size": "1000.0"},
            {"price": "0.45", "size": "1100.0"}, {"price": "0.44", "size": "1200.0"},
            {"price": "0.43", "size": "1300.0"}, {"price": "0.42", "size": "1400.0"},
            {"price": "0.41", "size": "1500.0"}, {"price": "0.40", "size": "1600.0"},
            {"price": "0.39", "size": "1700.0"}, {"price": "0.38", "size": "1800.0"},
            {"price": "0.37", "size": "1900.0"}, {"price": "0.36", "size": "2000.0"}
        ],
        "asks": [
            {"price": "0.56", "size": "150.0"}, {"price": "0.57", "size": "175.0"},
            {"price": "0.58", "size": "200.0"}, {"price": "0.59", "size": "225.0"},
            {"price": "0.60", "size": "250.0"}, {"price": "0.61", "size": "275.0"},
            {"price": "0.62", "size": "300.0"}, {"price": "0.63", "size": "325.0"},
            {"price": "0.64", "size": "350.0"}, {"price": "0.65", "size": "375.0"},
            {"price": "0.66", "size": "400.0"}, {"price": "0.67", "size": "425.0"},
            {"price": "0.68", "size": "450.0"}, {"price": "0.69", "size": "475.0"},
            {"price": "0.70", "size": "500.0"}, {"price": "0.71", "size": "525.0"},
            {"price": "0.72", "size": "550.0"}, {"price": "0.73", "size": "575.0"},
            {"price": "0.74", "size": "600.0"}, {"price": "0.75", "size": "625.0"}
        ]
    }"#;

    for (name, json) in [
        ("1_level", book_small),
        ("5_levels", book_medium),
        ("20_levels", book_large),
    ] {
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(BenchmarkId::new("BookUpdate", name), &json, |b, json| {
            b.iter(|| {
                let _: BookUpdate = serde_json::from_str(std::hint::black_box(json))
                    .expect("Deserialization should succeed");
            });
        });
    }

    group.finish();
}

fn bench_user_messages(c: &mut Criterion) {
    let mut group = c.benchmark_group("websocket/user_messages");

    let trade_minimal = r#"{
        "id": "trade_123",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "side": "BUY",
        "size": "25.0",
        "price": "0.55",
        "status": "MATCHED",
        "maker_orders": []
    }"#;

    let trade_full = r#"{
        "id": "trade_123",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "side": "BUY",
        "size": "25.0",
        "price": "0.55",
        "status": "CONFIRMED",
        "type": "TRADE",
        "last_update": "1704110400000",
        "matchtime": "1704110400000",
        "timestamp": "1704110400000",
        "outcome": "Yes",
        "owner": "550e8400-e29b-41d4-a716-446655440000",
        "trade_owner": "550e8400-e29b-41d4-a716-446655440000",
        "taker_order_id": "0xabcdef",
        "maker_orders": [
            {
                "order_id": "0x111",
                "asset_id": "123456789",
                "outcome": "Yes",
                "price": "0.55",
                "matched_amount": "10.0",
                "owner": "550e8400-e29b-41d4-a716-446655440000"
            },
            {
                "order_id": "0x222",
                "asset_id": "123456789",
                "outcome": "Yes",
                "price": "0.55",
                "matched_amount": "15.0",
                "owner": "550e8400-e29b-41d4-a716-446655440000"
            }
        ],
        "fee_rate_bps": "25",
        "transaction_hash": "0x0000000000000000000000000000000000000000000000000000000000000abc",
        "trader_side": "TAKER"
    }"#;

    for (name, json) in [("minimal", trade_minimal), ("full", trade_full)] {
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(BenchmarkId::new("TradeMessage", name), &json, |b, json| {
            b.iter(|| {
                let _: TradeMessage = serde_json::from_str(std::hint::black_box(json))
                    .expect("Deserialization should succeed");
            });
        });
    }

    let order_minimal = r#"{
        "id": "0x123",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "side": "BUY",
        "price": "0.55"
    }"#;

    let order_full = r#"{
        "id": "0x123",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "asset_id": "123456789",
        "side": "BUY",
        "price": "0.55",
        "type": "PLACEMENT",
        "outcome": "Yes",
        "owner": "550e8400-e29b-41d4-a716-446655440000",
        "order_owner": "550e8400-e29b-41d4-a716-446655440000",
        "original_size": "100.0",
        "size_matched": "25.0",
        "timestamp": "1704110400000",
        "associate_trades": ["trade_123", "trade_456"]
    }"#;

    for (name, json) in [("minimal", order_minimal), ("full", order_full)] {
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(BenchmarkId::new("OrderMessage", name), &json, |b, json| {
            b.iter(|| {
                let _: OrderMessage = serde_json::from_str(std::hint::black_box(json))
                    .expect("Deserialization should succeed");
            });
        });
    }

    group.finish();
}

fn bench_market_data_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("websocket/market_data");

    let price_change_single = r#"{
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "price_changes": [{
            "asset_id": "123456789",
            "price": "0.65",
            "side": "BUY"
        }]
    }"#;

    let price_change_batch = r#"{
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "price_changes": [
            {"asset_id": "123456789", "price": "0.65", "side": "BUY", "hash": "abc1", "best_bid": "0.64", "best_ask": "0.66"},
            {"asset_id": "987654321", "price": "0.35", "side": "SELL", "hash": "abc2", "best_bid": "0.34", "best_ask": "0.36"},
            {"asset_id": "555555555", "price": "0.50", "side": "BUY", "hash": "abc3", "best_bid": "0.49", "best_ask": "0.51"}
        ]
    }"#;

    for (name, json) in [
        ("single", price_change_single),
        ("batch_3", price_change_batch),
    ] {
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(BenchmarkId::new("PriceChange", name), &json, |b, json| {
            b.iter(|| {
                let _: PriceChange = serde_json::from_str(std::hint::black_box(json))
                    .expect("Deserialization should succeed");
            });
        });
    }

    let last_trade_price = r#"{
        "asset_id": "123456789",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "price": "0.55",
        "side": "BUY"
    }"#;
    group.throughput(Throughput::Bytes(last_trade_price.len() as u64));
    group.bench_function("LastTradePrice", |b| {
        b.iter(|| {
            let _: LastTradePrice = serde_json::from_str(std::hint::black_box(last_trade_price))
                .expect("Deserialization should succeed");
        });
    });

    let best_bid_ask = r#"{
        "asset_id": "123456789",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "best_bid": "0.54",
        "best_ask": "0.56",
        "spread": "0.02"
    }"#;
    group.throughput(Throughput::Bytes(best_bid_ask.len() as u64));
    group.bench_function("BestBidAsk", |b| {
        b.iter(|| {
            let _: BestBidAsk = serde_json::from_str(std::hint::black_box(best_bid_ask))
                .expect("Deserialization should succeed");
        });
    });

    let tick_size_change = r#"{
        "asset_id": "123456789",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "old_tick_size": "0.01",
        "new_tick_size": "0.001",
        "timestamp": "1"
    }"#;
    group.throughput(Throughput::Bytes(tick_size_change.len() as u64));
    group.bench_function("TickSizeChange", |b| {
        b.iter(|| {
            let _: TickSizeChange = serde_json::from_str(std::hint::black_box(tick_size_change))
                .expect("Deserialization should succeed");
        });
    });

    let midpoint_update = r#"{
        "asset_id": "123456789",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "timestamp": "1234567890123",
        "midpoint": "0.55"
    }"#;
    group.throughput(Throughput::Bytes(midpoint_update.len() as u64));
    group.bench_function("MidpointUpdate", |b| {
        b.iter(|| {
            let _: MidpointUpdate = serde_json::from_str(std::hint::black_box(midpoint_update))
                .expect("Deserialization should succeed");
        });
    });

    group.finish();
}

fn bench_market_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("websocket/market_events");

    let new_market = r#"{
        "id": "1",
        "question": "Will X happen?",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "slug": "test-market-2024",
        "description": "Test market for benchmarking",
        "assets_ids": ["123456789", "987654321"],
        "outcomes": ["Yes", "No"],
        "timestamp": "1704110400000"
    }"#;
    group.throughput(Throughput::Bytes(new_market.len() as u64));
    group.bench_function("NewMarket", |b| {
        b.iter(|| {
            let _: NewMarket = serde_json::from_str(std::hint::black_box(new_market))
                .expect("Deserialization should succeed");
        });
    });

    let market_resolved = r#"{
        "id": "1",
        "question": "Will X happen?",
        "market": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "slug": "test-market-2024",
        "description": "Test market for benchmarking",
        "assets_ids": ["123456789", "987654321"],
        "outcomes": ["Yes", "No"],
        "winning_asset_id": "123456789",
        "winning_outcome": "Yes",
        "timestamp": "1704110400000"
    }"#;
    group.throughput(Throughput::Bytes(market_resolved.len() as u64));
    group.bench_function("MarketResolved", |b| {
        b.iter(|| {
            let _: MarketResolved = serde_json::from_str(std::hint::black_box(market_resolved))
                .expect("Deserialization should succeed");
        });
    });

    group.finish();
}

fn bench_orderbook_level(c: &mut Criterion) {
    let mut group = c.benchmark_group("websocket/primitives");

    let level = r#"{"price": "0.55", "size": "100.0"}"#;
    group.throughput(Throughput::Bytes(level.len() as u64));
    group.bench_function("OrderBookLevel", |b| {
        b.iter(|| {
            let _: OrderBookLevel = serde_json::from_str(std::hint::black_box(level))
                .expect("Deserialization should succeed");
        });
    });

    let maker_order = r#"{
        "order_id": "0x123",
        "asset_id": "123456789",
        "outcome": "Yes",
        "price": "0.55",
        "matched_amount": "10.0",
        "owner": "550e8400-e29b-41d4-a716-446655440000"
    }"#;
    group.throughput(Throughput::Bytes(maker_order.len() as u64));
    group.bench_function("MakerOrder", |b| {
        b.iter(|| {
            let _: MakerOrder = serde_json::from_str(std::hint::black_box(maker_order))
                .expect("Deserialization should succeed");
        });
    });

    group.finish();
}

criterion_group!(
    websocket_benches,
    bench_ws_message,
    bench_book_update,
    bench_user_messages,
    bench_market_data_updates,
    bench_market_events,
    bench_orderbook_level
);
criterion_main!(websocket_benches);
