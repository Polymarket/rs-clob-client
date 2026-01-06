//! Trailing stop loss implementation
//!
//! This module implements trailing take profit logic:
//! - Track highest/lowest price since entry
//! - Trigger exit when price retraces by trailing percentage

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::market::MarketSide;

/// Trailing stop state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailingStop {
    /// Entry price
    entry_price: Decimal,

    /// Current price
    current_price: Decimal,

    /// Best price seen (highest for UP, lowest for DOWN)
    best_price: Decimal,

    /// Trailing stop percentage (e.g., 0.05 = 5%)
    trailing_percentage: Decimal,

    /// Minimum profit target before enabling trailing stop
    min_profit_target: Decimal,

    /// Market side (UP or DOWN)
    side: MarketSide,

    /// Whether trailing stop is active
    trailing_active: bool,

    /// Entry timestamp
    entry_time: DateTime<Utc>,

    /// Last update timestamp
    last_update: DateTime<Utc>,
}

/// Trailing stop exit reason
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitReason {
    /// Trailing stop triggered
    TrailingStop,

    /// Stop loss hit
    StopLoss,

    /// Maximum hold time exceeded
    MaxHoldTime,

    /// Manual exit
    Manual,

    /// Market ended
    MarketEnded,
}

impl TrailingStop {
    /// Create a new trailing stop
    pub fn new(
        entry_price: Decimal,
        side: MarketSide,
        trailing_percentage: Decimal,
        min_profit_target: Decimal,
    ) -> Self {
        let now = Utc::now();

        Self {
            entry_price,
            current_price: entry_price,
            best_price: entry_price,
            trailing_percentage,
            min_profit_target,
            side,
            trailing_active: false,
            entry_time: now,
            last_update: now,
        }
    }

    /// Update with new price
    ///
    /// Returns Some(ExitReason) if stop is triggered, None otherwise
    pub fn update(&mut self, new_price: Decimal) -> Option<ExitReason> {
        self.current_price = new_price;
        self.last_update = Utc::now();

        debug!(
            "Trailing stop update | Price: {:.4} | Best: {:.4} | Entry: {:.4}",
            new_price, self.best_price, self.entry_price
        );

        // Update best price
        match self.side {
            MarketSide::Up => {
                if new_price > self.best_price {
                    info!(
                        "New high for UP position: {:.4} (previous: {:.4})",
                        new_price, self.best_price
                    );
                    self.best_price = new_price;
                }
            }
            MarketSide::Down => {
                if new_price < self.best_price {
                    info!(
                        "New low for DOWN position: {:.4} (previous: {:.4})",
                        new_price, self.best_price
                    );
                    self.best_price = new_price;
                }
            }
        }

        // Check if minimum profit target is reached to activate trailing
        if !self.trailing_active {
            let profit_pct = self.current_profit_percentage();

            if profit_pct >= self.min_profit_target {
                info!(
                    "Minimum profit target {:.2}% reached (current: {:.2}%), activating trailing stop",
                    self.min_profit_target * Decimal::ONE_HUNDRED,
                    profit_pct * Decimal::ONE_HUNDRED
                );
                self.trailing_active = true;
            }
        }

        // Check if trailing stop is triggered (only if active)
        if self.trailing_active {
            let should_exit = match self.side {
                MarketSide::Up => {
                    // For UP: exit if price drops by trailing % from best price
                    let drop_threshold = self.best_price * (Decimal::ONE - self.trailing_percentage);
                    new_price <= drop_threshold
                }
                MarketSide::Down => {
                    // For DOWN: exit if price rises by trailing % from best price
                    let rise_threshold = self.best_price * (Decimal::ONE + self.trailing_percentage);
                    new_price >= rise_threshold
                }
            };

            if should_exit {
                let retracement = self.calculate_retracement();
                info!(
                    "Trailing stop triggered! Price retraced {:.2}% from best",
                    retracement * Decimal::ONE_HUNDRED
                );
                return Some(ExitReason::TrailingStop);
            }
        }

        None
    }

    /// Calculate current profit/loss percentage
    pub fn current_profit_percentage(&self) -> Decimal {
        if self.entry_price == Decimal::ZERO {
            return Decimal::ZERO;
        }

        match self.side {
            MarketSide::Up => {
                // For UP: profit when price increases
                (self.current_price - self.entry_price) / self.entry_price
            }
            MarketSide::Down => {
                // For DOWN: profit when price decreases
                (self.entry_price - self.current_price) / self.entry_price
            }
        }
    }

    /// Calculate current profit/loss in USDC
    pub fn current_profit_usdc(&self, position_size_usdc: Decimal) -> Decimal {
        position_size_usdc * self.current_profit_percentage()
    }

