#![allow(clippy::exhaustive_enums, reason = "Fine for examples")]
#![allow(clippy::exhaustive_structs, reason = "Fine for examples")]
#![allow(clippy::unwrap_used, reason = "Fine for examples")]
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

//! Read-only example to check current token approvals for Polymarket CLOB trading.
//!
//! This example queries the blockchain to show which contracts are approved
//! for a given wallet address. No private key or gas required.
//!
//! Usage:
//!   `cargo run --example check_approvals -- <WALLET_ADDRESS>`
//!
//! Example:
//!   `cargo run --example check_approvals -- 0x1234...abcd`

use std::env;

use alloy::primitives::U256;
use alloy::providers::ProviderBuilder;
use alloy::sol;
use anyhow::Result;
use polymarket_client_sdk::types::{Address, address};
use polymarket_client_sdk::{POLYGON, contract_config};

const RPC_URL: &str = "https://polygon-rpc.com";

const USDC_ADDRESS: Address = address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174");

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function allowance(address owner, address spender) external view returns (uint256);
    }

    #[sol(rpc)]
    interface IERC1155 {
        function isApprovedForAll(address account, address operator) external view returns (bool);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run --example check_approvals -- <WALLET_ADDRESS>");
        println!(
            "Example: cargo run --example check_approvals -- 0x1234567890abcdef1234567890abcdef12345678"
        );
        std::process::exit(1);
    }

    let wallet_address: Address = args[1].parse()?;

    let provider = ProviderBuilder::new().connect(RPC_URL).await?;

    let config = contract_config(POLYGON, false).unwrap();
    let neg_risk_config = contract_config(POLYGON, true).unwrap();

    let usdc = IERC20::new(USDC_ADDRESS, provider.clone());
    let ctf = IERC1155::new(config.conditional_tokens, provider.clone());

    // All contracts that need approval for full CLOB trading
    let mut targets: Vec<(&str, Address)> = vec![
        ("CTF Exchange", config.exchange),
        ("Neg Risk CTF Exchange", neg_risk_config.exchange),
    ];

    if let Some(adapter) = neg_risk_config.neg_risk_adapter {
        targets.push(("Neg Risk Adapter", adapter));
    }

    println!();
    println!("Checking approvals for wallet: {wallet_address}");
    println!("Chain: Polygon Mainnet (137)");
    println!();
    println!("{}", "=".repeat(70));

    let mut all_approved = true;

    for (name, target) in &targets {
        let usdc_allowance = usdc.allowance(wallet_address, *target).call().await?;
        let ctf_approved = ctf.isApprovedForAll(wallet_address, *target).call().await?;

        let usdc_ok = usdc_allowance > U256::ZERO;
        let ctf_ok = ctf_approved;

        let usdc_status = if usdc_ok { "✅" } else { "❌" };
        let ctf_status = if ctf_ok { "✅" } else { "❌" };

        println!();
        println!("{name}");
        println!("  Address: {target}");
        println!(
            "  USDC allowance: {} {}",
            format_allowance(usdc_allowance),
            usdc_status
        );
        println!("  CTF approved:   {ctf_approved} {ctf_status}");

        if !usdc_ok || !ctf_ok {
            all_approved = false;
        }
    }

    println!();
    println!("{}", "=".repeat(70));
    println!();

    if all_approved {
        println!("✅ All contracts are properly approved! Ready to trade.");
    } else {
        println!("❌ Some approvals are missing. Run the approvals example to fix.");
        println!("   cargo run --example approvals");
    }

    println!();

    Ok(())
}

fn format_allowance(allowance: U256) -> String {
    if allowance == U256::MAX {
        "MAX (unlimited)".to_owned()
    } else if allowance == U256::ZERO {
        "0".to_owned()
    } else {
        // USDC has 6 decimals
        let usdc_decimals = U256::from(1_000_000);
        let whole = allowance / usdc_decimals;
        format!("{whole} USDC")
    }
}
