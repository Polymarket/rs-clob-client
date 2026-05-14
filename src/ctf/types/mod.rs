//! Types for CTF (Conditional Token Framework) operations.

mod request;
mod response;

pub use request::{
    BINARY_PARTITION, CollectionIdRequest, ConditionIdRequest, MergePositionsRequest,
    PositionIdRequest, RedeemNegRiskRequest, RedeemPositionsRequest, SplitPositionRequest,
};
pub use response::{
    CollectionIdResponse, ConditionIdResponse, MergePositionsResponse, PositionIdResponse,
    RedeemNegRiskResponse, RedeemPositionsResponse, SplitPositionResponse,
};
use serde::{Deserialize, Serialize};

/// The wallet type being used to interact with the CTF contract, which determines
/// the method of calling write functions on the contract.
///
/// - **Externally Owned Accounts** (EOA) call the CTF contract directly and pay
///   gas fees directly. This is the default behaviour and should be used when
///   using your own wallet (e.g. metamask)
/// - **Proxy wallets** call the CTF contract via the `ProxyFactory` contract. Proxy
///   wallets are generated when signing up to Polymarket with an email or via
///   google auth etc
/// - **GNOSIS** safe wallet is currently unimplemented for CTF calls
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalletType {
    /// **Externally Owned Accounts** (EOA) call the CTF contract directly and pay
    /// gas fees directly. This is the default behaviour and should be used when
    /// using your own wallet (e.g. metamask)
    #[default]
    #[serde(alias = "Eoa")]
    #[serde(alias = "eoa")]
    EOA,
    /// **Proxy wallets** call the CTF contract via the `ProxyFactory` contract. Proxy
    /// wallets are generated when signing up to Polymarket with an email or via
    /// google auth etc
    #[serde(alias = "Proxy")]
    #[serde(alias = "proxy")]
    Proxy,
    // TODO: GNOSIS safe wallet
    // GnosisSafe,
}