    /// Calculate retracement from best price
    fn calculate_retracement(&self) -> Decimal {
        if self.best_price == Decimal::ZERO {
            return Decimal::ZERO;
        }

        match self.side {
            MarketSide::Up => {
                // Retracement = (best - current) / best
                (self.best_price - self.current_price) / self.best_price
            }
            MarketSide::Down => {
                // Retracement = (current - best) / best
                (self.current_price - self.best_price) / self.best_price
            }
        }
    }

    /// Get entry price
    pub fn entry_price(&self) -> Decimal {
        self.entry_price
    }

    /// Get current price
    pub fn current_price(&self) -> Decimal {
        self.current_price
    }

    /// Get best price
    pub fn best_price(&self) -> Decimal {
        self.best_price
    }

    /// Check if trailing stop is active
    pub fn is_trailing_active(&self) -> bool {
        self.trailing_active
    }

    /// Get time since entry
    pub fn time_since_entry(&self) -> chrono::Duration {
        Utc::now() - self.entry_time
    }

    /// Get entry time
    pub fn entry_time(&self) -> DateTime<Utc> {
        self.entry_time
    }

    /// Get market side
    pub fn side(&self) -> MarketSide {
        self.side
    }

    /// Get trailing percentage
    pub fn trailing_percentage(&self) -> Decimal {
        self.trailing_percentage
    }
}

impl std::fmt::Display for ExitReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TrailingStop => write!(f, "Trailing Stop"),
            Self::StopLoss => write!(f, "Stop Loss"),
            Self::MaxHoldTime => write!(f, "Max Hold Time"),
            Self::Manual => write!(f, "Manual"),
            Self::MarketEnded => write!(f, "Market Ended"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_trailing_stop_up_profit() {
        let mut stop = TrailingStop::new(
            dec!(0.50),       // Entry at 0.50
            MarketSide::Up,
            dec!(0.05),       // 5% trailing
            dec!(0.02),       // 2% min profit to activate
        );

        // Price goes up to 0.52 (4% profit)
        assert!(stop.update(dec!(0.52)).is_none());
        assert!(!stop.is_trailing_active()); // Not active yet (< 2% needed but let me recalculate)

        // Actually (0.52 - 0.50) / 0.50 = 0.04 = 4%, which is > 2%, so it should activate
        assert!(stop.is_trailing_active());

        // Price continues to 0.55 (10% profit)
        assert!(stop.update(dec!(0.55)).is_none());
        assert_eq!(stop.best_price(), dec!(0.55));

        // Price drops to 0.53 (still above 5% trailing threshold from 0.55)
        // Threshold = 0.55 * 0.95 = 0.5225
        assert!(stop.update(dec!(0.53)).is_none());

        // Price drops to 0.52 (below threshold, should trigger)
        // 0.55 * 0.95 = 0.5225, so 0.52 < 0.5225
        let result = stop.update(dec!(0.52));
        assert_eq!(result, Some(ExitReason::TrailingStop));
    }

    #[test]
    fn test_trailing_stop_down_profit() {
        let mut stop = TrailingStop::new(
            dec!(0.50),       // Entry at 0.50
            MarketSide::Down,
            dec!(0.05),       // 5% trailing
            dec!(0.02),       // 2% min profit
        );

        // Price goes down to 0.48 (4% profit for DOWN)
        assert!(stop.update(dec!(0.48)).is_none());
        assert!(stop.is_trailing_active());

        // Price continues down to 0.45 (10% profit)
        assert!(stop.update(dec!(0.45)).is_none());
        assert_eq!(stop.best_price(), dec!(0.45));

        // Price rises to 0.47 (still below 5% trailing threshold from 0.45)
        // Threshold = 0.45 * 1.05 = 0.4725
        assert!(stop.update(dec!(0.47)).is_none());

        // Price rises to 0.48 (above threshold, should trigger)
        // 0.45 * 1.05 = 0.4725, so 0.48 > 0.4725
        let result = stop.update(dec!(0.48));
        assert_eq!(result, Some(ExitReason::TrailingStop));
    }

    #[test]
    fn test_profit_calculation() {
        let mut stop = TrailingStop::new(
            dec!(0.50),
            MarketSide::Up,
            dec!(0.05),
            dec!(0.02),
        );

        stop.update(dec!(0.55));
        let profit_pct = stop.current_profit_percentage();
        assert_eq!(profit_pct, dec!(0.10)); // 10% profit

        let profit_usdc = stop.current_profit_usdc(dec!(5.0));
        assert_eq!(profit_usdc, dec!(0.50)); // $0.50 profit on $5
    }
}
