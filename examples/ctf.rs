#![allow(clippy::exhaustive_enums, reason = "Fine for examples")]
#![allow(clippy::exhaustive_structs, reason = "Fine for examples")]
#![allow(clippy::unwrap_used, reason = "Fine for examples")]
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

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

use alloy::primitives::{FixedBytes, U256};
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

const RPC_URL: &str = "https://polygon-rpc.com";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let write_mode = args.iter().any(|arg| arg == "--write");

    println!("=== CTF (Conditional Token Framework) Example ===\n");

    // For read-only operations, we don't need a wallet
    let provider = ProviderBuilder::new().connect(RPC_URL).await?;
    let client = Client::new(provider, POLYGON)?;

    println!("Connected to Polygon mainnet");
    println!("CTF contract: 0x4D97DCd97eC945f40cF65F87097ACe5EA0476045\n");

    // Example: Calculate a condition ID
    println!("--- Calculating Condition ID ---");
    let oracle = address!("0x0000000000000000000000000000000000000001");
    let question_id = FixedBytes::<32>::ZERO;
    let outcome_slot_count = U256::from(2);

    let condition_req = ConditionIdRequest::builder()
        .oracle(oracle)
        .question_id(question_id)
        .outcome_slot_count(outcome_slot_count)
        .build();

    let condition_resp = client.condition_id(&condition_req).await?;
    println!("Oracle: {oracle}");
    println!("Question ID: {question_id}");
    println!("Outcome Slots: {outcome_slot_count}");
    println!("→ Condition ID: {}\n", condition_resp.condition_id);

    // Example: Calculate collection IDs for YES and NO tokens
    println!("--- Calculating Collection IDs ---");
    let parent_collection_id = FixedBytes::<32>::ZERO;

    // Collection ID for YES token (index set = 0b01 = 1)
    let yes_collection_req = CollectionIdRequest::builder()
        .parent_collection_id(parent_collection_id)
        .condition_id(condition_resp.condition_id)
        .index_set(U256::from(1))
        .build();

    let yes_collection_resp = client.collection_id(&yes_collection_req).await?;
    println!("YES token (index set = 1):");
    println!("→ Collection ID: {}\n", yes_collection_resp.collection_id);

    // Collection ID for NO token (index set = 0b10 = 2)
    let no_collection_req = CollectionIdRequest::builder()
        .parent_collection_id(parent_collection_id)
        .condition_id(condition_resp.condition_id)
        .index_set(U256::from(2))
        .build();

    let no_collection_resp = client.collection_id(&no_collection_req).await?;
    println!("NO token (index set = 2):");
    println!("→ Collection ID: {}\n", no_collection_resp.collection_id);

    // Example: Calculate position IDs (ERC1155 token IDs)
    println!("--- Calculating Position IDs ---");
    let usdc = address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174");

    let yes_position_req = PositionIdRequest::builder()
        .collateral_token(usdc)
        .collection_id(yes_collection_resp.collection_id)
        .build();

    let yes_position_resp = client.position_id(&yes_position_req).await?;
    println!(
        "YES position (ERC1155 token ID): {}",
        yes_position_resp.position_id
    );

    let no_position_req = PositionIdRequest::builder()
        .collateral_token(usdc)
        .collection_id(no_collection_resp.collection_id)
        .build();

    let no_position_resp = client.position_id(&no_position_req).await?;
    println!(
        "NO position (ERC1155 token ID): {}\n",
        no_position_resp.position_id
    );

    // Write operations require a wallet
    if write_mode {
        println!("--- Write Operations (requires wallet) ---\n");

        let private_key =
            env::var(PRIVATE_KEY_VAR).expect("Need a private key for write operations");
        let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(POLYGON));

        let provider = ProviderBuilder::new()
            .wallet(signer.clone())
            .connect(RPC_URL)
            .await?;

        let client = Client::new(provider, POLYGON)?;
        let wallet_address = signer.address();

        println!("Using wallet: {wallet_address:?}\n");

        // Example: Split 1 USDC into YES and NO tokens
        println!("--- Splitting Position ---");
        println!("This will split 1 USDC into 1 YES and 1 NO token");
        println!("Note: You must approve the CTF contract to spend your USDC first!\n");

        let split_req = SplitPositionRequest::builder()
            .collateral_token(usdc)
            .condition_id(condition_resp.condition_id)
            .partition(vec![U256::from(1), U256::from(2)])
            .amount(U256::from(1_000_000)) // 1 USDC (6 decimals)
            .build();

        match client.split_position(&split_req).await {
            Ok(split_resp) => {
                println!("✓ Split transaction successful!");
                println!("  Transaction hash: {}", split_resp.transaction_hash);
                println!("  Block number: {}\n", split_resp.block_number);
            }
            Err(e) => {
                println!("✗ Split failed: {e}");
                println!(
                    "  Make sure you have approved the CTF contract and have sufficient USDC\n"
                );
            }
        }

        // Example: Merge YES and NO tokens back into USDC
        println!("--- Merging Positions ---");
        println!("This will merge 1 YES and 1 NO token back into 1 USDC\n");

        let merge_req = MergePositionsRequest::builder()
            .collateral_token(usdc)
            .condition_id(condition_resp.condition_id)
            .partition(vec![U256::from(1), U256::from(2)])
            .amount(U256::from(1_000_000)) // 1 full set
            .build();

        match client.merge_positions(&merge_req).await {
            Ok(merge_resp) => {
                println!("✓ Merge transaction successful!");
                println!("  Transaction hash: {}", merge_resp.transaction_hash);
                println!("  Block number: {}\n", merge_resp.block_number);
            }
            Err(e) => {
                println!("✗ Merge failed: {e}");
                println!("  Make sure you have sufficient YES and NO tokens\n");
            }
        }

        // Example: Redeem winning tokens
        println!("--- Redeeming Positions ---");
        println!("This redeems winning tokens after market resolution\n");

        let redeem_req = RedeemPositionsRequest::builder()
            .collateral_token(usdc)
            .condition_id(condition_resp.condition_id)
            .index_sets(vec![U256::from(1)]) // Redeem YES tokens
            .build();

        match client.redeem_positions(&redeem_req).await {
            Ok(redeem_resp) => {
                println!("✓ Redeem transaction successful!");
                println!("  Transaction hash: {}", redeem_resp.transaction_hash);
                println!("  Block number: {}\n", redeem_resp.block_number);
            }
            Err(e) => {
                println!("✗ Redeem failed: {e}");
                println!("  Make sure the condition is resolved and you have winning tokens\n");
            }
        }
    } else {
        println!("--- Write Operations ---");
        println!("To test write operations (split, merge, redeem), run with --write flag:");
        println!("  export POLYMARKET_PRIVATE_KEY=\"your_private_key\"");
        println!("  cargo run --example ctf --features ctf -- --write\n");
    }

    println!("=== Example Complete ===");

    Ok(())
}
