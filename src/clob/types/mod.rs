use std::fmt;

use alloy::core::sol;
use alloy::primitives::{Signature, U256};
use bon::Builder;
use rust_decimal_macros::dec;
use serde::ser::{Error as _, SerializeStruct as _};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_repr::Serialize_repr;
use serde_with::{DisplayFromStr, serde_as};
use strum_macros::Display;

use crate::Result;
use crate::auth::ApiKey;
use crate::clob::order_builder::{LOT_SIZE_SCALE, USDC_DECIMALS};
use crate::error::Error;
use crate::types::Decimal;

pub mod request;
pub mod response;

// Re-export RFQ types for convenient access
#[cfg(feature = "rfq")]
pub use request::{
    AcceptRfqQuoteRequest, ApproveRfqOrderRequest, CancelRfqQuoteRequest, CancelRfqRequestRequest,
    CreateRfqQuoteRequest, CreateRfqRequestRequest, RfqQuotesRequest, RfqRequestsRequest,
};
#[cfg(feature = "rfq")]
pub use response::{
    AcceptRfqQuoteResponse, ApproveRfqOrderResponse, CreateRfqQuoteResponse,
    CreateRfqRequestResponse, RfqQuote, RfqRequest,
};

#[non_exhaustive]
#[derive(
    Clone, Debug, Display, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub enum OrderType {
    /// Good 'til Cancelled; If not fully filled, the order rests on the book until it is explicitly
    /// cancelled.
    #[serde(alias = "gtc")]
    GTC,
    /// Fill or Kill; Order is attempted to be filled, in full, immediately. If it cannot be fully
    /// filled, the entire order is cancelled.
    #[default]
    #[serde(alias = "fok")]
    FOK,
    /// Good 'til Date; If not fully filled, the order rests on the book until the specified date.
    #[serde(alias = "gtd")]
    GTD,
    /// Fill and Kill; Order is attempted to be filled, however much is possible, immediately. If
    /// the order cannot be fully filled, the remaining quantity is cancelled.
    #[serde(alias = "fak")]
    FAK,
    /// Unknown order type from the API (captures the raw value for debugging).
    #[serde(untagged)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(
    Clone, Copy, Debug, Display, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
#[repr(u8)]
pub enum Side {
    #[serde(alias = "buy")]
    Buy = 0,
    #[serde(alias = "sell")]
    Sell = 1,
    #[serde(other)]
    Unknown = 255,
}

impl TryFrom<u8> for Side {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Side::Buy),
            1 => Ok(Side::Sell),
            other => Err(Error::validation(format!(
                "Unable to create Side from {other}"
            ))),
        }
    }
}

/// Time interval for price history queries.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Serialize, Deserialize)]
pub enum Interval {
    /// 1 minute
    #[serde(rename = "1m")]
    #[strum(serialize = "1m")]
    OneMinute,
    /// 1 hour
    #[serde(rename = "1h")]
    #[strum(serialize = "1h")]
    OneHour,
    /// 6 hours
    #[serde(rename = "6h")]
    #[strum(serialize = "6h")]
    SixHours,
    /// 1 day
    #[serde(rename = "1d")]
    #[strum(serialize = "1d")]
    OneDay,
    /// 1 week
    #[serde(rename = "1w")]
    #[strum(serialize = "1w")]
    OneWeek,
    /// Maximum available history
    #[serde(rename = "max")]
    #[strum(serialize = "max")]
    Max,
}

/// Time range specification for price history queries.
///
/// The CLOB API requires either an interval or explicit start/end timestamps.
/// This enum enforces that requirement at compile time.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(untagged)]
pub enum TimeRange {
    /// Use a predefined interval (e.g., last day, last week).
    Interval {
        /// The time interval.
        interval: Interval,
    },
    /// Use explicit start and end timestamps.
    #[serde(rename_all = "camelCase")]
    Range {
        /// Start timestamp (Unix seconds).
        start_ts: i64,
        /// End timestamp (Unix seconds).
        end_ts: i64,
    },
}

