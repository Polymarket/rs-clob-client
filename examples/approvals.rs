#![allow(clippy::exhaustive_enums, reason = "Fine for examples")]
#![allow(clippy::exhaustive_structs, reason = "Fine for examples")]
#![allow(clippy::unwrap_used, reason = "Fine for examples")]
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

//! Token approval example for Polymarket CLOB trading.
//!
//! This example demonstrates how to set the required token allowances for trading on Polymarket.
//! You must approve three contracts:
//!
//! 1. **CTF Exchange** (`config.exchange`) - Standard market trading
//! 2. **Neg Risk CTF Exchange** (`neg_risk_config.exchange`) - Neg-risk market trading
//! 3. **Neg Risk Adapter** (`neg_risk_config.neg_risk_adapter`) - Token minting/splitting for neg-risk
//!
//! Each contract needs two approvals:
//! - ERC-20 approval for USDC (collateral token)
//! - ERC-1155 approval for Conditional Tokens (outcome tokens)
//!
//! You only need to run these approvals once per wallet.
//!
//! ## Usage
//!
//! ```sh
//! # Execute approvals (requires POLYMARKET_PRIVATE_KEY env var)
//! cargo run --example approvals
//!
//! # Dry run - show what would be approved without executing
//! cargo run --example approvals -- --dry-run
//! ```

use std::env;
use std::str::FromStr as _;

use alloy::primitives::U256;
use alloy::providers::ProviderBuilder;
use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use alloy::sol;
use anyhow::Result;
use polymarket_client_sdk::types::{Address, address};
use polymarket_client_sdk::{POLYGON, PRIVATE_KEY_VAR, contract_config};

const RPC_URL: &str = "https://polygon-rpc.com";

const USDC_ADDRESS: Address = address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174");
const TOKEN_TO_APPROVE: Address = USDC_ADDRESS;

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 value) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
    }

    #[sol(rpc)]
    interface IERC1155 {
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address account, address operator) external view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|arg| arg == "--dry-run");

    let chain = POLYGON;
    let config = contract_config(chain, false).unwrap();
    let neg_risk_config = contract_config(chain, true).unwrap();

    // Collect all contracts that need approval
    let mut targets: Vec<(&str, Address)> = vec![
        ("CTF Exchange", config.exchange),
        ("Neg Risk CTF Exchange", neg_risk_config.exchange),
    ];

    // Add the Neg Risk Adapter if available
    if let Some(adapter) = neg_risk_config.neg_risk_adapter {
        targets.push(("Neg Risk Adapter", adapter));
    }

    if dry_run {
        println!();
        println!("=== APPROVALS DRY RUN ===");
        println!("Shows what contracts would be approved (no transactions executed)");
        println!();
        println!("Contracts that WOULD receive approvals:");
        println!();

        for (name, target) in &targets {
            println!("  {name}");
            println!("    -> {target}");
            println!();
        }

        println!("Total: {} contracts would be approved", targets.len());
        println!();
        return Ok(());
    }

    let private_key = env::var(PRIVATE_KEY_VAR).expect("Need a private key");
    let signer = LocalSigner::from_str(&private_key)?.with_chain_id(Some(chain));

    let provider = ProviderBuilder::new()
        .wallet(signer.clone())
        .connect(RPC_URL)
        .await?;

    let owner = signer.address();
    println!("Using address: {owner:?}");

    let token = IERC20::new(TOKEN_TO_APPROVE, provider.clone());
    let ctf = IERC1155::new(config.conditional_tokens, provider.clone());

    println!("\n=== Checking current allowances ===\n");

    for (name, target) in &targets {
        let usdc_allowance = check_allowance(&token, owner, *target).await?;
        let ctf_approved = check_approval_for_all(&ctf, owner, *target).await?;

        println!("{name}:");
        println!("  USDC allowance: {usdc_allowance}");
        println!("  CTF approved: {ctf_approved}");
        println!();
    }

    println!("=== Setting approvals ===\n");

    for (name, target) in &targets {
        println!("Approving {name} ({target:?})...");

        approve(&token, *target, U256::MAX).await?;
        set_approval_for_all(&ctf, *target, true).await?;

        println!();
    }

    println!("=== Verifying approvals ===\n");

    for (name, target) in &targets {
        let usdc_allowance = check_allowance(&token, owner, *target).await?;
        let ctf_approved = check_approval_for_all(&ctf, owner, *target).await?;

        println!("{name}:");
        println!("  USDC allowance: {usdc_allowance}");
        println!("  CTF approved: {ctf_approved}");
        println!();
    }

    println!("All approvals complete!");

    Ok(())
}

async fn check_allowance<P: alloy::providers::Provider>(
    token: &IERC20::IERC20Instance<P>,
    owner: Address,
    spender: Address,
) -> Result<U256> {
    let allowance = token.allowance(owner, spender).call().await?;
    Ok(allowance)
}

async fn check_approval_for_all<P: alloy::providers::Provider>(
    ctf: &IERC1155::IERC1155Instance<P>,
    account: Address,
    operator: Address,
) -> Result<bool> {
    let approved = ctf.isApprovedForAll(account, operator).call().await?;
    Ok(approved)
}

async fn approve<P: alloy::providers::Provider>(
    usdc: &IERC20::IERC20Instance<P>,
    spender: Address,
    amount: U256,
) -> Result<()> {
    println!("  Calling USDC.approve({spender:?}, {amount})...");

    let receipt = usdc.approve(spender, amount).send().await?.watch().await?;

    println!("  USDC approve tx mined: {receipt:?}");

    Ok(())
}

async fn set_approval_for_all<P: alloy::providers::Provider>(
    ctf: &IERC1155::IERC1155Instance<P>,
    operator: Address,
    approved: bool,
) -> Result<()> {
    println!("  Calling CTF.setApprovalForAll({operator:?}, {approved})...");

    let receipt = ctf
        .setApprovalForAll(operator, approved)
        .send()
        .await?
        .watch()
        .await?;

    println!("  CTF setApprovalForAll tx mined: {receipt:?}");

    Ok(())
}
