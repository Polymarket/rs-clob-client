use std::sync::atomic::{AtomicU8, Ordering};

/// Flags representing interest in specific RTDS message types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageInterest(u8);

impl MessageInterest {
    /// No interest in any message types.
    pub const NONE: Self = Self(0);

    /// Interest in Binance crypto price updates.
    pub const CRYPTO_PRICES: Self = Self(1 << 0);

    /// Interest in Chainlink price feed updates.
    pub const CHAINLINK_PRICES: Self = Self(1 << 1);

    /// Interest in comment events.
    pub const COMMENTS: Self = Self(1 << 2);

    /// Interest in all RTDS message types.
    pub const ALL: Self = Self(Self::CRYPTO_PRICES.0 | Self::CHAINLINK_PRICES.0 | Self::COMMENTS.0);

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