impl TimeRange {
    /// Create a time range from a predefined interval.
    #[must_use]
    pub const fn from_interval(interval: Interval) -> Self {
        Self::Interval { interval }
    }

    /// Create a time range from explicit timestamps.
    #[must_use]
    pub const fn from_range(start_ts: i64, end_ts: i64) -> Self {
        Self::Range { start_ts, end_ts }
    }
}

impl From<Interval> for TimeRange {
    fn from(interval: Interval) -> Self {
        Self::from_interval(interval)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AmountInner {
    Usdc(Decimal),
    Shares(Decimal),
}

impl AmountInner {
    pub fn as_inner(&self) -> Decimal {
        match self {
            AmountInner::Usdc(d) | AmountInner::Shares(d) => *d,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Amount(pub(crate) AmountInner);

impl Amount {
    pub fn usdc(value: Decimal) -> Result<Amount> {
        let normalized = value.normalize();
        if normalized.scale() > USDC_DECIMALS {
            return Err(Error::validation(format!(
                "Unable to build Amount with {} decimal points, must be <= {USDC_DECIMALS}",
                normalized.scale()
            )));
        }

        Ok(Amount(AmountInner::Usdc(normalized)))
    }

    pub fn shares(value: Decimal) -> Result<Amount> {
        let normalized = value.normalize();
        if normalized.scale() > LOT_SIZE_SCALE {
            return Err(Error::validation(format!(
                "Unable to build Amount with {} decimal points, must be <= {LOT_SIZE_SCALE}",
                normalized.scale()
            )));
        }

        Ok(Amount(AmountInner::Shares(normalized)))
    }

    #[must_use]
    pub fn as_inner(&self) -> Decimal {
        self.0.as_inner()
    }

    #[must_use]
    pub fn is_usdc(&self) -> bool {
        matches!(self.0, AmountInner::Usdc(_))
    }

    #[must_use]
    pub fn is_shares(&self) -> bool {
        matches!(self.0, AmountInner::Shares(_))
    }
}

#[non_exhaustive]
#[derive(
    Clone,
    Copy,
    Display,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize_repr,
    Deserialize,
)]
#[repr(u8)]
pub enum SignatureType {
    #[default]
    Eoa = 0,
    Proxy = 1,
    GnosisSafe = 2,
}

/// RFQ state filter for queries.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RfqState {
    /// Active requests/quotes
    #[default]
    Active,
    /// Inactive requests/quotes
    Inactive,
}

/// Sort field for RFQ queries.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RfqSortBy {
    /// Sort by price
    Price,
    /// Sort by expiry
    Expiry,
    /// Sort by size
    Size,
    /// Sort by creation time (default)
    #[default]
    Created,
}

/// Sort direction for RFQ queries.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RfqSortDir {
    /// Ascending order (default)
    #[default]
    Asc,
    /// Descending order
    Desc,
}

#[non_exhaustive]
#[derive(Clone, Debug, Display, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum OrderStatusType {
    #[serde(alias = "live")]
    Live,
    #[serde(alias = "matched")]
    Matched,
    #[serde(alias = "canceled")]
    Canceled,
    #[serde(alias = "delayed")]
    Delayed,
    #[serde(alias = "unmatched")]
    Unmatched,
    /// Unknown order status type from the API (captures the raw value for debugging).
    #[serde(untagged)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(
    Clone, Debug, Default, Display, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum AssetType {
    #[default]
    Collateral,
    Conditional,
    /// Unknown asset type from the API (captures the raw value for debugging).
    #[serde(untagged)]
    Unknown(String),
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TraderSide {
    Taker,
    Maker,
    /// Unknown trader side from the API (captures the raw value for debugging).
    #[serde(untagged)]
    Unknown(String),
}

/// Represents the maximum number of decimal places for an order's price field
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum TickSize {
    Tenth,
    Hundredth,
    Thousandth,
    TenThousandth,
}

impl fmt::Display for TickSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TickSize::Tenth => "Tenth",
            TickSize::Hundredth => "Hundredth",
            TickSize::Thousandth => "Thousandth",
            TickSize::TenThousandth => "TenThousandth",
        };

        write!(f, "{name}({})", self.as_decimal())
    }
}

