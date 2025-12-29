#![allow(
    clippy::module_name_repetitions,
    reason = "Request suffix is intentional for clarity"
)]

use alloy::primitives::Address;
use bon::Builder;
use chrono::NaiveDate;
use serde::Serialize;

use crate::clob::types::{AssetType, Side, SignatureType};

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
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct OrderBookSummaryRequest {
    pub token_id: String,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct LastTradePriceRequest {
    pub token_id: String,
}

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(on(String, into))]
pub struct CancelMarketOrderRequest {
    pub market: Option<String>,
    pub asset_id: Option<String>,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Builder)]
#[builder(on(String, into))]
pub struct TradesRequest {
    pub id: Option<String>,
    pub maker_address: Option<Address>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub before: Option<i64>,
    pub after: Option<i64>,
}

impl TradesRequest {
    pub(crate) fn as_params(&self, next_cursor: Option<&String>) -> String {
        let id = self.id.as_ref().map(|o| format!("id={o}"));
        let maker_address = self
            .maker_address
            .as_ref()
            .map(|m| format!("maker_address={m}"));
        let market = self.market.as_ref().map(|a| format!("market={a}"));
        let asset_id = self.asset_id.as_ref().map(|a| format!("asset_id={a}"));
        let before = self.before.as_ref().map(|a| format!("before={a}"));
        let after = self.after.as_ref().map(|a| format!("after={a}"));

        let params = [id, maker_address, market, asset_id, before, after]
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join("&");

        format_params_with_cursor(params.as_str(), next_cursor)
    }
}

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Builder)]
#[builder(on(String, into))]
pub struct OrdersRequest {
    pub order_id: Option<String>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
}

impl OrdersRequest {
    pub(crate) fn as_params(&self, next_cursor: Option<&String>) -> String {
        let order_id = self.order_id.as_ref().map(|o| format!("order_id={o}"));
        let market = self.market.as_ref().map(|m| format!("market={m}"));
        let asset_id = self.asset_id.as_ref().map(|a| format!("asset_id={a}"));

        let params = [order_id, market, asset_id]
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join("&");

        format_params_with_cursor(params.as_str(), next_cursor)
    }
}

#[non_exhaustive]
#[derive(Debug, Default, Serialize, Builder)]
pub struct DeleteNotificationsRequest {
    pub notification_ids: Option<Vec<String>>,
}

impl DeleteNotificationsRequest {
    pub(crate) fn as_params(&self) -> String {
        self.notification_ids.as_ref().map_or(String::new(), |ids| {
            if ids.is_empty() {
                String::new()
            } else {
                format!("?ids={}", ids.join(","))
            }
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Builder)]
#[builder(on(String, into))]
pub struct BalanceAllowanceRequest {
    pub asset_type: AssetType,
    pub token_id: Option<String>,
    pub signature_type: Option<SignatureType>,
}

impl BalanceAllowanceRequest {
    pub(crate) fn as_params(&self, default_signature_type: SignatureType) -> String {
        let token_id = self.token_id.as_ref().map(|m| format!("token_id={m}"));
        let signature_type = self.signature_type.unwrap_or(default_signature_type);

        let signature_type = format!("signature_type={}", signature_type as u8);

        let params = [
            Some(format!("asset_type={}", self.asset_type)),
            token_id,
            Some(signature_type),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join("&");

        if params.is_empty() {
            String::new()
        } else {
            format!("?{params}")
        }
    }
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

impl UserRewardsEarningRequest {
    pub(crate) fn as_params(&self, next_cursor: Option<&String>) -> String {
        let order_by = format!("order_by={}", self.order_by);
        let position = format!("position={}", self.position);
        let no_competition = format!("no_competition={}", self.no_competition);

        let params = format!("date={}&{order_by}&{position}&{no_competition}", self.date);

        format_params_with_cursor(params.as_str(), next_cursor)
    }
}

fn format_params_with_cursor(params: &str, next_cursor: Option<&String>) -> String {
    match (params, next_cursor) {
        ("", Some(cursor)) => format!("?next_cursor={cursor}"),
        ("", None) => String::new(),
        (params, Some(cursor)) => format!("?{params}&next_cursor={cursor}"),
        (params, None) => format!("?{params}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trades_request_as_params_should_succeed() {
        let request = TradesRequest::builder()
            .market("10000")
            .asset_id("100")
            .id("aa-bb")
            .maker_address(Address::ZERO)
            .build();

        assert_eq!(
            request.as_params(None),
            "?id=aa-bb&maker_address=0x0000000000000000000000000000000000000000&market=10000&asset_id=100"
        );
        assert_eq!(
            request.as_params(Some(&"1".to_owned())),
            "?id=aa-bb&maker_address=0x0000000000000000000000000000000000000000&market=10000&asset_id=100&next_cursor=1"
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
            request.as_params(None),
            "?order_id=aa-bb&market=10000&asset_id=100"
        );
        assert_eq!(
            request.as_params(Some(&"1".to_owned())),
            "?order_id=aa-bb&market=10000&asset_id=100&next_cursor=1"
        );
    }

    #[test]
    fn delete_notifications_request_as_params_should_succeed() {
        let empty_request = DeleteNotificationsRequest::builder().build();
        let request = DeleteNotificationsRequest::builder()
            .notification_ids(vec!["1".to_owned(), "2".to_owned()])
            .build();

        assert_eq!(empty_request.as_params(), "");
        assert_eq!(request.as_params(), "?ids=1,2");
    }

    #[test]
    fn balance_allowance_request_as_params_should_succeed() {
        let request = BalanceAllowanceRequest::builder()
            .asset_type(AssetType::Collateral)
            .token_id("1".to_owned())
            .build();

        assert_eq!(
            request.as_params(SignatureType::Eoa),
            "?asset_type=COLLATERAL&token_id=1&signature_type=0"
        );
    }

    #[test]
    fn user_rewards_earning_request_as_params_should_succeed() {
        let request = UserRewardsEarningRequest::builder()
            .date(NaiveDate::MIN)
            .build();

        assert_eq!(
            request.as_params(Some(&"1".to_owned())),
            "?date=-262143-01-01&order_by=&position=&no_competition=false&next_cursor=1"
        );
    }
}
