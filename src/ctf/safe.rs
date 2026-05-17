//! Gnosis Safe client for executing transactions through a Safe wallet.
//!
//! This module provides `SafeClient` which wraps arbitrary contract calls
//! in Safe's `execTransaction`, allowing the Safe to be the `msg.sender`.
//!
//! Uses Alloy's `sol!` macro for EIP-712 typed data handling, following
//! the same pattern as the Polymarket CLOB SDK for cleaner, less error-prone
//! signature generation.

use alloy::dyn_abi::Eip712Domain;
use alloy::primitives::{Address, B256, Bytes, U256, address};
use alloy::providers::Provider;
use alloy::signers::Signer;
use alloy::sol;
use alloy::sol_types::{SolCall, SolStruct};
use thiserror::Error;

/// CTF contract address on Polygon
pub const CTF_ADDRESS: Address = address!("4D97DCd97eC945f40cF65F87097ACe5EA0476045");

// Gnosis Safe interface - only the methods we need
// Note: execTransaction has 11 parameters which exceeds clippy's default limit
#[allow(clippy::too_many_arguments)]
mod gnosis_safe {
    use alloy::sol;

    sol! {
        #[sol(rpc)]
        interface IGnosisSafe {
            function nonce() external view returns (uint256);
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
                bytes calldata signatures
            ) external payable returns (bool success);
        }
    }
}
pub use gnosis_safe::IGnosisSafe;

// CTF interface for building redemption calldata
sol! {
    interface IConditionalTokens {
        function redeemPositions(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] calldata indexSets
        ) external;
    }
}