impl TickSize {
    #[must_use]
    pub fn as_decimal(&self) -> Decimal {
        match self {
            TickSize::Tenth => dec!(0.1),
            TickSize::Hundredth => dec!(0.01),
            TickSize::Thousandth => dec!(0.001),
            TickSize::TenThousandth => dec!(0.0001),
        }
    }
}

impl From<TickSize> for Decimal {
    fn from(tick_size: TickSize) -> Self {
        tick_size.as_decimal()
    }
}

impl TryFrom<Decimal> for TickSize {
    type Error = Error;

    fn try_from(value: Decimal) -> std::result::Result<Self, Self::Error> {
        match value {
            v if v == dec!(0.1) => Ok(TickSize::Tenth),
            v if v == dec!(0.01) => Ok(TickSize::Hundredth),
            v if v == dec!(0.001) => Ok(TickSize::Thousandth),
            v if v == dec!(0.0001) => Ok(TickSize::TenThousandth),
            other => Err(Error::validation(format!(
                "Unknown tick size: {other}. Expected one of: 0.1, 0.01, 0.001, 0.0001"
            ))),
        }
    }
}

impl PartialEq for TickSize {
    fn eq(&self, other: &Self) -> bool {
        self.as_decimal() == other.as_decimal()
    }
}

impl<'de> Deserialize<'de> for TickSize {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dec = <Decimal as Deserialize>::deserialize(deserializer)?;
        TickSize::try_from(dec).map_err(de::Error::custom)
    }
}

sol! {
    /// Alloy solidity type representing an order in the context of the Polymarket exchange
    ///
    /// <!-- The CLOB expects all `uint256` types, [`U256`], excluding `salt`, to be presented as a
    /// string so we must serialize as Display, which for U256 is lower hex-encoded string.
    /// -->
    #[non_exhaustive]
    #[serde_as]
    #[derive(Serialize, Debug, Default, PartialEq)]
    struct Order {
        #[serde(serialize_with = "ser_salt")]
        uint256 salt;
        address maker;
        address signer;
        address taker;
        #[serde_as(as = "DisplayFromStr")]
        uint256 tokenId;
        #[serde_as(as = "DisplayFromStr")]
        uint256 makerAmount;
        #[serde_as(as = "DisplayFromStr")]
        uint256 takerAmount;
        #[serde_as(as = "DisplayFromStr")]
        uint256 expiration;
        #[serde_as(as = "DisplayFromStr")]
        uint256 nonce;
        #[serde_as(as = "DisplayFromStr")]
        uint256 feeRateBps;
        uint8   side;
        uint8   signatureType;
    }
}

// CLOB expects salt as a JSON number. U256 as an integer will not fit as a JSON number. Since
// we generated the salt as a u64 originally (see `salt_generator`), we can be very confident that
// we can invert the conversion to U256 and return a u64 when serializing.
fn ser_salt<S: Serializer>(value: &U256, serializer: S) -> std::result::Result<S::Ok, S::Error> {
    let v: u64 = value
        .try_into()
        .map_err(|e| S::Error::custom(format!("salt does not fit into u64: {e}")))?;
    serializer.serialize_u64(v)
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Serialize, Builder, PartialEq)]
pub struct SignableOrder {
    pub order: Order,
    pub order_type: OrderType,
    #[serde(rename = "postOnly", skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
}

#[non_exhaustive]
#[derive(Debug, Builder, PartialEq)]
pub struct SignedOrder {
    pub order: Order,
    pub signature: Signature,
    pub order_type: OrderType,
    pub owner: ApiKey,
    pub post_only: Option<bool>,
}

