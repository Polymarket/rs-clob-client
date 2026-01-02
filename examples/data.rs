#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use polymarket_client_sdk::data::Client;
use polymarket_client_sdk::data::types::request::{
    ActivityRequest, BuilderLeaderboardRequest, BuilderVolumeRequest, ClosedPositionsRequest,
    HoldersRequest, LiveVolumeRequest, OpenInterestRequest, PositionsRequest, TradedRequest,
    TraderLeaderboardRequest, TradesRequest, ValueRequest,
};
use polymarket_client_sdk::data::types::{LeaderboardCategory, TimePeriod};
use polymarket_client_sdk::types::address;

const EXAMPLE_MARKET: &str = "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = Client::default();

    let user = address!("56687bf447db6ffa42ffe2204a05edaa20f55839");
    let market = EXAMPLE_MARKET.to_owned();

    println!("health -- {:?}", client.health().await);

    let request = PositionsRequest::builder().user(user).limit(5)?.build();
    println!("positions -- {:?}", client.positions(&request).await);

    println!(
        "trades default -- {:?}",
        client.trades(&TradesRequest::default()).await
    );

    let request = ActivityRequest::builder().user(user).limit(5)?.build();
    println!("activity -- {:?}", client.activity(&request).await);

    let request = HoldersRequest::builder()
        .markets(vec![market.clone()])
        .limit(5)?
        .build();
    println!("holders -- {:?}", client.holders(&request).await);

    let request = ValueRequest::builder().user(user).build();
    println!("value -- {:?}", client.value(&request).await);

    let request = ClosedPositionsRequest::builder()
        .user(user)
        .limit(5)?
        .build();
    println!(
        "closed_positions -- {:?}",
        client.closed_positions(&request).await
    );

    let request = TraderLeaderboardRequest::builder()
        .category(LeaderboardCategory::Overall)
        .time_period(TimePeriod::Week)
        .limit(5)?
        .build();
    println!("leaderboard -- {:?}", client.leaderboard(&request).await);

    let request = TradedRequest::builder().user(user).build();
    println!("traded -- {:?}", client.traded(&request).await);

    println!(
        "open_interest -- {:?}",
        client.open_interest(&OpenInterestRequest::default()).await
    );

    let request = LiveVolumeRequest::builder().id(1).build();
    println!("live_volume -- {:?}", client.live_volume(&request).await);

    let request = BuilderLeaderboardRequest::builder()
        .time_period(TimePeriod::Week)
        .limit(5)?
        .build();
    println!(
        "builder_leaderboard -- {:?}",
        client.builder_leaderboard(&request).await
    );

    let request = BuilderVolumeRequest::builder()
        .time_period(TimePeriod::Week)
        .build();
    println!(
        "builder_volume -- {:?}",
        client.builder_volume(&request).await
    );

    Ok(())
}
