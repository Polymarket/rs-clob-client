#![allow(clippy::exhaustive_enums, reason = "Fine for examples")]
#![allow(clippy::exhaustive_structs, reason = "Fine for examples")]
#![allow(clippy::unwrap_used, reason = "Fine for examples")]
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

//! Dry-run version of approvals - shows what contracts WOULD be approved
//! without executing any transactions or spending gas.

use alloy::primitives::Address;
use polymarket_client_sdk::{POLYGON, contract_config};

fn main() {
    let chain = POLYGON;

    let config = contract_config(chain, false).unwrap();
    let neg_risk_config = contract_config(chain, true).unwrap();

    println!();
    println!("=== APPROVALS DRY RUN ===");
    println!("Shows what contracts would be approved (no transactions executed)");
    println!();

    // Collect all contracts that need approval
    let mut targets: Vec<(&str, Address)> = vec![
        ("CTF Exchange", config.exchange),
        ("Neg Risk CTF Exchange", neg_risk_config.exchange),
    ];

    // Add the Neg Risk Adapter if available
    if let Some(adapter) = neg_risk_config.neg_risk_adapter {
        targets.push(("Neg Risk Adapter", adapter));
    }

    println!("Contracts that WOULD receive approvals:");
    println!();

    for (name, target) in &targets {
        println!("  {} ", name);
        println!("    → {}", target);
        println!();
    }

    println!("Total: {} contracts would be approved", targets.len());
    println!();
}
