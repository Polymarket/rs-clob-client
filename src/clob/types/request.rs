#![allow(
    clippy::module_name_repetitions,
    reason = "Request suffix is intentional for clarity"
)]

use bon::Builder;
use chrono::NaiveDate;
use serde::{Serialize, Serializer};
use serde_with::{
    DisplayFromStr, StringWithSeparator, formats::CommaSeparator, serde_as, skip_serializing_none,
};
#[cfg(feature = "rfq")]
use {
    crate::clob::types::{RfqSortBy, RfqSortDir, RfqState},
    crate::{Timestamp, auth::ApiKey, types::Decimal},
};

use crate::clob::types::{AssetType, Side, SignatureType, TimeRange};
use crate::types::U256;
use crate::types::{Address, B256};

#[serde_as]
#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct MidpointRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
}

#[serde_as]
#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct PriceRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
    pub side: Side,
}

#[non_exhaustive]
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct SpreadRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
    pub side: Option<Side>,
}

#[non_exhaustive]
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct OrderBookSummaryRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
    pub side: Option<Side>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct LastTradePriceRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
}

#[non_exhaustive]
#[skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct PriceHistoryRequest {
    /// The market condition ID.
    pub market: B256,
    /// The time range for the price history query.
    /// Either a predefined interval or explicit start/end timestamps.
    #[serde(flatten)]
    #[builder(into)]
    pub time_range: TimeRange,
    /// Optional fidelity (number of data points).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fidelity: Option<u32>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(on(String, into))]
pub struct CancelMarketOrderRequest {
    /// The market condition ID to cancel orders for.
    pub market: Option<B256>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub asset_id: Option<U256>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Clone, Builder, Serialize)]
#[builder(on(String, into))]
pub struct TradesRequest {
    pub id: Option<String>,
    #[serde(rename = "taker")]
    pub taker_address: Option<Address>,
    #[serde(rename = "maker")]
    pub maker_address: Option<Address>,
    /// The market condition ID to filter trades.
    pub market: Option<B256>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub asset_id: Option<U256>,
    pub before: Option<i64>,
    pub after: Option<i64>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(on(String, into))]
pub struct OrdersRequest {
    #[serde(rename = "id")]
    pub order_id: Option<String>,
    /// The market condition ID to filter orders.
    pub market: Option<B256>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub asset_id: Option<U256>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Serialize, Builder)]
pub struct DeleteNotificationsRequest {
    #[serde(rename = "ids", skip_serializing_if = "Vec::is_empty")]
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    #[builder(default)]
    pub notification_ids: Vec<String>,
}

#[non_exhaustive]
#[serde_as]
#[derive(Debug, Default, Clone, Builder, Serialize)]
#[builder(on(String, into))]
pub struct BalanceAllowanceRequest {
    pub asset_type: AssetType,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub token_id: Option<U256>,
    pub signature_type: Option<SignatureType>,
}

pub type UpdateBalanceAllowanceRequest = BalanceAllowanceRequest;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(on(String, into))]
pub struct UserRewardsEarningRequest {
    pub date: NaiveDate,
    #[builder(default)]
    pub order_by: String,
    #[builder(default)]
    pub position: String,
    #[builder(default)]
    pub no_competition: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Asset {
    Usdc,
    Asset(U256),
}

impl Serialize for Asset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Asset::Usdc => serializer.serialize_str("0"),
            Asset::Asset(a) => serializer.collect_str(a),
        }
    }
}

/// Request body for creating an RFQ request.
///
/// Creates an RFQ Request to buy or sell outcome tokens.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct CreateRfqRequestRequest {
    /// Token ID the Requester wants to receive. "0" indicates USDC.
    pub asset_in: Asset,
    /// Token ID the Requester wants to give. "0" indicates USDC.
    pub asset_out: Asset,
    /// Amount of asset to receive (in base units).
    pub amount_in: Decimal,
    /// Amount of asset to give (in base units).
    pub amount_out: Decimal,
    /// Signature type (`EOA`, `Proxy`, or `GnosisSafe`).
    pub user_type: SignatureType,
}

/// Request body for canceling an RFQ request.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CancelRfqRequestRequest {
    /// ID of the request to cancel.
    pub request_id: String,
}