/// Helper struct for serializing Order with signature injected.
/// This avoids the overhead of `serde_json::to_value()` followed by mutation.
#[serde_as]
#[derive(Serialize)]
struct OrderWithSignature<'order> {
    #[serde(serialize_with = "ser_salt")]
    salt: &'order U256,
    maker: &'order alloy::primitives::Address,
    signer: &'order alloy::primitives::Address,
    taker: &'order alloy::primitives::Address,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "tokenId")]
    token_id: &'order U256,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "makerAmount")]
    maker_amount: &'order U256,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "takerAmount")]
    taker_amount: &'order U256,
    #[serde_as(as = "DisplayFromStr")]
    expiration: &'order U256,
    #[serde_as(as = "DisplayFromStr")]
    nonce: &'order U256,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "feeRateBps")]
    fee_rate_bps: &'order U256,
    /// Side serialized as "BUY"/"SELL" string (CLOB API requirement)
    side: Side,
    #[serde(rename = "signatureType")]
    signature_type: u8,
    /// Signature injected into the order object
    signature: String,
}

// CLOB expects a struct that has the `signature` "folded" into the `order` key
impl Serialize for SignedOrder {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let len = if self.post_only.is_some() { 4 } else { 3 };
        let mut st = serializer.serialize_struct("SignedOrder", len)?;

        // Convert numeric side to Side enum for string serialization
        let side = Side::try_from(self.order.side).map_err(S::Error::custom)?;

        // Serialize order directly with signature injected, avoiding intermediate JSON tree
        let order_with_sig = OrderWithSignature {
            salt: &self.order.salt,
            maker: &self.order.maker,
            signer: &self.order.signer,
            taker: &self.order.taker,
            token_id: &self.order.tokenId,
            maker_amount: &self.order.makerAmount,
            taker_amount: &self.order.takerAmount,
            expiration: &self.order.expiration,
            nonce: &self.order.nonce,
            fee_rate_bps: &self.order.feeRateBps,
            side,
            signature_type: self.order.signatureType,
            signature: self.signature.to_string(),
        };

        st.serialize_field("order", &order_with_sig)?;
        st.serialize_field("orderType", &self.order_type)?;
        st.serialize_field("owner", &self.owner)?;
        if let Some(post_only) = self.post_only {
            st.serialize_field("postOnly", &post_only)?;
        }

        st.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::to_value;

    use super::*;
    use crate::error::Validation;

    #[test]
    fn tick_size_decimals_should_succeed() {
        assert_eq!(TickSize::Tenth.as_decimal().scale(), 1);
        assert_eq!(TickSize::Hundredth.as_decimal().scale(), 2);
        assert_eq!(TickSize::Thousandth.as_decimal().scale(), 3);
        assert_eq!(TickSize::TenThousandth.as_decimal().scale(), 4);
    }

    #[test]
    fn tick_size_should_display() {
        assert_eq!(format!("{}", TickSize::Tenth), "Tenth(0.1)");
        assert_eq!(format!("{}", TickSize::Hundredth), "Hundredth(0.01)");
        assert_eq!(format!("{}", TickSize::Thousandth), "Thousandth(0.001)");
        assert_eq!(
            format!("{}", TickSize::TenThousandth),
            "TenThousandth(0.0001)"
        );
    }

    #[test]
    fn tick_from_decimal_should_succeed() {
        assert_eq!(
            TickSize::try_from(dec!(0.0001)).unwrap(),
            TickSize::TenThousandth
        );
        assert_eq!(
            TickSize::try_from(dec!(0.001)).unwrap(),
            TickSize::Thousandth
        );
        assert_eq!(TickSize::try_from(dec!(0.01)).unwrap(), TickSize::Hundredth);
        assert_eq!(TickSize::try_from(dec!(0.1)).unwrap(), TickSize::Tenth);
    }

