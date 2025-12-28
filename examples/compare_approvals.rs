#![allow(clippy::exhaustive_enums, reason = "Fine for examples")]
#![allow(clippy::exhaustive_structs, reason = "Fine for examples")]
#![allow(clippy::unwrap_used, reason = "Fine for examples")]
#![allow(clippy::print_stdout, reason = "Examples are okay to print to stdout")]

//! Compares OLD (buggy) vs NEW (fixed) approval targets.
//! Shows the bug and the fix without executing any transactions.

use alloy::primitives::{Address, address};
use polymarket_client_sdk::{POLYGON, contract_config};

fn main() {
    let chain = POLYGON;

    let config = contract_config(chain, false).unwrap();
    let neg_risk_config = contract_config(chain, true).unwrap();

    // The CTF Exchange address that SHOULD be approved
    let ctf_exchange = config.exchange;

    // The Conditional Tokens address that the OLD code incorrectly approved
    let conditional_tokens = config.conditional_tokens;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║              APPROVAL COMPARISON: OLD vs NEW                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();

    // ========== OLD CODE (BUGGY) ==========
    println!("┌─────────────────────────────────────────────────────────────────────┐");
    println!("│ OLD CODE (v0.1.2) - BUGGY                                           │");
    println!("├─────────────────────────────────────────────────────────────────────┤");
    println!("│                                                                     │");
    println!("│ // Lines 54-55 in old approvals.rs:                                 │");
    println!("│ approve(&token, config.conditional_tokens, ...).await?;             │");
    println!("│ set_approval_for_all(&ctf, config.conditional_tokens, true).await?; │");
    println!("│                                                                     │");
    println!("└─────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("Contracts the OLD code would approve:");
    println!();

    // What the old code actually did
    let old_targets: Vec<(&str, Address, bool)> = vec![
        ("Conditional Tokens", conditional_tokens, false), // ❌ WRONG - should be CTF Exchange
        ("Neg Risk CTF Exchange", neg_risk_config.exchange, true),
        (
            "Neg Risk Adapter (from AMOY!)",
            address!("0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296"),
            true,
        ),
    ];

    for (name, addr, correct) in &old_targets {
        let status = if *correct { "✅" } else { "❌ WRONG!" };
        println!("  {status} {name}");
        println!("       {addr}");
    }

    println!();
    println!("  ⚠️  CTF Exchange NOT approved: {ctf_exchange}");
    println!("      Users cannot trade on standard markets!");
    println!();

    // ========== NEW CODE (FIXED) ==========
    println!("┌─────────────────────────────────────────────────────────────────────┐");
    println!("│ NEW CODE (this fix) - CORRECT                                       │");
    println!("├─────────────────────────────────────────────────────────────────────┤");
    println!("│                                                                     │");
    println!("│ // Now uses config.exchange (CTF Exchange):                         │");
    println!("│ (\"CTF Exchange\", config.exchange),                                  │");
    println!("│ (\"Neg Risk CTF Exchange\", neg_risk_config.exchange),                │");
    println!("│ (\"Neg Risk Adapter\", neg_risk_config.neg_risk_adapter),             │");
    println!("│                                                                     │");
    println!("└─────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("Contracts the NEW code would approve:");
    println!();

    // What the new code does
    let new_targets: Vec<(&str, Address)> = vec![
        ("CTF Exchange", config.exchange),
        ("Neg Risk CTF Exchange", neg_risk_config.exchange),
        (
            "Neg Risk Adapter",
            neg_risk_config.neg_risk_adapter.unwrap(),
        ),
    ];

    for (name, addr) in &new_targets {
        println!("  ✅ {name}");
        println!("       {addr}");
    }

    println!();
    println!("  ✅ All required contracts are now approved!");
    println!("     Users can trade on both standard and neg-risk markets.");
    println!();

    // ========== SUMMARY ==========
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║ SUMMARY                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════════════╣");
    println!("║                                                                      ║");
    println!("║ The bug: OLD code approved 'conditional_tokens' instead of           ║");
    println!("║          'exchange' (CTF Exchange).                                  ║");
    println!("║                                                                      ║");
    println!("║ Missing approval:                                                    ║");
    println!("║   CTF Exchange: {ctf_exchange}     ║");
    println!("║                                                                      ║");
    println!("║ The fix: NEW code correctly approves config.exchange                 ║");
    println!("║                                                                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}
