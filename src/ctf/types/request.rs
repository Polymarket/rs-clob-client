//! Request types for CTF operations.

use alloy::primitives::{FixedBytes, U256};
use bon::Builder;

use crate::types::Address;

/// Request to calculate a condition ID.
///
/// The condition ID is derived from the oracle address, question hash, and number of outcome slots.
#[non_exhaustive]
#[derive(Debug, Clone, Builder)]
pub struct ConditionIdRequest {
    /// The oracle address that will report the outcome
    pub oracle: Address,
    /// Hash of the question being resolved
    pub question_id: FixedBytes<32>,
    /// Number of outcome slots (typically 2 for binary markets)
    pub outcome_slot_count: U256,
}

/// Request to calculate a collection ID.
///
/// Creates collection identifiers using parent collection, condition ID, and index set.
#[non_exhaustive]
#[derive(Debug, Clone, Builder)]
pub struct CollectionIdRequest {
    /// Parent collection ID (typically zero for top-level positions)
    pub parent_collection_id: FixedBytes<32>,
    /// The condition ID
    pub condition_id: FixedBytes<32>,
    /// Index set representing outcome slots (e.g., 0b01 = 1, 0b10 = 2)
    pub index_set: U256,
}

/// Request to calculate a position ID.
///
/// Generates final ERC1155 token IDs from collateral token and collection ID.
#[non_exhaustive]
#[derive(Debug, Clone, Builder)]
pub struct PositionIdRequest {
    /// The collateral token address (e.g., USDC)
    pub collateral_token: Address,
    /// The collection ID
    pub collection_id: FixedBytes<32>,
}

/// Request to split collateral into outcome tokens.
///
/// Converts USDC collateral into matched outcome token pairs (YES/NO).
#[non_exhaustive]
#[derive(Debug, Clone, Builder)]
pub struct SplitPositionRequest {
    /// The collateral token address (e.g., USDC)
    pub collateral_token: Address,
    /// Parent collection ID (typically zero for Polymarket)
    #[builder(default = FixedBytes::<32>::ZERO)]
    pub parent_collection_id: FixedBytes<32>,
    /// The condition ID to split on
    pub condition_id: FixedBytes<32>,
    /// Array of disjoint index sets representing outcome slots.
    /// For binary markets: [1, 2] where 1 = 0b01 (YES) and 2 = 0b10 (NO)
    pub partition: Vec<U256>,
    /// Amount of collateral to split
    pub amount: U256,
}

/// Request to merge outcome tokens back into collateral.
///
/// Combines matched outcome token pairs back into USDC.
#[non_exhaustive]
#[derive(Debug, Clone, Builder)]
pub struct MergePositionsRequest {
    /// The collateral token address (e.g., USDC)
    pub collateral_token: Address,
    /// Parent collection ID (typically zero for Polymarket)
    #[builder(default = FixedBytes::<32>::ZERO)]
    pub parent_collection_id: FixedBytes<32>,
    /// The condition ID to merge on
    pub condition_id: FixedBytes<32>,
    /// Array of disjoint index sets representing outcome slots.
    /// For binary markets: [1, 2] where 1 = 0b01 (YES) and 2 = 0b10 (NO)
    pub partition: Vec<U256>,
    /// Amount of full sets to merge
    pub amount: U256,
}

/// Request to redeem winning outcome tokens for collateral.
///
/// After a condition is resolved, burns winning tokens to recover USDC.
#[non_exhaustive]
#[derive(Debug, Clone, Builder)]
pub struct RedeemPositionsRequest {
    /// The collateral token address (e.g., USDC)
    pub collateral_token: Address,
    /// Parent collection ID (typically zero for Polymarket)
    #[builder(default = FixedBytes::<32>::ZERO)]
    pub parent_collection_id: FixedBytes<32>,
    /// The condition ID to redeem
    pub condition_id: FixedBytes<32>,
    /// Array of disjoint index sets representing outcome slots to redeem
    pub index_sets: Vec<U256>,
}
