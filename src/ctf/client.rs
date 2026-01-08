//! CTF (Conditional Token Framework) client for interacting with the Gnosis CTF contract.
//!
//! The CTF contract is deployed at `0x4D97DCd97eC945f40cF65F87097ACe5EA0476045` on Polygon.
//!
//! # Operations
//!
//! - **ID Calculation**: Compute condition IDs, collection IDs, and position IDs
//! - **Split**: Convert USDC collateral into outcome token pairs (YES/NO)
//! - **Merge**: Combine outcome token pairs back into USDC
//! - **Redeem**: Redeem winning outcome tokens after market resolution
//!
//! # Example
//!
//! ```no_run
//! use polymarket_client_sdk::ctf::Client;
//! use alloy::providers::ProviderBuilder;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = ProviderBuilder::new()
//!     .connect("https://polygon-rpc.com")
//!     .await?;
//!
//! let client = Client::new(provider, 137)?;
//! # Ok(())
//! # }
//! ```

#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "Alloy sol! macro generates code that triggers these lints"
)]

use alloy::primitives::ChainId;
use alloy::providers::Provider;
use alloy::sol;

use super::error::CtfError;
use super::types::{
    CollectionIdRequest, CollectionIdResponse, ConditionIdRequest, ConditionIdResponse,
    MergePositionsRequest, MergePositionsResponse, PositionIdRequest, PositionIdResponse,
    RedeemNegRiskRequest, RedeemNegRiskResponse, RedeemPositionsRequest, RedeemPositionsResponse,
    SplitPositionRequest, SplitPositionResponse,
};
use crate::{Result, contract_config};

// CTF (Conditional Token Framework) contract interface
//
// This interface is based on the Gnosis CTF contract.
//
// Source: https://github.com/gnosis/conditional-tokens-contracts
// Documentation: https://docs.polymarket.com/developers/CTF/overview
//
// Key functions implemented:
// - getConditionId, getCollectionId, getPositionId: Pure/view functions for ID calculations
// - splitPosition: Convert collateral into outcome tokens
// - mergePositions: Combine outcome tokens back into collateral
// - redeemPositions: Redeem winning tokens after resolution
// - prepareCondition: Initialize a new condition (included for completeness)
sol! {
    #[sol(rpc)]
    interface IConditionalTokens {
        /// Prepares a condition by initializing it with an oracle, question hash, and outcome slot count.
        function prepareCondition(
            address oracle,
            bytes32 questionId,
            uint256 outcomeSlotCount
        ) external;

        /// Calculates the condition ID from oracle, question hash, and outcome slot count.
        function getConditionId(
            address oracle,
            bytes32 questionId,
            uint256 outcomeSlotCount
        ) external pure returns (bytes32);

        /// Calculates the collection ID from parent collection, condition ID, and index set.
        function getCollectionId(
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256 indexSet
        ) external view returns (bytes32);

        /// Calculates the position ID (ERC1155 token ID) from collateral token and collection ID.
        function getPositionId(
            address collateralToken,
            bytes32 collectionId
        ) external pure returns (uint256);

        /// Splits collateral into outcome tokens.
        function splitPosition(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] calldata partition,
            uint256 amount
        ) external;

        /// Merges outcome tokens back into collateral.
        function mergePositions(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] calldata partition,
            uint256 amount
        ) external;

        /// Redeems winning outcome tokens for collateral.
        function redeemPositions(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] calldata indexSets
        ) external;
    }

    #[sol(rpc)]
    interface INegRiskAdapter {
        /// Redeems positions from negative risk markets with specific amounts.
        function redeemPositions(
            bytes32 conditionId,
            uint256[] calldata amounts
        ) external;
    }
}

/// Client for interacting with the Conditional Token Framework contract.
///
/// The CTF contract handles tokenization of market outcomes as ERC1155 tokens.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Client<P: Provider> {
    contract: IConditionalTokens::IConditionalTokensInstance<P>,
    neg_risk_adapter: Option<INegRiskAdapter::INegRiskAdapterInstance<P>>,
    provider: P,
}

impl<P: Provider + Clone> Client<P> {
    /// Creates a new CTF client for the specified chain.
    ///
    /// # Arguments
    ///
    /// * `provider` - An alloy provider instance
    /// * `chain_id` - The chain ID (137 for Polygon mainnet, 80002 for Amoy testnet)
    ///
    /// # Errors
    ///
    /// Returns an error if the contract configuration is not found for the given chain.
    pub fn new(provider: P, chain_id: ChainId) -> Result<Self> {
        let config = contract_config(chain_id, false).ok_or_else(|| {
            CtfError::ContractCall(format!(
                "CTF contract configuration not found for chain ID {chain_id}"
            ))
        })?;

        let contract = IConditionalTokens::new(config.conditional_tokens, provider.clone());

        Ok(Self {
            contract,
            neg_risk_adapter: None,
            provider,
        })
    }

