use std::sync::atomic::{AtomicU8, Ordering};

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MessageInterest: u8 {
        /// No interest in any message types.
        const NONE = 0;

        /// Interest in Binance crypto price updates.
        const CRYPTO_PRICES = 1;

        /// Interest in Chainlink price feed updates.
        const CHAINLINK_PRICES = 1 << 1;

        /// Interest in comment events.
        const COMMENTS = 1 << 2;

        /// Interest in all RTDS message types.
        const ALL = Self::CRYPTO_PRICES.bits()
            | Self::CHAINLINK_PRICES.bits()
            | Self::COMMENTS.bits();
    }
}

impl MessageInterest {
    /// Get the interest flag for a given topic string.
    #[must_use]
    pub fn from_topic(topic: &str) -> Self {
        match topic {
            "crypto_prices" => Self::CRYPTO_PRICES,
            "crypto_prices_chainlink" => Self::CHAINLINK_PRICES,
            "comments" => Self::COMMENTS,
            _ => Self::NONE,
        }
    }

    /// Check if interested in messages from a given topic.
    #[must_use]
    pub fn is_interested_in_topic(&self, topic: &str) -> bool {
        let interest = Self::from_topic(topic);
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

    /// Check if there's interest in messages from a given topic.
    #[must_use]
    pub fn is_interested_in_topic(&self, topic: &str) -> bool {
        let interest = MessageInterest::from_topic(topic);
        !interest.is_empty() && self.is_interested(interest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interest_contains() {
        assert!(MessageInterest::ALL.contains(MessageInterest::CRYPTO_PRICES));
        assert!(MessageInterest::ALL.contains(MessageInterest::CHAINLINK_PRICES));
        assert!(MessageInterest::ALL.contains(MessageInterest::COMMENTS));
        assert!(!MessageInterest::CRYPTO_PRICES.contains(MessageInterest::COMMENTS));
    }

    #[test]
    fn interest_from_topic() {
        assert_eq!(
            MessageInterest::from_topic("crypto_prices"),
            MessageInterest::CRYPTO_PRICES
        );
        assert_eq!(
            MessageInterest::from_topic("crypto_prices_chainlink"),
            MessageInterest::CHAINLINK_PRICES
        );
        assert_eq!(
            MessageInterest::from_topic("comments"),
            MessageInterest::COMMENTS
        );
        assert_eq!(
            MessageInterest::from_topic("unknown"),
            MessageInterest::NONE
        );
    }

    #[test]
    fn tracker_add_and_get() {
        let tracker = InterestTracker::new();
        assert!(tracker.get().is_empty());

        tracker.add(MessageInterest::CRYPTO_PRICES);
        assert!(tracker.is_interested(MessageInterest::CRYPTO_PRICES));
        assert!(!tracker.is_interested(MessageInterest::COMMENTS));

        tracker.add(MessageInterest::COMMENTS);
        assert!(tracker.is_interested(MessageInterest::CRYPTO_PRICES));
        assert!(tracker.is_interested(MessageInterest::COMMENTS));
    }

    #[test]
    fn tracker_is_interested_in_topic() {
        let tracker = InterestTracker::new();
        tracker.add(MessageInterest::CRYPTO_PRICES);

        assert!(tracker.is_interested_in_topic("crypto_prices"));
        assert!(!tracker.is_interested_in_topic("comments"));
        assert!(!tracker.is_interested_in_topic("unknown"));
    }
}
