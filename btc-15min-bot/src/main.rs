//! Bitcoin 15min Trading Bot for Polymarket
//!
//! This bot automatically trades "Bitcoin 15min Up or Down" markets on Polymarket
//! using a volume-based strategy with trailing take profit.
//!
//! Strategy:
//! 1. Discover the next upcoming BTC 15min market
//! 2. 60 seconds before market start, analyze UP vs DOWN volume
//! 3. Enter position on the side with higher volume
//! 4. Use trailing take profit to exit
//! 5. Move to next market and repeat

mod config;
mod market;
mod strategy;
mod trader;
mod trailing_stop;
mod utils;

use anyhow::{Context, Result};
use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::{Client, Config as ClientConfig};
use std::str::FromStr;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::market::{BtcMarket, MarketDiscovery};
use crate::strategy::VolumeStrategy;
use crate::trader::Trader;
use crate::utils::{format_duration, retry_with_backoff, sleep_seconds, sleep_until};

/// Bot state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BotState {
    /// Looking for next market
    Discovery,
    /// Waiting for entry timing
    Waiting,
    /// Analyzing volume
    Analyzing,
    /// In position, monitoring
    InPosition,
    /// Error recovery
    Error,
}

/// Main bot orchestrator
struct Bot {
    trader: Trader,
    market_discovery: MarketDiscovery,
    strategy: VolumeStrategy,
    config: Config,
    state: BotState,
    current_market: Option<BtcMarket>,
}

impl Bot {
    /// Create a new bot instance
    async fn new(config: Config) -> Result<Self> {
        info!("Initializing Bitcoin 15min Trading Bot...");

        // Validate and prepare private key
        let private_key = utils::validate_private_key(&config.blockchain.private_key)?;

        // Create signer
        let signer = LocalSigner::from_str(&private_key)
            .context("Failed to create signer from private key")?
            .with_chain_id(Some(config.blockchain.chain_id));

        info!("Wallet address: {:?}", signer.address());

        // Create CLOB client
        let client_config = ClientConfig::default();
        let client = Client::new(&config.blockchain.clob_endpoint, client_config)
            .context("Failed to create CLOB client")?;

        // Authenticate
        info!("Authenticating with Polymarket...");
        let authenticated_client = client
            .authentication_builder(&signer)
            .authenticate()
            .await
            .context("Failed to authenticate with Polymarket")?;

        info!("Authentication successful");

        // Create components
        let market_discovery = MarketDiscovery::new(
            authenticated_client.clone(),
            config.strategy.market_query.clone(),
        );

        let strategy = VolumeStrategy::new(
            authenticated_client.clone(),
            config.strategy.clone(),
        );

        let trader = Trader::new(authenticated_client, signer, config.clone());

        Ok(Self {
            trader,
            market_discovery,
            strategy,
            config,
            state: BotState::Discovery,
            current_market: None,
        })
    }

    /// Main bot loop
    async fn run(&mut self) -> Result<()> {
        info!("Starting bot main loop...");

        if self.config.operational.dry_run {
            warn!("⚠️  DRY RUN MODE ENABLED - No real trades will be executed ⚠️");
        }

        loop {
            match self.state {
                BotState::Discovery => {
                    if let Err(e) = self.discover_next_market().await {
                        error!("Market discovery failed: {}", e);
                        self.state = BotState::Error;
                        continue;
                    }
                }

                BotState::Waiting => {
                    if let Err(e) = self.wait_for_entry_timing().await {
                        error!("Wait phase failed: {}", e);
                        self.state = BotState::Error;
                        continue;
                    }
                }

                BotState::Analyzing => {
                    if let Err(e) = self.analyze_and_enter().await {
                        error!("Analysis/entry failed: {}", e);
                        // Move to next market
                        self.state = BotState::Discovery;
                        continue;
                    }
                }

                BotState::InPosition => {
                    if let Err(e) = self.monitor_position().await {
                        error!("Position monitoring failed: {}", e);
                        self.state = BotState::Error;
                        continue;
                    }
                }

                BotState::Error => {
                    warn!("In error state, attempting recovery...");
                    self.handle_error().await;
                }
            }

            // Small delay to avoid tight loops
            sleep_seconds(1).await;
        }
    }

    /// Discover next upcoming market
    async fn discover_next_market(&mut self) -> Result<()> {
        info!("Discovering next upcoming market...");

        let market = retry_with_backoff(
            self.config.operational.retry_attempts,
            self.config.operational.retry_delay_ms,
            || async {
                self.market_discovery
                    .find_next_upcoming_market()
                    .await
            },
        )
        .await?;

        match market {
            Some(m) => {
                let time_until_start = MarketDiscovery::seconds_until_start(&m);

                info!(
                    "Found next market: {} | Starts in: {}",
                    m.question,
                    format_duration(chrono::Duration::seconds(time_until_start))
                );

                self.current_market = Some(m);
                self.state = BotState::Waiting;
            }
            None => {
                warn!("No upcoming markets found, retrying in 5 minutes...");
                sleep_seconds(self.config.operational.market_refresh_interval_seconds).await;
            }
        }

        Ok(())
    }

