use std::error::Error as StdError;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Address(String);

impl Address {
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

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AddressError {
    MissingPrefix,
    InvalidLength(usize),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hash64(String);

impl Hash64 {
    pub fn new<S: Into<String>>(s: S) -> Result<Self, Hash64Error> {
        let s = s.into();
        if !s.starts_with("0x") {
            return Err(Hash64Error::MissingPrefix);
        }
        if s.len() != 66 {
            return Err(Hash64Error::InvalidLength(s.len()));
        }
        if !s
            .get(2..)
            .is_some_and(|hex| hex.chars().all(|c| c.is_ascii_hexdigit()))
        {
            return Err(Hash64Error::InvalidHex);
        }
        Ok(Self(s.to_lowercase()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Hash64Error {
    MissingPrefix,
    InvalidLength(usize),
    InvalidHex,
}

impl fmt::Display for Hash64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPrefix => write!(f, "hash must start with 0x"),
            Self::InvalidLength(len) => write!(f, "hash must be 66 characters (got {len})"),
            Self::InvalidHex => write!(f, "hash must contain only hex characters"),
        }
    }
}

impl StdError for Hash64Error {}

impl TryFrom<String> for Hash64 {
    type Error = Hash64Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl From<Hash64> for String {
    fn from(h: Hash64) -> Self {
        h.0
    }
}

impl fmt::Display for Hash64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct EventId(u64);

impl EventId {
    pub fn new(value: u64) -> Result<Self, EventIdError> {
        if value == 0 {
            Err(EventIdError(value))
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub struct EventIdError(u64);

impl fmt::Display for EventIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "event ID must be >= 1 (got {})", self.0)
    }
}

impl StdError for EventIdError {}

impl TryFrom<u64> for EventId {
    type Error = EventIdError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<EventId> for u64 {
    fn from(e: EventId) -> Self {
        e.0
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum ActivityType {
    Trade,
    Split,
    Merge,
    Redeem,
    Reward,
    Conversion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[non_exhaustive]
pub enum PositionSortBy {
    #[serde(rename = "CURRENT")]
    #[strum(serialize = "CURRENT")]
    Current,
    #[serde(rename = "INITIAL")]
    #[strum(serialize = "INITIAL")]
    Initial,
    #[serde(rename = "TOKENS")]
    #[strum(serialize = "TOKENS")]
    Tokens,
    #[serde(rename = "CASHPNL")]
    #[strum(serialize = "CASHPNL")]
    CashPnl,
    #[serde(rename = "PERCENTPNL")]
    #[strum(serialize = "PERCENTPNL")]
    PercentPnl,
    #[serde(rename = "TITLE")]
    #[strum(serialize = "TITLE")]
    Title,
    #[serde(rename = "RESOLVING")]
    #[strum(serialize = "RESOLVING")]
    Resolving,
    #[serde(rename = "PRICE")]
    #[strum(serialize = "PRICE")]
    Price,
    #[serde(rename = "AVGPRICE")]
    #[strum(serialize = "AVGPRICE")]
    AvgPrice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[non_exhaustive]
pub enum ClosedPositionSortBy {
    #[serde(rename = "REALIZEDPNL")]
    #[strum(serialize = "REALIZEDPNL")]
    RealizedPnl,
    #[serde(rename = "TITLE")]
    #[strum(serialize = "TITLE")]
    Title,
    #[serde(rename = "PRICE")]
    #[strum(serialize = "PRICE")]
    Price,
    #[serde(rename = "AVGPRICE")]
    #[strum(serialize = "AVGPRICE")]
    AvgPrice,
    #[serde(rename = "TIMESTAMP")]
    #[strum(serialize = "TIMESTAMP")]
    Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum ActivitySortBy {
    Timestamp,
    Tokens,
    Cash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum FilterType {
    Cash,
    Tokens,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum TimePeriod {
    Day,
    Week,
    Month,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum LeaderboardCategory {
    Overall,
    Politics,
    Sports,
    Crypto,
    Culture,
    Mentions,
    Weather,
    Economics,
    Tech,
    Finance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[non_exhaustive]
pub enum LeaderboardOrderBy {
    Pnl,
    Vol,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MarketFilter {
    Markets(Vec<Hash64>),
    EventIds(Vec<EventId>),
}

impl MarketFilter {
    #[must_use]
    pub fn markets<I: IntoIterator<Item = Hash64>>(ids: I) -> Self {
        Self::Markets(ids.into_iter().collect())
    }

    #[must_use]
    pub fn event_ids<I: IntoIterator<Item = EventId>>(ids: I) -> Self {
        Self::EventIds(ids.into_iter().collect())
    }

    pub(crate) fn append_to_params(&self, params: &mut Vec<(&'static str, String)>) {
        match self {
            Self::Markets(ids) => {
                if !ids.is_empty() {
                    let s = ids.iter().map(Hash64::as_str).collect::<Vec<_>>().join(",");
                    params.push(("market", s));
                }
            }
            Self::EventIds(ids) => {
                if !ids.is_empty() {
                    let s = ids
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    params.push(("eventId", s));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct BoundedIntError {
    value: u32,
    min: u32,
    max: u32,
    type_name: &'static str,
}

impl fmt::Display for BoundedIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} must be between {} and {} (got {})",
            self.type_name, self.min, self.max, self.value
        )
    }
}

impl StdError for BoundedIntError {}

macro_rules! bounded_u32 {
    ($name:ident, min = $min:expr, max = $max:expr, default = $default:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "u32", into = "u32")]
        pub struct $name(u32);

        impl $name {
            pub const MIN: u32 = $min;
            pub const MAX: u32 = $max;
            pub const DEFAULT: u32 = $default;

            pub fn new(value: u32) -> Result<Self, BoundedIntError> {
                if (Self::MIN..=Self::MAX).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(BoundedIntError {
                        value,
                        min: Self::MIN,
                        max: Self::MAX,
                        type_name: stringify!($name),
                    })
                }
            }

            #[must_use]
            pub fn value(self) -> u32 {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(Self::DEFAULT)
            }
        }

        impl TryFrom<u32> for $name {
            type Error = BoundedIntError;
            fn try_from(value: u32) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for u32 {
            fn from(b: $name) -> Self {
                b.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

bounded_u32!(PositionsLimit, min = 0, max = 500, default = 100);
bounded_u32!(PositionsOffset, min = 0, max = 10000, default = 0);
bounded_u32!(TradesLimit, min = 0, max = 10000, default = 100);
bounded_u32!(TradesOffset, min = 0, max = 10000, default = 0);
bounded_u32!(ActivityLimit, min = 0, max = 500, default = 100);
bounded_u32!(ActivityOffset, min = 0, max = 10000, default = 0);
bounded_u32!(HoldersLimit, min = 0, max = 20, default = 20);
bounded_u32!(HoldersMinBalance, min = 0, max = 999_999, default = 1);
bounded_u32!(ClosedPositionsLimit, min = 0, max = 50, default = 10);
bounded_u32!(ClosedPositionsOffset, min = 0, max = 100_000, default = 0);
bounded_u32!(BuilderLeaderboardLimit, min = 0, max = 50, default = 25);
bounded_u32!(BuilderLeaderboardOffset, min = 0, max = 1000, default = 0);
bounded_u32!(TraderLeaderboardLimit, min = 1, max = 50, default = 25);
bounded_u32!(TraderLeaderboardOffset, min = 0, max = 1000, default = 0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Title(String);

impl Title {
    pub const MAX_LEN: usize = 100;

    pub fn new<S: Into<String>>(s: S) -> Result<Self, TitleError> {
        let s = s.into();
        if s.len() > Self::MAX_LEN {
            Err(TitleError(s.len()))
        } else {
            Ok(Self(s))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct TitleError(usize);

impl fmt::Display for TitleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "title must be at most 100 characters (got {})", self.0)
    }
}

impl StdError for TitleError {}

impl TryFrom<String> for Title {
    type Error = TitleError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl From<Title> for String {
    fn from(t: Title) -> Self {
        t.0
    }
}

impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TradeFilter {
    pub filter_type: FilterType,
    pub filter_amount: f64,
}

impl TradeFilter {
    pub fn new(filter_type: FilterType, filter_amount: f64) -> Result<Self, TradeFilterError> {
        if filter_amount < 0.0 {
            return Err(TradeFilterError::NegativeAmount(filter_amount));
        }
        Ok(Self {
            filter_type,
            filter_amount,
        })
    }

    pub fn cash(amount: f64) -> Result<Self, TradeFilterError> {
        Self::new(FilterType::Cash, amount)
    }

    pub fn tokens(amount: f64) -> Result<Self, TradeFilterError> {
        Self::new(FilterType::Tokens, amount)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TradeFilterError {
    NegativeAmount(f64),
}

impl fmt::Display for TradeFilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeAmount(amount) => {
                write!(f, "filter amount must be >= 0 (got {amount})")
            }
        }
    }
}

impl StdError for TradeFilterError {}
