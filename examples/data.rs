#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use polymarket_client_sdk::data_api::Client;
use polymarket_client_sdk::data_api::types::{
    ActivityLimit, ActivityRequest, BuilderLeaderboardLimit, BuilderLeaderboardRequest,
    BuilderVolumeRequest, ClosedPositionsLimit, ClosedPositionsRequest, HoldersLimit,
    HoldersRequest, LeaderboardCategory, LiveVolumeRequest, OpenInterestRequest, PositionsLimit,
    PositionsRequest, TimePeriod, TradedRequest, TraderLeaderboardLimit, TraderLeaderboardRequest,
    TradesRequest, ValueRequest,
};

const EXAMPLE_USER: &str = "0x56687bf447db6ffa42ffe2204a05edaa20f55839";

const EXAMPLE_MARKET: &str = "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::default();

    let user = EXAMPLE_USER.to_owned();
    let market = EXAMPLE_MARKET.to_owned();

    println!("health -- {:?}", client.health().await);

    let request = PositionsRequest::builder()
        .user(user.clone())
        .limit(PositionsLimit::new(5)?)
        .build();
    println!("positions -- {:?}", client.positions(&request).await);

    println!(
        "trades default -- {:?}",
        client.trades(&TradesRequest::default()).await
    );

    let request = ActivityRequest::builder()
        .user(user.clone())
        .limit(ActivityLimit::new(5)?)
        .build();
    println!("activity -- {:?}", client.activity(&request).await);

    let request = HoldersRequest::builder()
        .markets(vec![market.clone()])
        .limit(HoldersLimit::new(5)?)
        .build();
    println!("holders -- {:?}", client.holders(&request).await);

    let request = ValueRequest::builder().user(user.clone()).build();
    println!("value -- {:?}", client.value(&request).await);

    let request = ClosedPositionsRequest::builder()
        .user(user.clone())
        .limit(ClosedPositionsLimit::new(5)?)
        .build();
    println!(
        "closed_positions -- {:?}",
        client.closed_positions(&request).await
    );

    let request = TraderLeaderboardRequest::builder()
        .category(LeaderboardCategory::Overall)
        .time_period(TimePeriod::Week)
        .limit(TraderLeaderboardLimit::new(5)?)
        .build();
    println!("leaderboard -- {:?}", client.leaderboard(&request).await);

    let request = TradedRequest::builder().user(user.clone()).build();
    println!("traded -- {:?}", client.traded(&request).await);

    println!(
        "open_interest -- {:?}",
        client.open_interest(&OpenInterestRequest::default()).await
    );

    let request = LiveVolumeRequest::builder().id(1).build();
    println!("live_volume -- {:?}", client.live_volume(&request).await);

    let request = BuilderLeaderboardRequest::builder()
        .time_period(TimePeriod::Week)
        .limit(BuilderLeaderboardLimit::new(5)?)
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