    /// Wait for entry timing (60 seconds before start)
    async fn wait_for_entry_timing(&mut self) -> Result<()> {
        let market = self
            .current_market
            .as_ref()
            .context("No current market")?;

        let time_until_start = MarketDiscovery::seconds_until_start(market);

        if time_until_start <= 0 {
            warn!("Market has already started, moving to next market");
            self.state = BotState::Discovery;
            return Ok(());
        }

        // Check if it's time to analyze
        if time_until_start <= self.config.strategy.entry_timing_seconds as i64 {
            info!("Entry timing reached, proceeding to analysis");
            self.state = BotState::Analyzing;
            return Ok(());
        }

        // Calculate when to wake up
        let seconds_to_wait = time_until_start - self.config.strategy.entry_timing_seconds as i64;

        if seconds_to_wait > 300 {
            // If more than 5 minutes, log status
            info!(
                "Waiting {} until entry timing ({} before market start)",
                format_duration(chrono::Duration::seconds(seconds_to_wait)),
                format_duration(chrono::Duration::seconds(
                    self.config.strategy.entry_timing_seconds as i64
                ))
            );
        }

        // Sleep until entry timing
        let entry_time = market.start_time
            - chrono::Duration::seconds(self.config.strategy.entry_timing_seconds as i64);

        sleep_until(entry_time).await;

        Ok(())
    }

    /// Analyze volume and enter position
    async fn analyze_and_enter(&mut self) -> Result<()> {
        let market = self
            .current_market
            .as_ref()
            .context("No current market")?
            .clone();

        info!("Analyzing market volume...");

        // Analyze market
        let signal = retry_with_backoff(
            self.config.operational.retry_attempts,
            self.config.operational.retry_delay_ms,
            || async { self.strategy.analyze_market(&market).await },
        )
        .await?;

        match signal {
            Some(sig) => {
                info!(
                    "Signal generated: {} (confidence: {:.2}%)",
                    sig.side,
                    sig.confidence * rust_decimal::Decimal::ONE_HUNDRED
                );

                // Enter position
                self.trader.enter_position(&market, &sig).await?;

                self.state = BotState::InPosition;
            }
            None => {
                warn!("No trading signal generated, moving to next market");
                self.state = BotState::Discovery;
            }
        }

        Ok(())
    }

    /// Monitor position and check trailing stop
    async fn monitor_position(&mut self) -> Result<()> {
        if !self.trader.has_position() {
            warn!("No position to monitor, moving to discovery");
            self.state = BotState::Discovery;
            return Ok(());
        }

        // Update position and check for exit
        let exit_reason = retry_with_backoff(
            self.config.operational.retry_attempts,
            self.config.operational.retry_delay_ms,
            || async { self.trader.update_position().await },
        )
        .await?;

        if let Some(reason) = exit_reason {
            info!("Exit condition triggered: {}", reason);

            // Exit position
            self.trader.exit_position(reason).await?;

            // Move to next market
            self.state = BotState::Discovery;
            return Ok(());
        }

        // Check if market has ended
        if let Some(market) = &self.current_market {
            if MarketDiscovery::has_market_ended(market) {
                warn!("Market has ended, force closing position");
                self.trader
                    .exit_position(trailing_stop::ExitReason::MarketEnded)
                    .await?;
                self.state = BotState::Discovery;
                return Ok(());
            }
        }

        // Sleep before next price check
        sleep_seconds(self.config.operational.price_poll_interval_seconds).await;

        Ok(())
    }

    /// Handle error state
    async fn handle_error(&mut self) {
        // Try to safely exit any open position
        if self.trader.has_position() {
            warn!("Attempting to close position due to error");
            if let Err(e) = self
                .trader
                .exit_position(trailing_stop::ExitReason::Manual)
                .await
            {
                error!("Failed to close position: {}", e);
            }
        }

        // Reset to discovery state
        self.current_market = None;
        self.state = BotState::Discovery;

        // Wait before retrying
        warn!("Waiting 60 seconds before retry...");
        sleep_seconds(60).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    // Initialize logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    info!("╔═══════════════════════════════════════════════════════════╗");
    info!("║     Bitcoin 15min Trading Bot for Polymarket             ║");
    info!("║     Version 0.1.0                                         ║");
    info!("╚═══════════════════════════════════════════════════════════╝");

    // Load configuration
    info!("Loading configuration...");
    let config = Config::load().context("Failed to load configuration")?;

    info!("Configuration loaded successfully");
    info!("CLOB Endpoint: {}", config.blockchain.clob_endpoint);
    info!("Chain ID: {}", config.blockchain.chain_id);
    info!("Market Query: {}", config.strategy.market_query);
    info!(
        "Trade Size: ${:.2}",
        config.strategy.trade_size_usdc
    );
    info!(
        "Trailing Stop: {:.2}%",
        config.risk.trailing_stop_percentage * rust_decimal::Decimal::ONE_HUNDRED
    );

    // Create and run bot
    let mut bot = Bot::new(config).await?;

    // Run bot (infinite loop)
    bot.run().await?;

    Ok(())
}
