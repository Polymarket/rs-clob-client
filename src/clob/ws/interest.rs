use std::sync::atomic::{AtomicU8, Ordering};

/// Flags representing interest in specific WebSocket message types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageInterest(u8);

impl MessageInterest {
    /// No interest in any message types.
    pub const NONE: Self = Self(0);

    /// Interest in orderbook updates.
    pub const BOOK: Self = Self(1 << 0);

    /// Interest in price change notifications.
    pub const PRICE_CHANGE: Self = Self(1 << 1);

    /// Interest in tick size changes.
    pub const TICK_SIZE: Self = Self(1 << 2);

    /// Interest in last trade price updates.
    pub const LAST_TRADE_PRICE: Self = Self(1 << 3);

    /// Interest in trade executions.
    pub const TRADE: Self = Self(1 << 4);

    /// Interest in order updates.
    pub const ORDER: Self = Self(1 << 5);

    /// Interest in all market data messages.
    pub const MARKET: Self =
        Self(Self::BOOK.0 | Self::PRICE_CHANGE.0 | Self::TICK_SIZE.0 | Self::LAST_TRADE_PRICE.0);

    /// Interest in all user channel messages.
    pub const USER: Self = Self(Self::TRADE.0 | Self::ORDER.0);

    /// Interest in all message types.
    pub const ALL: Self = Self(Self::MARKET.0 | Self::USER.0);

    /// Check if this interest set contains a specific interest.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Combine two interest sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Check if any interest is set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

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

impl std::ops::BitOr for MessageInterest {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for MessageInterest {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for MessageInterest {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
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
        self.interest.fetch_or(interest.0, Ordering::Release);
    }

    /// Get the current interest set.
    #[must_use]
    pub fn get(&self) -> MessageInterest {
        MessageInterest(self.interest.load(Ordering::Acquire))
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
