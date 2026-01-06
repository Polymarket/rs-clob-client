//! Order execution and position management
//!
//! This module handles:
//! - Placing market orders
//! - Tracking open positions
//! - Executing exits (trailing stop, etc.)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use alloy::signers::Signer;
use alloy::signers::local::LocalSigner;
use polymarket_client_sdk::clob::Client;
use polymarket_client_sdk::clob::types::{Amount, OrderType, Side};
use polymarket_client_sdk::types::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::config::{Config, RiskConfig};
use crate::market::{BtcMarket, MarketSide};
use crate::strategy::TradingSignal;
use crate::trailing_stop::{ExitReason, TrailingStop};

/// Active position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Market being traded
    pub market: BtcMarket,

    /// Side (UP or DOWN)
    pub side: MarketSide,

    /// Token ID being held
    pub token_id: String,

    /// Entry price (average fill price)
    pub entry_price: Decimal,

    /// Position size in USDC
    pub size_usdc: Decimal,

    /// Number of shares/contracts
    pub shares: Decimal,

    /// Entry timestamp
    pub entry_time: DateTime<Utc>,

    /// Order IDs associated with this position
    pub order_ids: Vec<String>,

    /// Trailing stop manager
    pub trailing_stop: TrailingStop,
}

/// Trader for executing orders
pub struct Trader {
    client: Client,
    signer: LocalSigner,
    config: Config,
    current_position: Option<Position>,
}

impl Trader {
    /// Create a new trader
    pub fn new(client: Client, signer: LocalSigner, config: Config) -> Self {
        Self {
            client,
            signer,
            config,
            current_position: None,
        }
    }

    /// Check if there's an open position
    pub fn has_position(&self) -> bool {
        self.current_position.is_some()
    }

    /// Get current position
    pub fn position(&self) -> Option<&Position> {
        self.current_position.as_ref()
    }

    /// Enter a position based on trading signal
    pub async fn enter_position(
        &mut self,
        market: &BtcMarket,
        signal: &TradingSignal,
    ) -> Result<Position> {
        if self.has_position() {
            anyhow::bail!("Already have an open position");
        }

        info!(
            "Entering {} position on market: {}",
            signal.side, market.question
        );

        // Determine which token to buy
        let token_id = match signal.side {
            MarketSide::Up => &market.up_token_id,
            MarketSide::Down => &market.down_token_id,
        };

        // Get current price for the token
        let current_price = self.get_token_price(token_id).await?;

        info!(
            "Current {} price: {:.4}",
            signal.side, current_price
        );

        // Calculate shares to buy with available USDC
        // For binary outcomes: shares = USDC / price
        let shares = self.config.strategy.trade_size_usdc / current_price;

        if self.config.operational.dry_run {
            warn!("DRY RUN MODE: Would place order for {:.2} shares at {:.4}", shares, current_price);
        } else {
            // Place market order
            self.place_market_order(token_id, shares, signal.side).await?;
        }

        // Create position
        let position = Position {
            market: market.clone(),
            side: signal.side,
            token_id: token_id.clone(),
            entry_price: current_price,
            size_usdc: self.config.strategy.trade_size_usdc,
            shares,
            entry_time: Utc::now(),
            order_ids: vec![],
            trailing_stop: TrailingStop::new(
                current_price,
                signal.side,
                self.config.risk.trailing_stop_percentage,
                self.config.risk.min_profit_target,
            ),
        };

        info!(
            "Position opened: {} {} shares @ {:.4} (${:.2})",
            position.side,
            position.shares,
            position.entry_price,
            position.size_usdc
        );

        self.current_position = Some(position.clone());
        Ok(position)
    }

    /// Exit current position
    pub async fn exit_position(&mut self, reason: ExitReason) -> Result<()> {
        let position = self
            .current_position
            .as_ref()
            .context("No position to exit")?;

        info!(
            "Exiting {} position | Reason: {}",
            position.side, reason
        );

        let current_price = position.trailing_stop.current_price();
        let profit_pct = position.trailing_stop.current_profit_percentage();
        let profit_usdc = position.trailing_stop.current_profit_usdc(position.size_usdc);

        info!(
            "Exit price: {:.4} | P&L: ${:.2} ({:.2}%)",
            current_price,
            profit_usdc,
            profit_pct * Decimal::ONE_HUNDRED
        );

        if self.config.operational.dry_run {
            warn!("DRY RUN MODE: Would sell {} shares at {:.4}", position.shares, current_price);
        } else {
            // Place market sell order
            self.place_market_sell(&position.token_id, position.shares).await?;
        }

        // Clear position
        self.current_position = None;

        info!("Position closed successfully");
        Ok(())
    }

