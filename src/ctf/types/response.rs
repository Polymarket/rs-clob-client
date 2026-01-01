//! Response types for CTF operations.

use alloy::primitives::{FixedBytes, U256};

/// Response from calculating a condition ID.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ConditionIdResponse {
    /// The calculated condition ID
    pub condition_id: FixedBytes<32>,
}

/// Response from calculating a collection ID.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CollectionIdResponse {
    /// The calculated collection ID
    pub collection_id: FixedBytes<32>,
}

/// Response from calculating a position ID.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct PositionIdResponse {
    /// The calculated position ID (ERC1155 token ID)
    pub position_id: U256,
}

/// Response from a split position transaction.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SplitPositionResponse {
    /// Transaction hash
    pub transaction_hash: FixedBytes<32>,
    /// Block number where the transaction was mined
    pub block_number: u64,
}

/// Response from a merge positions transaction.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct MergePositionsResponse {
    /// Transaction hash
    pub transaction_hash: FixedBytes<32>,
    /// Block number where the transaction was mined
    pub block_number: u64,
}

/// Response from a redeem positions transaction.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct RedeemPositionsResponse {
    /// Transaction hash
    pub transaction_hash: FixedBytes<32>,
    /// Block number where the transaction was mined
    pub block_number: u64,
}
