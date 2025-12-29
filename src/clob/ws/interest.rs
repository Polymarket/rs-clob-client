use std::sync::atomic::{AtomicU8, Ordering};

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MessageInterest: u8 {
        /// No interest in any message types.
        const NONE = 0;

        /// Interest in orderbook updates.
        const BOOK = 1;

        /// Interest in price change notifications.
        const PRICE_CHANGE = 1 << 1;

        /// Interest in tick size changes.
        const TICK_SIZE = 1 << 2;

        /// Interest in last trade price updates.
        const LAST_TRADE_PRICE = 1 << 3;

        /// Interest in trade executions.
        const TRADE = 1 << 4;

        /// Interest in order updates.
        const ORDER = 1 << 5;

        /// Interest in all market data messages.
        const MARKET = Self::BOOK.bits()
            | Self::PRICE_CHANGE.bits()
            | Self::TICK_SIZE.bits()
            | Self::LAST_TRADE_PRICE.bits();

        /// Interest in all user channel messages.
        const USER = Self::TRADE.bits() | Self::ORDER.bits();

        /// Interest in all message types.
        const ALL = Self::MARKET.bits() | Self::USER.bits();
    }
}

impl MessageInterest {
    /// Get the interest flag for a given event type string.
    #[must_use]
    pub fn from_event_type(event_type: &str) -> Self {
        match event_type {
            "book" => Self::BOOK,
            "price_change" => Self::PRICE_CHANGE,
            "tick_size_change" => Self::TICK_SIZE,
            "last_trade_price" => Self::LAST_TRADE_PRICE,
            "trade" => Self::TRADE,
            "order" => Self::ORDER,
            _ => Self::NONE,
        }
    }

    #[must_use]
    pub fn is_interested_in_event(&self, event_type: &str) -> bool {
        let interest = MessageInterest::from_event_type(event_type);
        !interest.is_empty() && self.contains(interest)
    }
}

impl Default for MessageInterest {
    fn default() -> Self {
        Self::ALL
    }
}

/// Thread-safe interest tracker that can be shared between subscription manager and connection.
#[derive(Debug, Default)]
pub struct InterestTracker {
    interest: AtomicU8,
}

impl InterestTracker {
    /// Create a new tracker with no interest.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            interest: AtomicU8::new(0),
        }
    }

    /// Add interest in specific message types.
    pub fn add(&self, interest: MessageInterest) {
        self.interest.fetch_or(interest.bits(), Ordering::Release);
    }

    /// Get the current interest set.
    #[must_use]
    pub fn get(&self) -> MessageInterest {
        MessageInterest::from_bits(self.interest.load(Ordering::Acquire))
            .unwrap_or(MessageInterest::NONE)
    }

    /// Check if there's interest in a specific message type.
    #[must_use]
    pub fn is_interested(&self, interest: MessageInterest) -> bool {
        self.get().contains(interest)
    }

    /// Check if there's interest in a message with the given event type.
    #[must_use]
    pub fn is_interested_in_event(&self, event_type: &str) -> bool {
        let interest = MessageInterest::from_event_type(event_type);
        !interest.is_empty() && self.is_interested(interest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interest_contains() {
        assert!(MessageInterest::MARKET.contains(MessageInterest::BOOK));
        assert!(MessageInterest::MARKET.contains(MessageInterest::PRICE_CHANGE));
        assert!(!MessageInterest::MARKET.contains(MessageInterest::TRADE));
        assert!(MessageInterest::ALL.contains(MessageInterest::TRADE));
    }

    #[test]
    fn interest_from_event_type() {
        assert_eq!(
            MessageInterest::from_event_type("book"),
            MessageInterest::BOOK
        );
        assert_eq!(
            MessageInterest::from_event_type("trade"),
            MessageInterest::TRADE
        );
        assert_eq!(
            MessageInterest::from_event_type("unknown"),
            MessageInterest::NONE
        );
    }

    #[test]
    fn tracker_add_and_get() {
        let tracker = InterestTracker::new();
        assert!(tracker.get().is_empty());

        tracker.add(MessageInterest::BOOK);
        assert!(tracker.is_interested(MessageInterest::BOOK));
        assert!(!tracker.is_interested(MessageInterest::TRADE));

        tracker.add(MessageInterest::TRADE);
        assert!(tracker.is_interested(MessageInterest::BOOK));
        assert!(tracker.is_interested(MessageInterest::TRADE));
    }
}