    #[test]
    fn non_standard_decimal_to_tick_size_should_fail() {
        let result = TickSize::try_from(Decimal::ONE);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unknown tick size: 1")
        );
    }

    #[test]
    fn amount_should_succeed() -> Result<()> {
        let usdc = Amount::usdc(Decimal::ONE_HUNDRED)?;
        assert!(usdc.is_usdc());
        assert_eq!(usdc.as_inner(), Decimal::ONE_HUNDRED);

        let shares = Amount::shares(Decimal::ONE_HUNDRED)?;
        assert!(shares.is_shares());
        assert_eq!(shares.as_inner(), Decimal::ONE_HUNDRED);

        Ok(())
    }

    #[test]
    fn improper_shares_lot_size_should_fail() {
        let Err(err) = Amount::shares(dec!(0.23400)) else {
            panic!()
        };

        let message = err.downcast_ref::<Validation>().unwrap();
        assert_eq!(
            message.reason,
            format!("Unable to build Amount with 3 decimal points, must be <= {LOT_SIZE_SCALE}")
        );
    }

    #[test]
    fn improper_usdc_decimal_size_should_fail() {
        let Err(err) = Amount::usdc(dec!(0.2340011)) else {
            panic!()
        };

        let message = err.downcast_ref::<Validation>().unwrap();
        assert_eq!(
            message.reason,
            format!("Unable to build Amount with 7 decimal points, must be <= {USDC_DECIMALS}")
        );
    }

    #[test]
    fn side_to_string_should_succeed() {
        assert_eq!(Side::Buy.to_string(), "BUY");
        assert_eq!(Side::Sell.to_string(), "SELL");
    }

    #[test]
    fn order_type_deserialize_known_variants() {
        // Test that known variants still deserialize correctly
        assert_eq!(
            serde_json::from_str::<OrderType>(r#""GTC""#).unwrap(),
            OrderType::GTC
        );
        assert_eq!(
            serde_json::from_str::<OrderType>(r#""gtc""#).unwrap(),
            OrderType::GTC
        );
        assert_eq!(
            serde_json::from_str::<OrderType>(r#""FOK""#).unwrap(),
            OrderType::FOK
        );
    }

    #[test]
    fn order_type_deserialize_unknown_variant() {
        // Test that unknown variants are captured
        let result = serde_json::from_str::<OrderType>(r#""NEW_ORDER_TYPE""#).unwrap();
        assert_eq!(result, OrderType::Unknown("NEW_ORDER_TYPE".to_owned()));
    }

    #[test]
    fn order_status_type_deserialize_known_variants() {
        assert_eq!(
            serde_json::from_str::<OrderStatusType>(r#""LIVE""#).unwrap(),
            OrderStatusType::Live
        );
        assert_eq!(
            serde_json::from_str::<OrderStatusType>(r#""live""#).unwrap(),
            OrderStatusType::Live
        );
    }

    #[test]
    fn order_status_type_deserialize_unknown_variant() {
        let result = serde_json::from_str::<OrderStatusType>(r#""NEW_STATUS""#).unwrap();
        assert_eq!(result, OrderStatusType::Unknown("NEW_STATUS".to_owned()));
    }

    #[test]
    fn order_type_display_known_variants() {
        assert_eq!(format!("{}", OrderType::GTC), "GTC");
        assert_eq!(format!("{}", OrderType::FOK), "FOK");
    }

    #[test]
    fn order_type_display_unknown_variant() {
        // strum Display will show the variant name + contents for tuple variants
        let unknown = OrderType::Unknown("NEW_TYPE".to_owned());
        let display = format!("{unknown}");
        // Just verify it displays something reasonable (contains the inner value)
        assert!(display.contains("Unknown") || display.contains("NEW_TYPE"));
    }

    #[test]
    fn signed_order_serialization_omits_post_only_when_none() {
        let signed_order = SignedOrder {
            order: Order::default(),
            signature: Signature::new(U256::ZERO, U256::ZERO, false),
            order_type: OrderType::GTC,
            owner: ApiKey::nil(),
            post_only: None,
        };

        let value = to_value(&signed_order).expect("serialize SignedOrder");
        let object = value
            .as_object()
            .expect("SignedOrder should serialize to an object");

        assert!(!object.contains_key("postOnly"));
    }
}
