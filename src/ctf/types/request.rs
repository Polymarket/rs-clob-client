//! Request types for CTF operations.

use alloy::primitives::{B256, U256};
use bon::Builder;

use crate::types::Address;

/// Standard partition for binary markets (YES/NO).
/// Index 1 (0b01) represents the first outcome (typically YES).
/// Index 2 (0b10) represents the second outcome (typically NO).
pub const BINARY_PARTITION: [u64; 2] = [1, 2];

/// Request to calculate a condition ID.
///
/// The condition ID is derived from the oracle address, question hash, and number of outcome slots.
#[non_exhaustive]
#[derive(Debug, Clone, Builder)]
pub struct ConditionIdRequest {
    /// The oracle address that will report the outcome
    pub oracle: Address,
    /// Hash of the question being resolved
    pub question_id: B256,
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
    pub parent_collection_id: B256,
    /// The condition ID
    pub condition_id: B256,
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
    pub collection_id: B256,
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
    #[builder(default)]
    pub parent_collection_id: B256,
    /// The condition ID to split on
    pub condition_id: B256,
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
    #[builder(default)]
    pub parent_collection_id: B256,
    /// The condition ID to merge on
    pub condition_id: B256,
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
    #[builder(default)]
    pub parent_collection_id: B256,
    /// The condition ID to redeem
    pub condition_id: B256,
    /// Array of disjoint index sets representing outcome slots to redeem
    pub index_sets: Vec<U256>,
}

/// Request to redeem positions using the `NegRisk` adapter.
///
/// This is used for negative risk markets where redemption requires specifying
/// the amounts of each outcome token to redeem.
#[non_exhaustive]
#[derive(Debug, Clone, Builder)]
pub struct RedeemNegRiskRequest {
    /// The condition ID to redeem
    pub condition_id: B256,
    /// Array of amounts for each outcome token [yesAmount, noAmount]
    /// For binary markets, this should have 2 elements
    pub amounts: Vec<U256>,
}

// Convenience methods for binary markets
impl SplitPositionRequest {
    /// Creates a split request for a binary market (YES/NO).
    ///
    /// This is a convenience method that automatically uses the standard binary partition [1, 2].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use polymarket_client_sdk::ctf::types::SplitPositionRequest;
    /// # use polymarket_client_sdk::types::address;
    /// # use alloy::primitives::{B256, U256};
    /// let request = SplitPositionRequest::for_binary_market(
    ///     address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"), // USDC
    ///     B256::default(),
    ///     U256::from(1_000_000), // 1 USDC (6 decimals)
    /// );
    /// ```
    #[must_use]
    pub fn for_binary_market(collateral_token: Address, condition_id: B256, amount: U256) -> Self {
        Self {
            collateral_token,
            parent_collection_id: B256::default(),
            condition_id,
            partition: BINARY_PARTITION.iter().map(|&i| U256::from(i)).collect(),
            amount,
        }
    }
}

impl MergePositionsRequest {
    /// Creates a merge request for a binary market (YES/NO).
    ///
    /// This is a convenience method that automatically uses the standard binary partition [1, 2].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use polymarket_client_sdk::ctf::types::MergePositionsRequest;
    /// # use polymarket_client_sdk::types::address;
    /// # use alloy::primitives::{B256, U256};
    /// let request = MergePositionsRequest::for_binary_market(
    ///     address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"), // USDC
    ///     B256::default(),
    ///     U256::from(1_000_000), // 1 full set
    /// );
    /// ```
    #[must_use]
    pub fn for_binary_market(collateral_token: Address, condition_id: B256, amount: U256) -> Self {
        Self {
            collateral_token,
            parent_collection_id: B256::default(),
            condition_id,
            partition: BINARY_PARTITION.iter().map(|&i| U256::from(i)).collect(),
            amount,
        }
    }
}

impl RedeemPositionsRequest {
    /// Creates a redeem request for a binary market (YES/NO).
    ///
    /// This is a convenience method that automatically uses the standard binary index sets [1, 2].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use polymarket_client_sdk::ctf::types::RedeemPositionsRequest;
    /// # use polymarket_client_sdk::types::address;
    /// # use alloy::primitives::{B256, U256};
    /// let request = RedeemPositionsRequest::for_binary_market(
    ///     address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"), // USDC
    ///     B256::default(),
    /// );
    /// ```
    #[must_use]
    pub fn for_binary_market(collateral_token: Address, condition_id: B256) -> Self {
        Self {
            collateral_token,
            parent_collection_id: B256::default(),
            condition_id,
            index_sets: BINARY_PARTITION.iter().map(|&i| U256::from(i)).collect(),
        }
    }
}
