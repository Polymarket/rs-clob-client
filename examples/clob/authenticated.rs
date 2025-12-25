#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use chrono::{TimeDelta, Utc};
use polymarket_client_sdk::clob::types::{
    Amount, BalanceAllowanceRequest, OrderType, OrdersRequest, Side, TradesRequest,
    UpdateBalanceAllowanceRequest, UserRewardsEarningRequest,
};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token_id = "15871154585880608648532107628464183779895785213830018178010423617714102767076";

    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need a private key");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let config = Config::builder().use_server_time(true).build();
    let client = Client::new("https://clob.polymarket.com", config)?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    println!("api_keys -- {:?}", client.api_keys().await?);
    println!("closed_only_mode -- {:?}", client.closed_only_mode().await?);

    let market_order = client
        .market_order()
        .token_id(token_id)
        .amount(Amount::usdc(Decimal::ONE_HUNDRED)?)
        .side(Side::Buy)
        .build()
        .await?;
    let signed_order = client.sign(&signer, market_order).await?;
    println!(
        "market order -- {:?}",
        client.post_order(signed_order).await
    );

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
    println!("limit order -- {:?}", client.post_order(signed_order).await);

    println!("notifications -- {:?}", client.notifications().await);
    println!(
        "balance -- {:#?}",
        client
            .balance_allowance(&BalanceAllowanceRequest::default())
            .await
    );
    println!(
        "update balance -- {:#?}",
        client
            .update_balance_allowance(&UpdateBalanceAllowanceRequest::default())
            .await
    );

    println!(
        "order -- {:?}",
        client
            .order("0xa1449ec0831c7d62f887c4653d0917f2445783ff30f0ca713d99c667fef17f2c")
            .await
    );
    println!(
        "orders -- {:?}",
        client.orders(&OrdersRequest::default(), None).await
    );
    println!(
        "cancel_order -- {:?}",
        client
            .cancel_order("0xa1449ec0831c7d62f887c4653d0917f2445783ff30f0ca713d99c667fef17f2c")
            .await
    );
    println!(
        "cancel_orders -- {:?}",
        client
            .cancel_orders(&["0xa1449ec0831c7d62f887c4653d0917f2445783ff30f0ca713d99c667fef17f2c"])
            .await
    );

    println!(
        "cancel_all_orders -- {:?}",
        client.cancel_all_orders().await
    );

    println!(
        "orders -- {:?}",
        client.orders(&OrdersRequest::default(), None).await
    );

    println!(
        "trades -- {:?}",
        client.trades(&TradesRequest::default(), None).await
    );
    println!(
        "earnings -- {:?}",
        client
            .earnings_for_user_for_day(Utc::now().date_naive(), None)
            .await
    );
    let request = UserRewardsEarningRequest::builder()
        .date(Utc::now().date_naive() - TimeDelta::days(30))
        .build();
    println!(
        "earnings -- {:?}",
        client
            .user_earnings_and_markets_config(&request, None)
            .await
    );

    println!(
        "reward percentages -- {:?}",
        client.reward_percentages().await
    );

    println!(
        "current rewards -- {:?}",
        client.current_rewards(None).await
    );

    println!(
        "raw rewards -- {:?}",
        client
            .raw_rewards_for_market(
                "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
                None
            )
            .await
    );

    Ok(())
}
