#![allow(
    clippy::module_name_repetitions,
    reason = "Request suffix is intentional for clarity"
)]

use alloy::primitives::Address;
use bon::Builder;
use chrono::NaiveDate;
use serde::Serialize;
use serde_with::{StringWithSeparator, formats::CommaSeparator, serde_as};

use crate::clob::types::{AssetType, Interval, Side, SignatureType};

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct MidpointRequest {
    pub token_id: String,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct PriceRequest {
    pub token_id: String,
    pub side: Side,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct SpreadRequest {
    pub token_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct OrderBookSummaryRequest {
    pub token_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct LastTradePriceRequest {
    pub token_id: String,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct PriceHistoryRequest {
    pub market: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<Interval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fidelity: Option<u32>,
}

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(on(String, into))]
pub struct CancelMarketOrderRequest {
    pub market: Option<String>,
    pub asset_id: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Builder, Serialize)]
#[builder(on(String, into))]
pub struct TradesRequest {
    pub id: Option<String>,
    #[serde(rename = "taker")]
    pub taker_address: Option<Address>,
    #[serde(rename = "maker")]
    pub maker_address: Option<Address>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub before: Option<i64>,
    pub after: Option<i64>,
}

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(on(String, into))]
pub struct OrdersRequest {
    #[serde(rename = "id")]
    pub order_id: Option<String>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
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
#[derive(Debug, Default, Clone, Builder, Serialize)]
#[builder(on(String, into))]
pub struct BalanceAllowanceRequest {
    pub asset_type: AssetType,
    pub token_id: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToQueryParams as _;

    #[test]
    fn trades_request_as_params_should_succeed() {
        let request = TradesRequest::builder()
            .market("10000")
            .asset_id("100")
            .id("aa-bb")
            .maker_address(Address::ZERO)
            .build();

        assert_eq!(
            request.query_params(None),
            "?id=aa-bb&maker=0x0000000000000000000000000000000000000000&market=10000&asset_id=100"
        );
        assert_eq!(
            request.query_params(Some("1")),
            "?id=aa-bb&maker=0x0000000000000000000000000000000000000000&market=10000&asset_id=100&next_cursor=1"
        );
    }

    #[test]
    fn orders_request_as_params_should_succeed() {
        let request = OrdersRequest::builder()
            .market("10000")
            .asset_id("100")
            .order_id("aa-bb")
            .build();

        assert_eq!(
            request.query_params(None),
            "?id=aa-bb&market=10000&asset_id=100"
        );
        assert_eq!(
            request.query_params(Some("1")),
            "?id=aa-bb&market=10000&asset_id=100&next_cursor=1"
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
            .token_id("1".to_owned())
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
