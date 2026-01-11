//! Response types for CTF operations.

use alloy::primitives::{B256, U256};

/// Response from calculating a condition ID.
#[derive(Debug, Clone)]
pub struct ConditionIdResponse {
    /// The calculated condition ID
    pub condition_id: B256,
}

/// Response from calculating a collection ID.
#[derive(Debug, Clone)]
pub struct CollectionIdResponse {
    /// The calculated collection ID
    pub collection_id: B256,
}

/// Response from calculating a position ID.
#[derive(Debug, Clone)]
pub struct PositionIdResponse {
    /// The calculated position ID (ERC1155 token ID)
    pub position_id: U256,
}

/// Response from a split position transaction.
#[derive(Debug, Clone)]
pub struct SplitPositionResponse {
    /// Transaction hash
    pub transaction_hash: B256,
    /// Block number where the transaction was mined
    pub block_number: u64,
}

/// Response from a merge positions transaction.
#[derive(Debug, Clone)]
pub struct MergePositionsResponse {
    /// Transaction hash
    pub transaction_hash: B256,
    /// Block number where the transaction was mined
    pub block_number: u64,
}

/// Response from a redeem positions transaction.
#[derive(Debug, Clone)]
pub struct RedeemPositionsResponse {
    /// Transaction hash
    pub transaction_hash: B256,
    /// Block number where the transaction was mined
    pub block_number: u64,
}

/// Response from a `NegRisk` redeem transaction.
#[derive(Debug, Clone)]
pub struct RedeemNegRiskResponse {
    /// Transaction hash
    pub transaction_hash: B256,
    /// Block number where the transaction was mined
    pub block_number: u64,
}