/// Query parameters for getting RFQ requests.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct RfqRequestsRequest {
    /// Cursor offset for pagination (base64 encoded).
    pub offset: Option<String>,
    /// Max requests to return. Defaults to 50, max 1000.
    pub limit: Option<u32>,
    /// Filter by state (active or inactive).
    pub state: Option<RfqState>,
    /// Filter by request IDs.
    #[serde(rename = "requestIds", skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub request_ids: Vec<String>,
    /// Filter by market condition IDs.
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, B256>")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub markets: Vec<B256>,
    /// Minimum size in tokens.
    pub size_min: Option<Decimal>,
    /// Maximum size in tokens.
    pub size_max: Option<Decimal>,
    /// Minimum size in USDC.
    pub size_usdc_min: Option<Decimal>,
    /// Maximum size in USDC.
    pub size_usdc_max: Option<Decimal>,
    /// Minimum price.
    pub price_min: Option<Decimal>,
    /// Maximum price.
    pub price_max: Option<Decimal>,
    /// Sort field.
    pub sort_by: Option<RfqSortBy>,
    /// Sort direction.
    pub sort_dir: Option<RfqSortDir>,
}

/// Request body for creating an RFQ quote.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CreateRfqQuoteRequest {
    /// ID of the Request to quote.
    pub request_id: String,
    /// Token ID the Quoter wants to receive. "0" indicates USDC.
    pub asset_in: Asset,
    /// Token ID the Quoter wants to give. "0" indicates USDC.
    pub asset_out: Asset,
    /// Amount of asset to receive (in base units).
    pub amount_in: Decimal,
    /// Amount of asset to give (in base units).
    pub amount_out: Decimal,
    /// Signature type (`EOA`, `Proxy`, or `GnosisSafe`).
    pub user_type: SignatureType,
}

/// Request body for canceling an RFQ quote.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct CancelRfqQuoteRequest {
    /// ID of the quote to cancel.
    pub quote_id: String,
}

/// Query parameters for getting RFQ quotes.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct RfqQuotesRequest {
    /// Cursor offset for pagination (base64 encoded).
    pub offset: Option<String>,
    /// Max quotes to return. Defaults to 50, max 1000.
    pub limit: Option<u32>,
    /// Filter by state (active or inactive).
    pub state: Option<RfqState>,
    /// Filter by quote IDs.
    #[serde(rename = "quoteIds", skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub quote_ids: Vec<String>,
    /// Filter by request IDs.
    #[serde(rename = "requestIds", skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub request_ids: Vec<String>,
    /// Filter by market condition IDs.
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, B256>")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub markets: Vec<B256>,
    /// Minimum size in tokens.
    pub size_min: Option<Decimal>,
    /// Maximum size in tokens.
    pub size_max: Option<Decimal>,
    /// Minimum size in USDC.
    pub size_usdc_min: Option<Decimal>,
    /// Maximum size in USDC.
    pub size_usdc_max: Option<Decimal>,
    /// Minimum price.
    pub price_min: Option<Decimal>,
    /// Maximum price.
    pub price_max: Option<Decimal>,
    /// Sort field.
    pub sort_by: Option<RfqSortBy>,
    /// Sort direction.
    pub sort_dir: Option<RfqSortDir>,
}

/// Request body for accepting an RFQ quote.
///
/// This creates an Order that the Requester must sign.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct AcceptRfqQuoteRequest {
    /// ID of the Request.
    pub request_id: String,
    /// ID of the Quote being accepted.
    pub quote_id: String,
    /// Maker's amount in base units.
    pub maker_amount: Decimal,
    /// Taker's amount in base units.
    pub taker_amount: Decimal,
    /// Outcome token ID.
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
    /// Maker's address.
    pub maker: Address,
    /// Signer's address.
    pub signer: Address,
    /// Taker's address.
    pub taker: Address,
    /// Order nonce.
    pub nonce: u64,
    /// Unix timestamp for order expiration.
    pub expiration: i64,
    /// Order side (BUY or SELL).
    pub side: Side,
    /// Fee rate in basis points.
    pub fee_rate_bps: u64,
    /// EIP-712 signature.
    pub signature: String,
    /// Random salt for order uniqueness.
    pub salt: String,
    /// Owner identifier.
    pub owner: ApiKey,
}

