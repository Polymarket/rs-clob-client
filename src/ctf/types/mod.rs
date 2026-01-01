//! Types for CTF (Conditional Token Framework) operations.

mod request;
mod response;

pub use request::{
    CollectionIdRequest, ConditionIdRequest, MergePositionsRequest, PositionIdRequest,
    RedeemPositionsRequest, SplitPositionRequest,
};
pub use response::{
    CollectionIdResponse, ConditionIdResponse, MergePositionsResponse, PositionIdResponse,
    RedeemPositionsResponse, SplitPositionResponse,
};
