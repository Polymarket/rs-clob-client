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