    /// Creates a new CTF client with `NegRisk` adapter support.
    ///
    /// Use this constructor when you need to work with negative risk markets.
    ///
    /// # Arguments
    ///
    /// * `provider` - An alloy provider instance
    /// * `chain_id` - The chain ID (137 for Polygon mainnet, 80002 for Amoy testnet)
    ///
    /// # Errors
    ///
    /// Returns an error if the contract configuration is not found for the given chain,
    /// or if the `NegRisk` adapter is not configured for the chain.
    pub fn with_neg_risk(provider: P, chain_id: ChainId) -> Result<Self> {
        let config = contract_config(chain_id, true).ok_or_else(|| {
            CtfError::ContractCall(format!(
                "NegRisk contract configuration not found for chain ID {chain_id}"
            ))
        })?;

        let contract = IConditionalTokens::new(config.conditional_tokens, provider.clone());

        let neg_risk_adapter = config
            .neg_risk_adapter
            .map(|addr| INegRiskAdapter::new(addr, provider.clone()));

        Ok(Self {
            contract,
            neg_risk_adapter,
            provider,
        })
    }

    /// Calculates a condition ID.
    ///
    /// The condition ID is derived from the oracle address, question hash, and number of outcome slots.
    ///
    /// # Errors
    ///
    /// Returns an error if the contract call fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self), fields(
            oracle = %request.oracle,
            question_id = %request.question_id,
            outcome_slot_count = %request.outcome_slot_count
        ))
    )]
    pub async fn condition_id(&self, request: &ConditionIdRequest) -> Result<ConditionIdResponse> {
        let condition_id = self
            .contract
            .getConditionId(
                request.oracle,
                request.question_id,
                request.outcome_slot_count,
            )
            .call()
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to get condition ID: {e}")))?;

        Ok(ConditionIdResponse { condition_id })
    }

    /// Calculates a collection ID.
    ///
    /// Creates collection identifiers using parent collection, condition ID, and index set.
    ///
    /// # Errors
    ///
    /// Returns an error if the contract call fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self), fields(
            parent_collection_id = %request.parent_collection_id,
            condition_id = %request.condition_id,
            index_set = %request.index_set
        ))
    )]
    pub async fn collection_id(
        &self,
        request: &CollectionIdRequest,
    ) -> Result<CollectionIdResponse> {
        let collection_id = self
            .contract
            .getCollectionId(
                request.parent_collection_id,
                request.condition_id,
                request.index_set,
            )
            .call()
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to get collection ID: {e}")))?;

        Ok(CollectionIdResponse { collection_id })
    }

    /// Calculates a position ID (ERC1155 token ID).
    ///
    /// Generates final token IDs from collateral token and collection ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the contract call fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self), fields(
            collateral_token = %request.collateral_token,
            collection_id = %request.collection_id
        ))
    )]
    pub async fn position_id(&self, request: &PositionIdRequest) -> Result<PositionIdResponse> {
        let position_id = self
            .contract
            .getPositionId(request.collateral_token, request.collection_id)
            .call()
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to get position ID: {e}")))?;

        Ok(PositionIdResponse { position_id })
    }

    /// Splits collateral into outcome tokens.
    ///
    /// Converts USDC collateral into matched outcome token pairs (YES/NO).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transaction fails to send
    /// - The transaction fails to be mined
    /// - The wallet doesn't have sufficient collateral
    /// - The condition hasn't been prepared
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self), fields(
            collateral_token = %request.collateral_token,
            condition_id = %request.condition_id,
            amount = %request.amount
        ))
    )]
    pub async fn split_position(
        &self,
        request: &SplitPositionRequest,
    ) -> Result<SplitPositionResponse> {
        let pending_tx = self
            .contract
            .splitPosition(
                request.collateral_token,
                request.parent_collection_id,
                request.condition_id,
                request.partition.clone(),
                request.amount,
            )
            .send()
            .await
            .map_err(|e| {
                CtfError::ContractCall(format!("Failed to send split transaction: {e}"))
            })?;

        let transaction_hash = *pending_tx.tx_hash();

        let receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to get split receipt: {e}")))?;

        Ok(SplitPositionResponse {
            transaction_hash,
            block_number: receipt.block_number.ok_or_else(|| {
                CtfError::ContractCall("Block number not available in receipt".to_owned())
            })?,
        })
    }

    /// Merges outcome tokens back into collateral.
    ///
    /// Combines matched outcome token pairs back into USDC.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transaction fails to send
    /// - The transaction fails to be mined
    /// - The wallet doesn't have sufficient outcome tokens
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self), fields(
            collateral_token = %request.collateral_token,
            condition_id = %request.condition_id,
            amount = %request.amount
        ))
    )]
    pub async fn merge_positions(
        &self,
        request: &MergePositionsRequest,
    ) -> Result<MergePositionsResponse> {
        let pending_tx = self
            .contract
            .mergePositions(
                request.collateral_token,
                request.parent_collection_id,
                request.condition_id,
                request.partition.clone(),
                request.amount,
            )
            .send()
            .await
            .map_err(|e| {
                CtfError::ContractCall(format!("Failed to send merge transaction: {e}"))
            })?;

        let transaction_hash = *pending_tx.tx_hash();

        let receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to get merge receipt: {e}")))?;

        Ok(MergePositionsResponse {
            transaction_hash,
            block_number: receipt.block_number.ok_or_else(|| {
                CtfError::ContractCall("Block number not available in receipt".to_owned())
            })?,
        })
    }

    /// Redeems winning outcome tokens for collateral.
    ///
    /// After a condition is resolved, burns winning tokens to recover USDC.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transaction fails to send
    /// - The transaction fails to be mined
    /// - The condition hasn't been resolved
    /// - The wallet doesn't have the specified outcome tokens
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self), fields(
            collateral_token = %request.collateral_token,
            condition_id = %request.condition_id
        ))
    )]
    pub async fn redeem_positions(
        &self,
        request: &RedeemPositionsRequest,
    ) -> Result<RedeemPositionsResponse> {
        let pending_tx = self
            .contract
            .redeemPositions(
                request.collateral_token,
                request.parent_collection_id,
                request.condition_id,
                request.index_sets.clone(),
            )
            .send()
            .await
            .map_err(|e| {
                CtfError::ContractCall(format!("Failed to send redeem transaction: {e}"))
            })?;

        let transaction_hash = *pending_tx.tx_hash();

        let receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to get redeem receipt: {e}")))?;

        Ok(RedeemPositionsResponse {
            transaction_hash,
            block_number: receipt.block_number.ok_or_else(|| {
                CtfError::ContractCall("Block number not available in receipt".to_owned())
            })?,
        })
    }

    /// Redeems positions from negative risk markets.
    ///
    /// This method uses the `NegRisk` adapter to redeem positions by specifying
    /// the exact amounts of each outcome token to redeem. This is different from
    /// the standard `redeem_positions` which uses index sets.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The client was not created with `with_neg_risk()` (adapter not available)
    /// - The transaction fails to send
    /// - The transaction fails to be mined
    /// - The condition hasn't been resolved
    /// - The wallet doesn't have the specified outcome token amounts
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self), fields(
            condition_id = %request.condition_id,
            amounts_len = request.amounts.len()
        ))
    )]
    pub async fn redeem_neg_risk(
        &self,
        request: &RedeemNegRiskRequest,
    ) -> Result<RedeemNegRiskResponse> {
        let adapter = self.neg_risk_adapter.as_ref().ok_or_else(|| {
            CtfError::ContractCall(
                "NegRisk adapter not available. Use Client::with_neg_risk() to enable NegRisk support".to_owned()
            )
        })?;

        let pending_tx = adapter
            .redeemPositions(request.condition_id, request.amounts.clone())
            .send()
            .await
            .map_err(|e| {
                CtfError::ContractCall(format!("Failed to send NegRisk redeem transaction: {e}"))
            })?;

        let transaction_hash = *pending_tx.tx_hash();

        let receipt = pending_tx.get_receipt().await.map_err(|e| {
            CtfError::ContractCall(format!("Failed to get NegRisk redeem receipt: {e}"))
        })?;

        Ok(RedeemNegRiskResponse {
            transaction_hash,
            block_number: receipt.block_number.ok_or_else(|| {
                CtfError::ContractCall("Block number not available in receipt".to_owned())
            })?,
        })
    }

    /// Returns a reference to the underlying provider.
    #[must_use]
    pub const fn provider(&self) -> &P {
        &self.provider
    }
}
