//! Shows how heartbeats are sent automatically when the corresponding feature flag is enabled.
//!
//! Run with:
//! ```sh
//! RUST_LOG=debug,hyper_util=off,hyper=off,reqwest=off,h2=off,rustls=off cargo run --example heartbeats --features heartbeats,tracing
//! ```
//!
use std::str::FromStr as _;
use std::time::Duration;

use polymarket_client_sdk::auth::{LocalSigner, Signer as _};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let private_key = std::env::var(PRIVATE_KEY_VAR).expect("Need a private key");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

    let config = Config::builder()
        .use_server_time(true)
        .heartbeat_interval(Duration::from_secs(1))
        .build();
    let client = Client::new("https://clob.polymarket.com", config)?
        .authentication_builder(&signer)
        .authenticate()
        .await?;

    tokio::time::sleep(Duration::from_secs(5)).await;

    drop(client);

    tokio::time::sleep(Duration::from_secs(2)).await;

    Ok(())
}