    /// Update position with current price and check trailing stop
    pub async fn update_position(&mut self) -> Result<Option<ExitReason>> {
        let position = match self.current_position.as_mut() {
            Some(p) => p,
            None => return Ok(None),
        };

        // Fetch current price
        let current_price = self.get_token_price(&position.token_id).await?;

        debug!(
            "Position update | Price: {:.4} | P&L: {:.2}%",
            current_price,
            position.trailing_stop.current_profit_percentage() * Decimal::ONE_HUNDRED
        );

        // Update trailing stop
        let exit_reason = position.trailing_stop.update(current_price);

        // Check maximum hold time
        if exit_reason.is_none() {
            let hold_time = position.trailing_stop.time_since_entry().num_seconds() as u64;
            if hold_time >= self.config.risk.max_position_hold_seconds {
                warn!(
                    "Maximum hold time {} seconds exceeded",
                    self.config.risk.max_position_hold_seconds
                );
                return Ok(Some(ExitReason::MaxHoldTime));
            }
        }

        Ok(exit_reason)
    }

    /// Get current price for a token
    async fn get_token_price(&self, token_id: &str) -> Result<Decimal> {
        // Try to get last trade price first
        if let Ok(Some(trade_price)) = self.client.last_trade_price(token_id).await {
            return Ok(trade_price);
        }

        // Fallback to midpoint price
        if let Ok(Some(midpoint)) = self.client.midpoint(token_id).await {
            return Ok(midpoint);
        }

        // Fallback to price endpoint
        let price = self
            .client
            .price(token_id)
            .await
            .context("Failed to fetch token price")?;

        Ok(price)
    }

    /// Place a market buy order
    async fn place_market_order(
        &self,
        token_id: &str,
        shares: Decimal,
        side: MarketSide,
    ) -> Result<()> {
        info!(
            "Placing market order: BUY {:.2} shares of {} token",
            shares, side
        );

        // Build market order
        let order = self
            .client
            .market_order()
            .token_id(token_id)
            .amount(Amount::usdc(shares)?)
            .side(Side::Buy)
            .build()
            .await
            .context("Failed to build market order")?;

        // Sign the order
        let signed_order = self
            .client
            .sign(&self.signer, order)
            .await
            .context("Failed to sign market order")?;

        // Post the order
        let response = self
            .client
            .post_order(signed_order)
            .await
            .context("Failed to post market order")?;

        info!(
            "Market order placed successfully | Order ID: {}",
            response.order_id
        );

        Ok(())
    }

    /// Place a market sell order
    async fn place_market_sell(&self, token_id: &str, shares: Decimal) -> Result<()> {
        info!("Placing market sell order: {} shares", shares);

        // Build market sell order
        let order = self
            .client
            .market_order()
            .token_id(token_id)
            .amount(Amount::usdc(shares)?)
            .side(Side::Sell)
            .build()
            .await
            .context("Failed to build market sell order")?;

        // Sign the order
        let signed_order = self
            .client
            .sign(&self.signer, order)
            .await
            .context("Failed to sign market sell order")?;

        // Post the order
        let response = self
            .client
            .post_order(signed_order)
            .await
            .context("Failed to post market sell order")?;

        info!(
            "Market sell order placed successfully | Order ID: {}",
            response.order_id
        );

        Ok(())
    }

    /// Get trader configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable reference to client (for advanced operations)
    pub fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let market = BtcMarket {
            condition_id: "test_condition".to_string(),
            question: "Test Market".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::minutes(15),
            up_token_id: "up_token".to_string(),
            down_token_id: "down_token".to_string(),
            active: true,
            closed: false,
        };

        let position = Position {
            market: market.clone(),
            side: MarketSide::Up,
            token_id: "up_token".to_string(),
            entry_price: Decimal::new(50, 2), // 0.50
            size_usdc: Decimal::new(5, 0),
            shares: Decimal::new(10, 0),
            entry_time: Utc::now(),
            order_ids: vec![],
            trailing_stop: TrailingStop::new(
                Decimal::new(50, 2),
                MarketSide::Up,
                Decimal::new(5, 2),
                Decimal::new(2, 2),
            ),
        };

        assert_eq!(position.side, MarketSide::Up);
        assert_eq!(position.size_usdc, Decimal::new(5, 0));
    }
}
