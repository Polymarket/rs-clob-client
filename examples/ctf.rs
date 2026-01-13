#![allow(clippy::exhaustive_enums, reason = "Fine for examples")]
#![allow(clippy::exhaustive_structs, reason = "Fine for examples")]

//! CTF (Conditional Token Framework) example.
//!
//! This example demonstrates how to interact with the CTF contract to:
//! - Calculate condition IDs, collection IDs, and position IDs
//! - Split USDC collateral into outcome tokens (YES/NO)
//! - Merge outcome tokens back into USDC
//! - Redeem winning tokens after market resolution
//!
//! ## Usage
//!
//! For read-only operations (ID calculations):
//! ```sh
//! cargo run --example ctf --features ctf
//! ```
//!
//! For write operations (split, merge, redeem), you need a private key:
//! ```sh
//! export POLYMARKET_PRIVATE_KEY="your_private_key"
//! cargo run --example ctf --features ctf -- --write
//! ```

use std::env;
use std::str::FromStr as _;

use alloy::primitives::{B256, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use anyhow::Result;
use polymarket_client_sdk::ctf::Client;
use polymarket_client_sdk::ctf::types::{
    CollectionIdRequest, ConditionIdRequest, MergePositionsRequest, PositionIdRequest,
    RedeemPositionsRequest, SplitPositionRequest,
};
use polymarket_client_sdk::types::address;
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR};
use tracing::{error, info};

