//! Configuration management for the BTC 15min trading bot
//!
//! This module handles loading and validating bot configuration from TOML files
//! and environment variables.

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main bot configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Blockchain and API settings
    pub blockchain: BlockchainConfig,

    /// Trading strategy parameters
    pub strategy: StrategyConfig,

    /// Risk management settings
    pub risk: RiskConfig,

    /// Operational settings
    pub operational: OperationalConfig,
}

/// Blockchain and API configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockchainConfig {
    /// Private key for signing transactions (hex format without 0x prefix)
    /// Should be loaded from environment variable for security
    #[serde(skip_serializing)]
    pub private_key: String,

    /// Polymarket CLOB API endpoint
    #[serde(default = "default_clob_endpoint")]
    pub clob_endpoint: String,

    /// Chain ID (137 for Polygon mainnet, 80002 for Amoy testnet)
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,

    /// Polygon RPC endpoint (for token approvals if needed)
    pub rpc_endpoint: Option<String>,
}

/// Trading strategy parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StrategyConfig {
    /// Market search query (e.g., "Bitcoin 15min")
    #[serde(default = "default_market_query")]
    pub market_query: String,

    /// Seconds before market start to analyze and place order
    #[serde(default = "default_entry_timing")]
    pub entry_timing_seconds: u64,

    /// Trade size in USDC
    #[serde(default = "default_trade_size")]
    pub trade_size_usdc: Decimal,

    /// Minimum volume differential to trade (e.g., 0.1 = 10% difference)
    #[serde(default = "default_min_volume_diff")]
    pub min_volume_differential: Decimal,

    /// Lookback period for volume analysis in seconds
    #[serde(default = "default_volume_lookback")]
    pub volume_lookback_seconds: u64,
}

/// Risk management configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiskConfig {
    /// Trailing stop percentage (e.g., 0.05 = 5%)
    #[serde(default = "default_trailing_stop_pct")]
    pub trailing_stop_percentage: Decimal,

    /// Maximum position hold time in seconds (safety exit)
    #[serde(default = "default_max_hold_time")]
    pub max_position_hold_seconds: u64,

    /// Maximum slippage tolerance (e.g., 0.02 = 2%)
    #[serde(default = "default_max_slippage")]
    pub max_slippage: Decimal,

    /// Minimum profit target before enabling trailing stop (e.g., 0.02 = 2%)
    #[serde(default = "default_min_profit_target")]
    pub min_profit_target: Decimal,
}

/// Operational settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OperationalConfig {
    /// Price polling interval in seconds
    #[serde(default = "default_price_poll_interval")]
    pub price_poll_interval_seconds: u64,

    /// Market discovery refresh interval in seconds
    #[serde(default = "default_market_refresh_interval")]
    pub market_refresh_interval_seconds: u64,

    /// API retry attempts
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,

    /// Delay between retries in milliseconds
    #[serde(default = "default_retry_delay_ms")]
    pub retry_delay_ms: u64,

    /// Enable dry run mode (no real trades)
    #[serde(default)]
    pub dry_run: bool,
}

// Default value functions
fn default_clob_endpoint() -> String {
    "https://clob.polymarket.com".to_string()
}

fn default_chain_id() -> u64 {
    137 // Polygon mainnet
}

fn default_market_query() -> String {
    "Bitcoin 15min".to_string()
}

fn default_entry_timing() -> u64 {
    60 // 1 minute before start
}

fn default_trade_size() -> Decimal {
    Decimal::new(5, 0) // $5
}

fn default_min_volume_diff() -> Decimal {
    Decimal::new(10, 2) // 0.10 = 10%
}

fn default_volume_lookback() -> u64 {
    3600 // 1 hour
}

fn default_trailing_stop_pct() -> Decimal {
    Decimal::new(5, 2) // 0.05 = 5%
}

fn default_max_hold_time() -> u64 {
    900 // 15 minutes (market duration)
}

fn default_max_slippage() -> Decimal {
    Decimal::new(2, 2) // 0.02 = 2%
}

fn default_min_profit_target() -> Decimal {
    Decimal::new(2, 2) // 0.02 = 2%
}

fn default_price_poll_interval() -> u64 {
    3 // 3 seconds
}

fn default_market_refresh_interval() -> u64 {
    300 // 5 minutes
}

fn default_retry_attempts() -> u32 {
    3
}

fn default_retry_delay_ms() -> u64 {
    1000 // 1 second
}

impl Config {
    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref())
            .context("Failed to read config file")?;

        let mut config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;

        // Override private key from environment if set
        if let Ok(private_key) = std::env::var("PRIVATE_KEY") {
            config.blockchain.private_key = private_key;
        }

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate private key format
        if self.blockchain.private_key.is_empty() {
            anyhow::bail!("Private key is required");
        }

        // Remove 0x prefix if present
        let key = self.blockchain.private_key.trim_start_matches("0x");
        if key.len() != 64 {
            anyhow::bail!("Private key must be 64 hex characters (32 bytes)");
        }

        // Validate trade size
        if self.strategy.trade_size_usdc <= Decimal::ZERO {
            anyhow::bail!("Trade size must be greater than 0");
        }

        // Validate percentages
        if self.risk.trailing_stop_percentage <= Decimal::ZERO
            || self.risk.trailing_stop_percentage >= Decimal::ONE {
            anyhow::bail!("Trailing stop percentage must be between 0 and 1");
        }

        if self.risk.max_slippage < Decimal::ZERO
            || self.risk.max_slippage >= Decimal::ONE {
            anyhow::bail!("Max slippage must be between 0 and 1");
        }

        // Validate timing
        if self.strategy.entry_timing_seconds == 0 {
            anyhow::bail!("Entry timing must be greater than 0");
        }

        if self.operational.price_poll_interval_seconds == 0 {
            anyhow::bail!("Price poll interval must be greater than 0");
        }

        Ok(())
    }

    /// Load from default config file path
    pub fn load() -> Result<Self> {
        // Try loading from current directory first
        if Path::new("config.toml").exists() {
            return Self::from_file("config.toml");
        }

        // Try loading from config directory
        if Path::new("config/config.toml").exists() {
            return Self::from_file("config/config.toml");
        }

        anyhow::bail!(
            "Config file not found. Please create config.toml in the current directory or config/ directory"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config {
            blockchain: BlockchainConfig {
                private_key: "0".repeat(64),
                clob_endpoint: default_clob_endpoint(),
                chain_id: default_chain_id(),
                rpc_endpoint: None,
            },
            strategy: StrategyConfig {
                market_query: default_market_query(),
                entry_timing_seconds: default_entry_timing(),
                trade_size_usdc: default_trade_size(),
                min_volume_differential: default_min_volume_diff(),
                volume_lookback_seconds: default_volume_lookback(),
            },
            risk: RiskConfig {
                trailing_stop_percentage: default_trailing_stop_pct(),
                max_position_hold_seconds: default_max_hold_time(),
                max_slippage: default_max_slippage(),
                min_profit_target: default_min_profit_target(),
            },
            operational: OperationalConfig {
                price_poll_interval_seconds: default_price_poll_interval(),
                market_refresh_interval_seconds: default_market_refresh_interval(),
                retry_attempts: default_retry_attempts(),
                retry_delay_ms: default_retry_delay_ms(),
                dry_run: false,
            },
        };

        assert!(config.validate().is_ok());
    }
}
