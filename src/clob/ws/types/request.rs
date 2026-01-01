use serde::Serialize;

use crate::auth::Credentials;

/// Subscription request message sent to the WebSocket server.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize)]
pub struct SubscriptionRequest {
    /// Subscription type ("market" or "user")
    pub r#type: String,
    /// Operation type ("subscribe" or "unsubscribe")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// List of market IDs
    pub markets: Vec<String>,
    /// List of asset IDs
    #[serde(rename = "assets_ids")]
    pub asset_ids: Vec<String>,
    /// Request initial state dump
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_dump: Option<bool>,
    /// Enable custom features (`best_bid_ask`, `new_market`, `market_resolved`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_feature_enabled: Option<bool>,
    /// Authentication credentials
    #[serde(skip)]
    pub auth: Option<Credentials>,
}

impl SubscriptionRequest {
    /// Create a market subscription request.
    #[must_use]
    pub fn market(asset_ids: Vec<String>) -> Self {
        Self {
            r#type: "market".to_owned(),
            operation: Some("subscribe".to_owned()),
            markets: vec![],
            asset_ids,
            initial_dump: Some(true),
            custom_feature_enabled: None,
            auth: None,
        }
    }

    /// Create a market unsubscribe request.
    #[must_use]
    pub fn market_unsubscribe(asset_ids: Vec<String>) -> Self {
        Self {
            r#type: "market".to_owned(),
            operation: Some("unsubscribe".to_owned()),
            markets: vec![],
            asset_ids,
            initial_dump: None,
            custom_feature_enabled: None,
            auth: None,
        }
    }

    /// Create a user subscription request.
    #[must_use]
    pub fn user(markets: Vec<String>, auth: Credentials) -> Self {
        Self {
            r#type: "user".to_owned(),
            operation: Some("subscribe".to_owned()),
            markets,
            asset_ids: vec![],
            initial_dump: Some(true),
            custom_feature_enabled: None,
            auth: Some(auth),
        }
    }

    /// Create a user unsubscribe request.
    #[must_use]
    pub fn user_unsubscribe(markets: Vec<String>, auth: Credentials) -> Self {
        Self {
            r#type: "user".to_owned(),
            operation: Some("unsubscribe".to_owned()),
            markets,
            asset_ids: vec![],
            initial_dump: None,
            custom_feature_enabled: None,
            auth: Some(auth),
        }
    }

    /// Enable custom features on this subscription request.
    ///
    /// Enables receiving additional message types: `best_bid_ask`, `new_market`,
    /// `market_resolved`.
    #[must_use]
    pub fn with_custom_features(mut self, enabled: bool) -> Self {
        self.custom_feature_enabled = Some(enabled);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ApiKey;

    #[test]
    fn serialize_market_subscription_request() {
        let request = SubscriptionRequest::market(vec!["asset1".to_owned(), "asset2".to_owned()]);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"market\""));
        assert!(json.contains("\"assets_ids\""));
        assert!(json.contains("\"initial_dump\":true"));
    }

    #[test]
    fn serialize_user_subscription_request() {
        let credentials = Credentials::new(
            ApiKey::nil(),
            "test-secret".to_owned(),
            "test-pass".to_owned(),
        );
        let request = SubscriptionRequest::user(vec!["market1".to_owned()], credentials);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"user\""));
        assert!(json.contains("\"markets\""));
        assert!(json.contains("\"initial_dump\":true"));
    }

    #[test]
    fn serialize_market_subscription_with_custom_features() {
        let request =
            SubscriptionRequest::market(vec!["asset1".to_owned()]).with_custom_features(true);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"custom_feature_enabled\":true"));
    }

    #[test]
    fn serialize_market_unsubscribe_request() {
        let request =
            SubscriptionRequest::market_unsubscribe(vec!["asset1".to_owned(), "asset2".to_owned()]);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"market\""));
        assert!(json.contains("\"operation\":\"unsubscribe\""));
        assert!(json.contains("\"assets_ids\""));
        assert!(!json.contains("\"initial_dump\""));
    }

    #[test]
    fn serialize_user_unsubscribe_request() {
        let credentials = Credentials::new(
            ApiKey::nil(),
            "test-secret".to_owned(),
            "test-pass".to_owned(),
        );
        let request =
            SubscriptionRequest::user_unsubscribe(vec!["market1".to_owned()], credentials);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"user\""));
        assert!(json.contains("\"operation\":\"unsubscribe\""));
        assert!(!json.contains("\"initial_dump\""));
    }

    #[test]
    fn with_custom_features_false_serializes() {
        let request =
            SubscriptionRequest::market(vec!["asset1".to_owned()]).with_custom_features(false);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"custom_feature_enabled\":false"));
    }
}