const RPC_URL: &str = "https://polygon-rpc.com";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let write_mode = args.iter().any(|arg| arg == "--write");

    let chain = POLYGON;
    info!("=== CTF (Conditional Token Framework) Example ===");

    // For read-only operations, we don't need a wallet
    let provider = ProviderBuilder::new().connect(RPC_URL).await?;
    let client = Client::new(provider, chain)?;

    info!("Connected to Polygon {chain}");
    info!("CTF contract: 0x4D97DCd97eC945f40cF65F87097ACe5EA0476045");

    // Example: Calculate a condition ID
    info!("--- Calculating Condition ID ---");
    let oracle = address!("0x0000000000000000000000000000000000000001");
    let question_id = B256::ZERO;
    let outcome_slot_count = U256::from(2);

    let condition_req = ConditionIdRequest::builder()
        .oracle(oracle)
        .question_id(question_id)
        .outcome_slot_count(outcome_slot_count)
        .build();

    let condition_resp = client.condition_id(&condition_req).await?;
    info!("Oracle: {oracle}");
    info!("Question ID: {question_id}");
    info!("Outcome Slots: {outcome_slot_count}");
    info!("→ Condition ID: {}", condition_resp.condition_id);

    // Example: Calculate collection IDs for YES and NO tokens
    info!("--- Calculating Collection IDs ---");
    let parent_collection_id = B256::ZERO;

    // Collection ID for YES token (index set = 0b01 = 1)
    let yes_collection_req = CollectionIdRequest::builder()
        .parent_collection_id(parent_collection_id)
        .condition_id(condition_resp.condition_id)
        .index_set(U256::from(1))
        .build();

    let yes_collection_resp = client.collection_id(&yes_collection_req).await?;
    info!("YES token (index set = 1):");
    info!("→ Collection ID: {}", yes_collection_resp.collection_id);

    // Collection ID for NO token (index set = 0b10 = 2)
    let no_collection_req = CollectionIdRequest::builder()
        .parent_collection_id(parent_collection_id)
        .condition_id(condition_resp.condition_id)
        .index_set(U256::from(2))
        .build();

    let no_collection_resp = client.collection_id(&no_collection_req).await?;
    info!("NO token (index set = 2):");
    info!("→ Collection ID: {}", no_collection_resp.collection_id);

    // Example: Calculate position IDs (ERC1155 token IDs)
    info!("--- Calculating Position IDs ---");
    let usdc = address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174");

    let yes_position_req = PositionIdRequest::builder()
        .collateral_token(usdc)
        .collection_id(yes_collection_resp.collection_id)
        .build();

    let yes_position_resp = client.position_id(&yes_position_req).await?;
    info!(
        "YES position (ERC1155 token ID): {}",
        yes_position_resp.position_id
    );

    let no_position_req = PositionIdRequest::builder()
        .collateral_token(usdc)
        .collection_id(no_collection_resp.collection_id)
        .build();

    let no_position_resp = client.position_id(&no_position_req).await?;
    info!(
        "NO position (ERC1155 token ID): {}",
        no_position_resp.position_id
    );

    // Write operations require a wallet
    if write_mode {
        info!("--- Write Operations (requires wallet) ---");

        let private_key =
            env::var(PRIVATE_KEY_VAR).expect("Need a private key for write operations");
        let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(chain));

        let provider = ProviderBuilder::new()
            .wallet(signer.clone())
            .connect(RPC_URL)
            .await?;

        let client = Client::new(provider, chain)?;
        let wallet_address = signer.address();

        info!("Using wallet: {wallet_address:?}");

        // Example: Split 1 USDC into YES and NO tokens (using convenience method)
        info!("--- Splitting Position (Binary Market) ---");
        info!("This will split 1 USDC into 1 YES and 1 NO token");
        info!("Note: You must approve the CTF contract to spend your USDC first!");

        // Using the convenience method for binary markets
        let split_req = SplitPositionRequest::for_binary_market(
            usdc,
            condition_resp.condition_id,
            U256::from(1_000_000), // 1 USDC (6 decimals)
        );

        match client.split_position(&split_req).await {
            Ok(split_resp) => {
                info!("✓ Split transaction successful!");
                info!("  Transaction hash: {}", split_resp.transaction_hash);
                info!("  Block number: {}", split_resp.block_number);
            }
            Err(e) => {
                error!("✗ Split failed: {e}");
                error!("  Make sure you have approved the CTF contract and have sufficient USDC");
            }
        }

        // Example: Merge YES and NO tokens back into USDC (using convenience method)
        info!("--- Merging Positions (Binary Market) ---");
        info!("This will merge 1 YES and 1 NO token back into 1 USDC");

        // Using the convenience method for binary markets
        let merge_req = MergePositionsRequest::for_binary_market(
            usdc,
            condition_resp.condition_id,
            U256::from(1_000_000), // 1 full set
        );

        match client.merge_positions(&merge_req).await {
            Ok(merge_resp) => {
                info!("✓ Merge transaction successful!");
                info!("  Transaction hash: {}", merge_resp.transaction_hash);
                info!("  Block number: {}", merge_resp.block_number);
            }
            Err(e) => {
                error!("✗ Merge failed: {e}");
                error!("  Make sure you have sufficient YES and NO tokens");
            }
        }

        // Example: Redeem winning tokens
        info!("--- Redeeming Positions ---");
        info!("This redeems winning tokens after market resolution");

        // Using the convenience method for binary markets (redeems both YES and NO tokens)
        let redeem_req =
            RedeemPositionsRequest::for_binary_market(usdc, condition_resp.condition_id);

        match client.redeem_positions(&redeem_req).await {
            Ok(redeem_resp) => {
                info!("✓ Redeem transaction successful!");
                info!("  Transaction hash: {}", redeem_resp.transaction_hash);
                info!("  Block number: {}", redeem_resp.block_number);
            }
            Err(e) => {
                error!("✗ Redeem failed: {e}");
                error!("  Make sure the condition is resolved and you have winning tokens");
            }
        }
    } else {
        info!("--- Write Operations ---");
        info!("To test write operations (split, merge, redeem), run with --write flag:");
        info!("  export POLYMARKET_PRIVATE_KEY=\"your_private_key\"");
        info!("  cargo run --example ctf --features ctf -- --write");
    }

    info!("=== Example Complete ===");

    Ok(())
}
