#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

use polymarket_client_sdk::clob::types::{
    LastTradePriceRequest, MidpointRequest, OrderBookSummaryRequest, PriceRequest, Side,
    SpreadRequest,
};
use polymarket_client_sdk::clob::{Client, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("https://clob.polymarket.com", Config::default())?;

    let token_id = "102200530570339469387764365697342150521708074903735836831685780223982723092914";
    let condition_id = "0x65805e37d6c891808a44064013a0c80babf87010fe6e69204b17381fd0761fdd";

    println!("ok -- {:?}", client.ok().await);
    println!("server_time -- {:?}", client.server_time().await);

    let midpoint_request = MidpointRequest::builder().token_id(token_id).build();
    println!("midpoint -- {:?}", client.midpoint(&midpoint_request).await);
    println!(
        "midpoints -- {:?}",
        client.midpoints(&[midpoint_request]).await
    );

    let price_request = PriceRequest::builder()
        .token_id(token_id)
        .side(Side::Sell)
        .build();
    println!("price -- {:?}", client.price(&price_request).await);
    println!("prices -- {:?}", client.prices(&[price_request]).await);

    let spread_request = SpreadRequest::builder().token_id(token_id).build();
    println!("spread -- {:?}", client.spread(&spread_request).await);
    println!("spreads -- {:?}", client.spreads(&[spread_request]).await);

    println!("tick_size -- {:?}", client.tick_size(token_id).await);
    println!("neg_risk -- {:?}", client.neg_risk(token_id).await);
    println!("fee_rate_bps -- {:?}", client.fee_rate_bps(token_id).await);

    let order_book_request = OrderBookSummaryRequest::builder()
        .token_id(token_id)
        .build();
    let book = client.order_book(&order_book_request).await;
    if let Ok(book) = book {
        println!("order_book -- {book:?}");
        println!("order_book hash -- {:?}", book.hash());
    }

    println!(
        "order_books -- {:?}",
        client.order_books(&[order_book_request]).await
    );

    let last_trade_price_request = LastTradePriceRequest::builder().token_id(token_id).build();
    println!(
        "last_trade_price -- {:?}",
        client.last_trade_price(&last_trade_price_request).await
    );
    println!(
        "last_trade_prices -- {:?}",
        client.last_trades_prices(&[last_trade_price_request]).await
    );

    println!("market -- {:?}", client.market(condition_id).await);
    println!("markets -- {:?}", client.markets(None).await);
    println!(
        "sampling_markets -- {:?}",
        client.sampling_markets(None).await
    );
    println!(
        "simplified_markets -- {:?}",
        client.simplified_markets(None).await
    );
    println!(
        "sampling_simplified_markets -- {:?}",
        client.sampling_simplified_markets(None).await
    );

    Ok(())
}
