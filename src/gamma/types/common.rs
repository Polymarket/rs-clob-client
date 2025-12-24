//! Common types used across the Gamma API.
//!
//! This module contains fundamental types, enums, and validated wrappers
//! that are shared across request and response types.

use std::error::Error as StdError;
use std::fmt;

use serde::{Deserialize, Serialize};

/// An Ethereum address.
///
/// Addresses are 0x-prefixed, 40 hex character strings (20 bytes).
/// They are stored in lowercase and validated on construction.
///
/// # Example
///
/// ```
/// use polymarket_client_sdk::gamma::types::Address;
///
/// let addr = Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap();
/// assert_eq!(addr.as_str(), "0x56687bf447db6ffa42ffe2204a05edaa20f55839");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Address(String);

impl Address {
    /// Creates a new validated Ethereum address.
    ///
    /// # Errors
    ///
    /// Returns [`AddressError`] if the string is not a valid Ethereum address.
    pub fn new<S: Into<String>>(s: S) -> Result<Self, AddressError> {
        let s = s.into();
        if !s.starts_with("0x") {
            return Err(AddressError::MissingPrefix);
        }
        if s.len() != 42 {
            return Err(AddressError::InvalidLength(s.len()));
        }
        if !s
            .get(2..)
            .is_some_and(|hex| hex.chars().all(|c| c.is_ascii_hexdigit()))
        {
            return Err(AddressError::InvalidHex);
        }
        Ok(Self(s.to_lowercase()))
    }

    /// Returns the address as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Error type for invalid Ethereum addresses.
#[derive(Debug)]
#[non_exhaustive]
pub enum AddressError {
    /// The address is missing the `0x` prefix.
    MissingPrefix,
    /// The address has an invalid length (expected 42 characters).
    InvalidLength(usize),
    /// The address contains non-hexadecimal characters.
    InvalidHex,
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPrefix => write!(f, "address must start with 0x"),
            Self::InvalidLength(len) => write!(f, "address must be 42 characters (got {len})"),
            Self::InvalidHex => write!(f, "address must contain only hex characters"),
        }
    }
}

impl StdError for AddressError {}

impl TryFrom<String> for Address {
    type Error = AddressError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl From<Address> for String {
    fn from(a: Address) -> Self {
        a.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status filter for related tags queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[non_exhaustive]
pub enum RelatedTagsStatus {
    /// Only active (open) markets.
    Active,
    /// Only closed markets.
    Closed,
    /// All markets regardless of status.
    All,
}

/// Parent entity type for comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[non_exhaustive]
pub enum ParentEntityType {
    /// Event entity.
    Event,
    /// Series entity.
    Series,
    /// Market entity (lowercase in API).
    #[serde(rename = "market")]
    #[strum(serialize = "market")]
    Market,
}

/// Helper function to join array items for query parameters.
pub(crate) fn join_array<T: fmt::Display>(items: &[T]) -> String {
    items
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