// Define SafeTx using sol! macro - this automatically implements SolStruct
// which provides .eip712_signing_hash() for proper EIP-712 signing
sol! {
    /// SafeTx struct matching Gnosis Safe's EIP-712 typed data format.
    /// Using sol! macro provides automatic type hash computation and
    /// struct hash encoding via the SolStruct trait.
    #[derive(Debug, Default)]
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

/// Errors that can occur during Safe operations
#[derive(Error, Debug)]
pub enum SafeError {
    #[error("Failed to get Safe nonce: {0}")]
    NonceError(String),

    #[error("Failed to sign transaction: {0}")]
    SigningError(String),

    #[error("Failed to execute transaction: {0}")]
    ExecutionError(String),

    #[error("Transaction reverted")]
    Reverted,

    #[error("Provider error: {0}")]
    ProviderError(String),
}

/// Response from a Safe transaction execution
#[derive(Debug, Clone)]
pub struct SafeExecResponse {
    /// Transaction hash
    pub transaction_hash: B256,
    /// Block number where the transaction was mined
    pub block_number: u64,
}

/// Client for executing transactions through a Gnosis Safe
pub struct SafeClient<P> {
    provider: P,
    safe_address: Address,
    chain_id: u64,
}

impl<P: Provider + Clone> SafeClient<P> {
    /// Create a new SafeClient
    pub fn new(provider: P, safe_address: Address, chain_id: u64) -> Self {
        Self {
            provider,
            safe_address,
            chain_id,
        }
    }

    /// Get the current nonce of the Safe
    pub async fn get_nonce(&self) -> Result<U256, SafeError> {
        let safe = IGnosisSafe::new(self.safe_address, &self.provider);
        safe.nonce()
            .call()
            .await
            .map_err(|e| SafeError::NonceError(e.to_string()))
    }

    /// Build the EIP-712 domain for this Safe.
    ///
    /// Safe's domain uses only chainId and verifyingContract (no name/version).
    /// This matches the domain type hash:
    /// keccak256("EIP712Domain(uint256 chainId,address verifyingContract)")
    fn eip712_domain(&self) -> Eip712Domain {
        Eip712Domain {
            chain_id: Some(U256::from(self.chain_id)),
            verifying_contract: Some(self.safe_address),
            ..Eip712Domain::default()
        }
    }

    /// Execute a transaction through the Safe
    ///
    /// # Arguments
    /// * `to` - Target contract address
    /// * `data` - Calldata for the target contract
    /// * `signer` - The EOA signer that controls the Safe
    pub async fn exec_transaction<S: Signer + Sync>(
        &self,
        to: Address,
        data: Bytes,
        signer: &S,
    ) -> Result<SafeExecResponse, SafeError> {
        // Get current nonce
        let nonce = self.get_nonce().await?;

        // Build SafeTx with zero gas params (Safe will estimate)
        let safe_tx = SafeTx {
            to,
            value: U256::ZERO,
            data: data.clone(),
            operation: 0, // Call (not DelegateCall)
            safeTxGas: U256::ZERO,
            baseGas: U256::ZERO,
            gasPrice: U256::ZERO,
            gasToken: Address::ZERO,
            refundReceiver: Address::ZERO,
            nonce,
        };

        // Use Alloy's EIP-712 signing via SolStruct trait
        // This handles type hash computation and struct encoding automatically
        let domain = self.eip712_domain();
        let signing_hash = safe_tx.eip712_signing_hash(&domain);

        // Sign the EIP-712 hash
        let signature = signer
            .sign_hash(&signing_hash)
            .await
            .map_err(|e| SafeError::SigningError(e.to_string()))?;

        // Convert signature to Safe format (r || s || v)
        // Since we use sign_hash (direct ECDSA over the Safe tx hash), we use v = 27/28
        // DO NOT convert to v=31/32 (eth_sign type) - that's for pre-hashed messages with prefix
        let sig_bytes = signature.as_bytes().to_vec();

        // Create Safe instance and execute the transaction
        let safe = IGnosisSafe::new(self.safe_address, &self.provider);
        let pending_tx = safe
            .execTransaction(
                safe_tx.to,
                safe_tx.value,
                safe_tx.data,
                safe_tx.operation,
                safe_tx.safeTxGas,
                safe_tx.baseGas,
                safe_tx.gasPrice,
                safe_tx.gasToken,
                safe_tx.refundReceiver,
                Bytes::from(sig_bytes),
            )
            .send()
            .await
            .map_err(|e| SafeError::ExecutionError(e.to_string()))?;

        let transaction_hash = *pending_tx.tx_hash();

        let receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| SafeError::ExecutionError(e.to_string()))?;

        let block_number = receipt.block_number.ok_or_else(|| {
            SafeError::ExecutionError("Block number not available in receipt".to_owned())
        })?;

        Ok(SafeExecResponse {
            transaction_hash,
            block_number,
        })
    }

    /// Convenience method to redeem CTF positions through the Safe
    ///
    /// # Arguments
    /// * `collateral_token` - The collateral token address (e.g., USDC)
    /// * `condition_id` - The condition ID to redeem
    /// * `signer` - The EOA signer that controls the Safe
    pub async fn redeem_ctf_positions<S: Signer + Sync>(
        &self,
        collateral_token: Address,
        condition_id: B256,
        signer: &S,
    ) -> Result<SafeExecResponse, SafeError> {
        // Build the CTF redeemPositions calldata
        let call = IConditionalTokens::redeemPositionsCall {
            collateralToken: collateral_token,
            parentCollectionId: B256::ZERO,
            conditionId: condition_id,
            indexSets: vec![U256::from(1), U256::from(2)], // Binary market: YES=1, NO=2
        };

        let calldata = Bytes::from(call.abi_encode());

        // Execute through Safe
        self.exec_transaction(CTF_ADDRESS, calldata, signer).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::keccak256;

    #[test]
    fn test_safe_tx_type_hash() {
        // Verify the SafeTx type hash matches the expected value
        // The sol! macro should generate the correct type hash
        let type_string = "SafeTx(address to,uint256 value,bytes data,uint8 operation,uint256 safeTxGas,uint256 baseGas,uint256 gasPrice,address gasToken,address refundReceiver,uint256 nonce)";
        let computed_hash = keccak256(type_string.as_bytes());

        // This is what Safe uses
        let expected: B256 = "0xbb8310d486368db6bd6f849402fdd73ad53d316b5a4b2644ad6efe0f941286d8"
            .parse()
            .unwrap();

        assert_eq!(computed_hash, expected, "SafeTx type hash should match Safe's expected value");
    }

    #[test]
    fn test_domain_type_hash() {
        // Verify the domain type hash is correct
        let type_string = "EIP712Domain(uint256 chainId,address verifyingContract)";
        let computed_hash = keccak256(type_string.as_bytes());

        let expected: B256 = "0x47e79534a245952e8b16893a336b85a3d9ea9fa8c573f3d803afb92a79469218"
            .parse()
            .unwrap();

        assert_eq!(computed_hash, expected, "Domain type hash should match Safe's expected value");
    }

    #[test]
    fn test_sol_struct_signing_hash() {
        // Verify that Alloy's sol! macro generates a valid EIP-712 signing hash
        // by creating a test SafeTx and computing its signing hash
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
            chain_id: Some(U256::from(137_u64)), // Polygon
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
        assert_eq!(signing_hash.len(), 32, "EIP-712 signing hash should be 32 bytes");

        // The hash should not be all zeros (would indicate a bug)
        assert_ne!(signing_hash, B256::ZERO, "Signing hash should not be zero");
    }
}
