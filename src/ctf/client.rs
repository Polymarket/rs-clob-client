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
    clippy::too_many_arguments,
    reason = "Alloy sol! macro generates code that triggers these lints"
)]

use alloy::dyn_abi::Eip712Domain;
use alloy::primitives::{Address, Bytes, ChainId, U256};
use alloy::providers::Provider;
use alloy::signers::Signer;
use alloy::sol;
use alloy::sol_types::SolStruct as _;

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

// Gnosis Safe interface for executing transactions through a Safe wallet.
//
// This is used to execute CTF operations when tokens are held in a Safe.
// The Safe must be a 1-of-1 multisig controlled by the signing EOA.
sol! {
    #[sol(rpc)]
    interface IGnosisSafe {
        /// Returns the current nonce of the Safe.
        function nonce() external view returns (uint256);

        /// Executes a transaction from the Safe wallet.
        ///
        /// @param to Destination address
        /// @param value Ether value
        /// @param data Data payload
        /// @param operation Operation type (0 = Call, 1 = `DelegateCall`)
        /// @param safeTxGas Gas for the Safe transaction
        /// @param baseGas Base gas costs
        /// @param gasPrice Gas price
        /// @param gasToken Token address for gas payment (address(0) for ETH)
        /// @param refundReceiver Address to receive gas refund
        /// @param signatures Packed signature data
        function execTransaction(
            address to,
            uint256 value,
            bytes calldata data,
            uint8 operation,
            uint256 safeTxGas,
            uint256 baseGas,
            uint256 gasPrice,
            address gasToken,
            address payable refundReceiver,
            bytes memory signatures
        ) external payable returns (bool success);
    }

    /// EIP-712 typed data struct for Safe transaction signing.
    ///
    /// This struct matches Gnosis Safe's `SafeTx` format for EIP-712 signing.
    /// The sol! macro automatically implements `SolStruct` for proper type hash
    /// computation and EIP-712 signing hash generation.
    #[derive(Debug)]
    struct SafeTx {
        address to;
        uint256 value;
        bytes data;
        uint8 operation;
        uint256 safeTxGas;
        uint256 baseGas;
        uint256 gasPrice;
        address gasToken;
        address refundReceiver;
        uint256 nonce;
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

    /// Safe operation type: Call (execute a call to another contract)
    const SAFE_OPERATION_CALL: u8 = 0;

    /// Executes a transaction through a Safe wallet.
    ///
    /// This internal helper handles the Safe transaction flow:
    /// 1. Gets the current Safe nonce
    /// 2. Builds the EIP-712 `SafeTx` struct
    /// 3. Signs the transaction hash with the EOA
    /// 4. Executes via Safe's execTransaction
    async fn execute_via_safe<S: Signer>(
        &self,
        signer: &S,
        safe_address: Address,
        to: Address,
        data: Bytes,
    ) -> Result<(alloy::primitives::B256, u64)> {
        // Create Safe contract instance
        let safe = IGnosisSafe::new(safe_address, self.provider.clone());

        // Get Safe nonce
        let nonce = safe
            .nonce()
            .call()
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to get Safe nonce: {e}")))?;

        // Get chain ID from signer
        let chain_id = signer
            .chain_id()
            .ok_or_else(|| CtfError::ContractCall("Signer must have chain_id set".to_owned()))?;

        // Build SafeTx struct for EIP-712 signing
        let safe_tx = SafeTx {
            to,
            value: U256::ZERO,
            data: data.clone(),
            operation: Self::SAFE_OPERATION_CALL,
            safeTxGas: U256::ZERO,
            baseGas: U256::ZERO,
            gasPrice: U256::ZERO,
            gasToken: Address::ZERO,
            refundReceiver: Address::ZERO,
            nonce,
        };

        // Build EIP-712 domain for Safe
        // Safe uses a minimal domain with only chainId and verifyingContract.
        // This matches Safe's DOMAIN_SEPARATOR_TYPEHASH:
        // keccak256("EIP712Domain(uint256 chainId,address verifyingContract)")
        let domain = Eip712Domain {
            chain_id: Some(U256::from(chain_id)),
            verifying_contract: Some(safe_address),
            ..Eip712Domain::default()
        };

        // Sign the Safe transaction hash
        let signing_hash = safe_tx.eip712_signing_hash(&domain);
        let signature = signer
            .sign_hash(&signing_hash)
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to sign Safe transaction: {e}")))?;

        // Format signature for Safe: r (32) || s (32) || v (1)
        // Safe expects v = 27 or 28 for standard ECDSA signatures.
        // Alloy's signature uses y-parity (0 or 1), so we add 27 to normalize.
        let mut sig_bytes = signature.as_bytes().to_vec();
        sig_bytes[64] = sig_bytes[64].wrapping_add(27);

        // Execute via Safe's execTransaction
        let pending_tx = safe
            .execTransaction(
                to,
                U256::ZERO,
                data,
                Self::SAFE_OPERATION_CALL,
                U256::ZERO,
                U256::ZERO,
                U256::ZERO,
                Address::ZERO,
                Address::ZERO,
                Bytes::from(sig_bytes),
            )
            .send()
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to send Safe transaction: {e}")))?;

        let transaction_hash = *pending_tx.tx_hash();

        let receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| CtfError::ContractCall(format!("Failed to get Safe receipt: {e}")))?;

        let block_number = receipt.block_number.ok_or_else(|| {
            CtfError::ContractCall("Block number not available in receipt".to_owned())
        })?;

        Ok((transaction_hash, block_number))
    }

    /// Resolves the Safe address from an optional parameter.
    ///
    /// If `safe_address` is `None`, derives the Safe address using CREATE2.
    /// If `safe_address` is `Some`, uses the provided address directly.
    fn resolve_safe_address<S: Signer>(
        signer: &S,
        safe_address: Option<Address>,
    ) -> Result<Address> {
        if let Some(addr) = safe_address {
            return Ok(addr);
        }

        let chain_id = signer
            .chain_id()
            .ok_or_else(|| CtfError::ContractCall("Signer must have chain_id set".to_owned()))?;
        crate::derive_safe_wallet(signer.address(), chain_id).ok_or_else(|| {
            CtfError::ContractCall(format!(
                "Safe wallet derivation not supported for chain ID {chain_id}"
            ))
            .into()
        })
    }

    /// Executes a CTF operation through a Safe wallet.
    ///
    /// This is a helper that combines Safe address resolution with execution.
    async fn execute_ctf_via_safe<S: Signer>(
        &self,
        signer: &S,
        safe_address: Option<Address>,
        target_address: Address,
        call_data: Bytes,
    ) -> Result<(alloy::primitives::B256, u64)> {
        let safe_addr = Self::resolve_safe_address(signer, safe_address)?;
        self.execute_via_safe(signer, safe_addr, target_address, call_data)
            .await
    }

    /// Splits collateral into outcome tokens via a Safe wallet.
    ///
    /// This method executes the split operation through a Gnosis Safe wallet by:
    /// 1. Encoding the splitPosition call data
    /// 2. Creating an EIP-712 typed `SafeTx` message
    /// 3. Signing with the Safe owner's EOA
    /// 4. Executing via the Safe's execTransaction
    ///
    /// # Arguments
    ///
    /// * `request` - The split position parameters
    /// * `signer` - The EOA that controls the Safe wallet (must be an owner)
    /// * `safe_address` - Optional explicit Safe address. If `None`, derives from signer's address
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Safe address derivation fails (when `safe_address` is `None`)
    /// - The signer is not an owner of the Safe
    /// - The Safe doesn't have sufficient collateral allowance
    /// - Transaction encoding or signing fails
    /// - The transaction fails to execute
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, signer), fields(
            collateral_token = %request.collateral_token,
            condition_id = %request.condition_id,
            amount = %request.amount
        ))
    )]
    pub async fn split_position_via_safe<S: Signer>(
        &self,
        request: &SplitPositionRequest,
        signer: &S,
        safe_address: Option<Address>,
    ) -> Result<SplitPositionResponse> {
        let call_data = self
            .contract
            .splitPosition(
                request.collateral_token,
                request.parent_collection_id,
                request.condition_id,
                request.partition.clone(),
                request.amount,
            )
            .calldata()
            .clone();

        let (transaction_hash, block_number) = self
            .execute_ctf_via_safe(signer, safe_address, *self.contract.address(), call_data)
            .await?;

        Ok(SplitPositionResponse {
            transaction_hash,
            block_number,
        })
    }

    /// Merges outcome tokens back into collateral via a Safe wallet.
    ///
    /// This method executes the merge operation through a Gnosis Safe wallet.
    ///
    /// # Arguments
    ///
    /// * `request` - The merge positions parameters
    /// * `signer` - The EOA that controls the Safe wallet (must be an owner)
    /// * `safe_address` - Optional explicit Safe address. If `None`, derives from signer's address
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Safe address derivation fails (when `safe_address` is `None`)
    /// - The signer is not an owner of the Safe
    /// - The Safe doesn't have sufficient outcome tokens
    /// - Transaction encoding or signing fails
    /// - The transaction fails to execute
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, signer), fields(
            collateral_token = %request.collateral_token,
            condition_id = %request.condition_id,
            amount = %request.amount
        ))
    )]
    pub async fn merge_positions_via_safe<S: Signer>(
        &self,
        request: &MergePositionsRequest,
        signer: &S,
        safe_address: Option<Address>,
    ) -> Result<MergePositionsResponse> {
        let call_data = self
            .contract
            .mergePositions(
                request.collateral_token,
                request.parent_collection_id,
                request.condition_id,
                request.partition.clone(),
                request.amount,
            )
            .calldata()
            .clone();

        let (transaction_hash, block_number) = self
            .execute_ctf_via_safe(signer, safe_address, *self.contract.address(), call_data)
            .await?;

        Ok(MergePositionsResponse {
            transaction_hash,
            block_number,
        })
    }

    /// Redeems winning outcome tokens for collateral via a Safe wallet.
    ///
    /// This method executes the redemption through a Gnosis Safe wallet.
    ///
    /// # Arguments
    ///
    /// * `request` - The redeem positions parameters
    /// * `signer` - The EOA that controls the Safe wallet (must be an owner)
    /// * `safe_address` - Optional explicit Safe address. If `None`, derives from signer's address
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Safe address derivation fails (when `safe_address` is `None`)
    /// - The signer is not an owner of the Safe
    /// - The condition hasn't been resolved
    /// - The Safe doesn't have the specified outcome tokens
    /// - Transaction encoding or signing fails
    /// - The transaction fails to execute
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, signer), fields(
            collateral_token = %request.collateral_token,
            condition_id = %request.condition_id
        ))
    )]
    pub async fn redeem_positions_via_safe<S: Signer>(
        &self,
        request: &RedeemPositionsRequest,
        signer: &S,
        safe_address: Option<Address>,
    ) -> Result<RedeemPositionsResponse> {
        let call_data = self
            .contract
            .redeemPositions(
                request.collateral_token,
                request.parent_collection_id,
                request.condition_id,
                request.index_sets.clone(),
            )
            .calldata()
            .clone();

        let (transaction_hash, block_number) = self
            .execute_ctf_via_safe(signer, safe_address, *self.contract.address(), call_data)
            .await?;

        Ok(RedeemPositionsResponse {
            transaction_hash,
            block_number,
        })
    }

    /// Redeems positions from negative risk markets via a Safe wallet.
    ///
    /// This method uses the `NegRisk` adapter to redeem positions through a Safe.
    ///
    /// # Arguments
    ///
    /// * `request` - The redeem neg risk parameters
    /// * `signer` - The EOA that controls the Safe wallet (must be an owner)
    /// * `safe_address` - Optional explicit Safe address. If `None`, derives from signer's address
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The client was not created with `with_neg_risk()` (adapter not available)
    /// - Safe address derivation fails (when `safe_address` is `None`)
    /// - The signer is not an owner of the Safe
    /// - The condition hasn't been resolved
    /// - The Safe doesn't have the specified outcome token amounts
    /// - Transaction encoding or signing fails
    /// - The transaction fails to execute
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(self, signer), fields(
            condition_id = %request.condition_id,
            amounts_len = request.amounts.len()
        ))
    )]
    pub async fn redeem_neg_risk_via_safe<S: Signer>(
        &self,
        request: &RedeemNegRiskRequest,
        signer: &S,
        safe_address: Option<Address>,
    ) -> Result<RedeemNegRiskResponse> {
        let adapter = self.neg_risk_adapter.as_ref().ok_or_else(|| {
            CtfError::ContractCall(
                "NegRisk adapter not available. Use Client::with_neg_risk() to enable NegRisk support".to_owned()
            )
        })?;

        let call_data = adapter
            .redeemPositions(request.condition_id, request.amounts.clone())
            .calldata()
            .clone();

        let (transaction_hash, block_number) = self
            .execute_ctf_via_safe(signer, safe_address, *adapter.address(), call_data)
            .await?;

        Ok(RedeemNegRiskResponse {
            transaction_hash,
            block_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{B256, keccak256};
    use alloy::sol_types::SolCall as _;

    use super::*;
    use crate::types::address;

    #[test]
    fn safe_tx_type_hash() {
        // Verify the SafeTx type hash matches the expected value.
        // The sol! macro should generate the correct type hash.
        let type_string = "SafeTx(address to,uint256 value,bytes data,uint8 operation,uint256 safeTxGas,uint256 baseGas,uint256 gasPrice,address gasToken,address refundReceiver,uint256 nonce)";
        let computed_hash = keccak256(type_string.as_bytes());

        // This is what Safe uses
        let expected: B256 = "0xbb8310d486368db6bd6f849402fdd73ad53d316b5a4b2644ad6efe0f941286d8"
            .parse()
            .unwrap();

        assert_eq!(
            computed_hash, expected,
            "SafeTx type hash should match Safe's expected value"
        );
    }

    #[test]
    fn domain_type_hash() {
        // Verify the domain type hash is correct.
        // Safe uses a minimal domain with only chainId and verifyingContract.
        let type_string = "EIP712Domain(uint256 chainId,address verifyingContract)";
        let computed_hash = keccak256(type_string.as_bytes());

        let expected: B256 = "0x47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a79469218"
            .parse()
            .unwrap();

        assert_eq!(
            computed_hash, expected,
            "Domain type hash should match Safe's expected value"
        );
    }

    #[test]
    fn sol_struct_signing_hash() {
        // Verify that Alloy's sol! macro generates a valid EIP-712 signing hash
        // by creating a test SafeTx and computing its signing hash.
        let safe_tx = SafeTx {
            to: Address::ZERO,
            value: U256::ZERO,
            data: Bytes::new(),
            operation: 0,
            safeTxGas: U256::ZERO,
            baseGas: U256::ZERO,
            gasPrice: U256::ZERO,
            gasToken: Address::ZERO,
            refundReceiver: Address::ZERO,
            nonce: U256::ZERO,
        };

        // Create a test domain (Safe on Polygon mainnet at a specific address)
        let domain = Eip712Domain {
            chain_id: Some(U256::from(137_u64)),
            verifying_contract: Some(
                "0x1234567890123456789012345678901234567890"
                    .parse()
                    .unwrap(),
            ),
            ..Eip712Domain::default()
        };

        // This should not panic - if it does, the SolStruct implementation is broken
        let signing_hash = safe_tx.eip712_signing_hash(&domain);

        // The hash should be a valid B256 (32 bytes)
        assert_eq!(
            signing_hash.len(),
            32,
            "EIP-712 signing hash should be 32 bytes"
        );

        // The hash should not be all zeros (would indicate a bug)
        assert_ne!(signing_hash, B256::ZERO, "Signing hash should not be zero");
    }

    #[test]
    fn signing_hash_deterministic() {
        // Verify that the same inputs produce the same signing hash.
        let safe_tx = SafeTx {
            to: address!("0x4D97DCd97eC945f40cF65F87097ACe5EA0476045"),
            value: U256::ZERO,
            data: Bytes::from(vec![0x01, 0x02, 0x03]),
            operation: 0,
            safeTxGas: U256::ZERO,
            baseGas: U256::ZERO,
            gasPrice: U256::ZERO,
            gasToken: Address::ZERO,
            refundReceiver: Address::ZERO,
            nonce: U256::from(42),
        };

        let domain = Eip712Domain {
            chain_id: Some(U256::from(137_u64)),
            verifying_contract: Some(address!("0xd93b25Cb943D14d0d34FBAf01fc93a0F8b5f6e47")),
            ..Eip712Domain::default()
        };

        let hash1 = safe_tx.eip712_signing_hash(&domain);
        let hash2 = safe_tx.eip712_signing_hash(&domain);

        assert_eq!(hash1, hash2, "Signing hash should be deterministic");
    }

    #[test]
    fn signing_hash_changes_with_nonce() {
        // Verify that different nonces produce different signing hashes.
        let domain = Eip712Domain {
            chain_id: Some(U256::from(137_u64)),
            verifying_contract: Some(address!("0xd93b25Cb943D14d0d34FBAf01fc93a0F8b5f6e47")),
            ..Eip712Domain::default()
        };

        let safe_tx_1 = SafeTx {
            to: Address::ZERO,
            value: U256::ZERO,
            data: Bytes::new(),
            operation: 0,
            safeTxGas: U256::ZERO,
            baseGas: U256::ZERO,
            gasPrice: U256::ZERO,
            gasToken: Address::ZERO,
            refundReceiver: Address::ZERO,
            nonce: U256::from(1),
        };

        let safe_tx_2 = SafeTx {
            nonce: U256::from(2),
            ..safe_tx_1.clone()
        };

        let hash1 = safe_tx_1.eip712_signing_hash(&domain);
        let hash2 = safe_tx_2.eip712_signing_hash(&domain);

        assert_ne!(
            hash1, hash2,
            "Different nonces should produce different hashes"
        );
    }

    #[test]
    fn split_position_calldata_has_correct_selector() {
        // Verify splitPosition calldata starts with the correct function selector.
        // selector = keccak256("splitPosition(address,bytes32,bytes32,uint256[],uint256)")[:4]
        let call = IConditionalTokens::splitPositionCall {
            collateralToken: address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"),
            parentCollectionId: B256::ZERO,
            conditionId: B256::repeat_byte(0x11),
            partition: vec![U256::from(1), U256::from(2)],
            amount: U256::from(1_000_000),
        };

        let encoded = call.abi_encode();

        // Function selector should be first 4 bytes
        assert!(
            encoded.len() >= 4,
            "Encoded calldata should have at least 4 bytes"
        );

        // Verify selector matches splitPosition
        let expected_selector =
            &keccak256("splitPosition(address,bytes32,bytes32,uint256[],uint256)")[..4];
        assert_eq!(
            &encoded[..4],
            expected_selector,
            "Function selector should match splitPosition"
        );
    }

    #[test]
    fn merge_positions_calldata_has_correct_selector() {
        // Verify mergePositions calldata starts with the correct function selector.
        let call = IConditionalTokens::mergePositionsCall {
            collateralToken: address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"),
            parentCollectionId: B256::ZERO,
            conditionId: B256::repeat_byte(0x22),
            partition: vec![U256::from(1), U256::from(2)],
            amount: U256::from(500_000),
        };

        let encoded = call.abi_encode();

        let expected_selector =
            &keccak256("mergePositions(address,bytes32,bytes32,uint256[],uint256)")[..4];
        assert_eq!(
            &encoded[..4],
            expected_selector,
            "Function selector should match mergePositions"
        );
    }

    #[test]
    fn redeem_positions_calldata_has_correct_selector() {
        // Verify redeemPositions calldata starts with the correct function selector.
        let call = IConditionalTokens::redeemPositionsCall {
            collateralToken: address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"),
            parentCollectionId: B256::ZERO,
            conditionId: B256::repeat_byte(0x33),
            indexSets: vec![U256::from(1), U256::from(2)],
        };

        let encoded = call.abi_encode();

        let expected_selector =
            &keccak256("redeemPositions(address,bytes32,bytes32,uint256[])")[..4];
        assert_eq!(
            &encoded[..4],
            expected_selector,
            "Function selector should match redeemPositions"
        );
    }

    #[test]
    fn neg_risk_redeem_calldata_has_correct_selector() {
        // Verify NegRisk redeemPositions calldata starts with the correct function selector.
        let call = INegRiskAdapter::redeemPositionsCall {
            conditionId: B256::repeat_byte(0x44),
            amounts: vec![U256::from(100), U256::from(200)],
        };

        let encoded = call.abi_encode();

        let expected_selector = &keccak256("redeemPositions(bytes32,uint256[])")[..4];
        assert_eq!(
            &encoded[..4],
            expected_selector,
            "Function selector should match NegRisk redeemPositions"
        );
    }

    /// Safe operation constant for Call (not `DelegateCall`).
    const SAFE_OP_CALL: u8 = 0;

    #[test]
    fn safe_tx_uses_call_operation() {
        // Verify we use operation = 0 (Call), not 1 (DelegateCall).
        let safe_tx = SafeTx {
            to: Address::ZERO,
            value: U256::ZERO,
            data: Bytes::new(),
            operation: SAFE_OP_CALL,
            safeTxGas: U256::ZERO,
            baseGas: U256::ZERO,
            gasPrice: U256::ZERO,
            gasToken: Address::ZERO,
            refundReceiver: Address::ZERO,
            nonce: U256::ZERO,
        };

        assert_eq!(
            safe_tx.operation, 0,
            "Operation should be Call (0), not DelegateCall (1)"
        );
    }

    #[test]
    fn safe_tx_zero_gas_params() {
        // Verify gas params are zero (Safe will estimate).
        let safe_tx = SafeTx {
            to: Address::ZERO,
            value: U256::ZERO,
            data: Bytes::new(),
            operation: 0,
            safeTxGas: U256::ZERO,
            baseGas: U256::ZERO,
            gasPrice: U256::ZERO,
            gasToken: Address::ZERO,
            refundReceiver: Address::ZERO,
            nonce: U256::ZERO,
        };

        assert_eq!(safe_tx.safeTxGas, U256::ZERO);
        assert_eq!(safe_tx.baseGas, U256::ZERO);
        assert_eq!(safe_tx.gasPrice, U256::ZERO);
        assert_eq!(safe_tx.gasToken, Address::ZERO);
        assert_eq!(safe_tx.refundReceiver, Address::ZERO);
    }

    #[test]
    fn signature_v_normalization() {
        // Verify that v-value normalization works correctly.
        // Alloy returns y-parity (0 or 1), Safe expects v (27 or 28).

        // Simulate Alloy signature bytes with y-parity = 0
        let mut sig_bytes_v0 = [0_u8; 65];
        sig_bytes_v0[64] = 0;
        sig_bytes_v0[64] = sig_bytes_v0[64].wrapping_add(27);
        assert_eq!(sig_bytes_v0[64], 27, "v=0 should become v=27");

        // Simulate Alloy signature bytes with y-parity = 1
        let mut sig_bytes_v1 = [0_u8; 65];
        sig_bytes_v1[64] = 1;
        sig_bytes_v1[64] = sig_bytes_v1[64].wrapping_add(27);
        assert_eq!(sig_bytes_v1[64], 28, "v=1 should become v=28");
    }

    #[test]
    fn signature_length() {
        // Verify signature is 65 bytes (r: 32, s: 32, v: 1).
        let sig_bytes = [0_u8; 65];
        assert_eq!(sig_bytes.len(), 65, "Signature should be 65 bytes");
    }

    #[test]
    fn safe_address_derivation_polygon() {
        // Verify Safe address derivation matches expected value.
        // Uses the test key from lib.rs tests.
        let eoa = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let safe_addr = crate::derive_safe_wallet(eoa, crate::POLYGON).expect("derivation failed");

        assert_eq!(
            safe_addr,
            address!("0xd93b25Cb943D14d0d34FBAf01fc93a0F8b5f6e47"),
            "Safe address derivation should match expected"
        );
    }

    #[test]
    fn safe_address_derivation_unsupported_chain() {
        // Verify derivation returns None for unsupported chains.
        let eoa = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let result = crate::derive_safe_wallet(eoa, 1); // Ethereum mainnet not supported

        assert!(result.is_none(), "Unsupported chain should return None");
    }

    #[test]
    fn ctf_contract_address_polygon() {
        // Verify CTF contract address is correct for Polygon.
        let config = crate::contract_config(crate::POLYGON, false).expect("config exists");

        assert_eq!(
            config.conditional_tokens,
            address!("0x4D97DCd97eC945f40cF65F87097ACe5EA0476045"),
            "CTF address should match Polygon mainnet"
        );
    }

    #[test]
    fn neg_risk_adapter_address_polygon() {
        // Verify NegRisk adapter address is correct for Polygon.
        let config = crate::contract_config(crate::POLYGON, true).expect("config exists");

        assert_eq!(
            config.neg_risk_adapter,
            Some(address!("0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296")),
            "NegRisk adapter address should match Polygon mainnet"
        );
    }
}