/// Request body for approving an RFQ order.
///
/// Quoter approves an RFQ order during the last look window.
#[cfg(feature = "rfq")]
#[non_exhaustive]
#[serde_as]
#[derive(Debug, Clone, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(on(String, into))]
pub struct ApproveRfqOrderRequest {
    /// ID of the Request.
    pub request_id: String,
    /// ID of the Quote being approved.
    pub quote_id: String,
    /// Maker's amount in base units.
    pub maker_amount: Decimal,
    /// Taker's amount in base units.
    pub taker_amount: Decimal,
    /// Outcome token ID.
    #[serde_as(as = "DisplayFromStr")]
    pub token_id: U256,
    /// Maker's address.
    pub maker: Address,
    /// Signer's address.
    pub signer: Address,
    /// Taker's address.
    pub taker: Address,
    /// Order nonce.
    pub nonce: u64,
    /// Unix timestamp for order expiration.
    pub expiration: Timestamp,
    /// Order side (BUY or SELL).
    pub side: Side,
    /// Fee rate in basis points.
    pub fee_rate_bps: u64,
    /// EIP-712 signature.
    pub signature: String,
    /// Random salt for order uniqueness.
    pub salt: String,
    /// Owner identifier.
    pub owner: ApiKey,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToQueryParams as _;
    use crate::types::b256;

    #[test]
    fn trades_request_as_params_should_succeed() {
        let market = b256!("0000000000000000000000000000000000000000000000000000000000010000");
        let request = TradesRequest::builder()
            .market(market)
            .asset_id(U256::from(100))
            .id("aa-bb")
            .maker_address(Address::ZERO)
            .build();

        assert_eq!(
            request.query_params(None),
            "?id=aa-bb&maker=0x0000000000000000000000000000000000000000&market=0x0000000000000000000000000000000000000000000000000000000000010000&asset_id=100"
        );
        assert_eq!(
            request.query_params(Some("1")),
            "?id=aa-bb&maker=0x0000000000000000000000000000000000000000&market=0x0000000000000000000000000000000000000000000000000000000000010000&asset_id=100&next_cursor=1"
        );
    }

    #[test]
    fn orders_request_as_params_should_succeed() {
        let market = b256!("0000000000000000000000000000000000000000000000000000000000010000");
        let request = OrdersRequest::builder()
            .market(market)
            .asset_id(U256::from(100))
            .order_id("aa-bb")
            .build();

        assert_eq!(
            request.query_params(None),
            "?id=aa-bb&market=0x0000000000000000000000000000000000000000000000000000000000010000&asset_id=100"
        );
        assert_eq!(
            request.query_params(Some("1")),
            "?id=aa-bb&market=0x0000000000000000000000000000000000000000000000000000000000010000&asset_id=100&next_cursor=1"
        );
    }

    #[test]
    fn delete_notifications_request_as_params_should_succeed() {
        let empty_request = DeleteNotificationsRequest::builder().build();
        let request = DeleteNotificationsRequest::builder()
            .notification_ids(vec!["1".to_owned(), "2".to_owned()])
            .build();

        assert_eq!(empty_request.query_params(None), "");
        assert_eq!(request.query_params(None), "?ids=1%2C2");
    }

    #[test]
    fn balance_allowance_request_as_params_should_succeed() {
        let request = BalanceAllowanceRequest::builder()
            .asset_type(AssetType::Collateral)
            .token_id(U256::from(1))
            .signature_type(SignatureType::Eoa)
            .build();

        assert_eq!(
            request.query_params(None),
            "?asset_type=COLLATERAL&token_id=1&signature_type=0"
        );
    }

    #[test]
    fn user_rewards_earning_request_as_params_should_succeed() {
        let request = UserRewardsEarningRequest::builder()
            .date(NaiveDate::MIN)
            .build();

        assert_eq!(
            request.query_params(Some("1")),
            "?date=-262143-01-01&order_by=&position=&no_competition=false&next_cursor=1"
        );
    }
}
