#![cfg(feature = "ctf")]
#![allow(clippy::unwrap_used, reason = "Fine for tests")]

use alloy::primitives::{B256, U256};
use alloy::providers::ProviderBuilder;
use polymarket_client_sdk::POLYGON;
use polymarket_client_sdk::ctf::Client;
use polymarket_client_sdk::ctf::types::*;
use polymarket_client_sdk::types::address;

const RPC_URL: &str = "https://polygon-rpc.com";

#[tokio::test]
async fn get_condition_id() -> anyhow::Result<()> {
    // This is a view function so we can test it with a real provider
    let provider = ProviderBuilder::new().connect(RPC_URL).await?;
    let client = Client::new(provider, POLYGON)?;

    let request = ConditionIdRequest::builder()
        .oracle(address!("0x0000000000000000000000000000000000000001"))
        .question_id(B256::ZERO)
        .outcome_slot_count(U256::from(2))
        .build();

    let response = client.condition_id(&request).await?;

    // Just verify we got a response
    assert_ne!(response.condition_id, B256::ZERO);

    Ok(())
}

#[tokio::test]
async fn get_collection_id() -> anyhow::Result<()> {
    // This is a view function so we can test it with a real provider
    let provider = ProviderBuilder::new().connect(RPC_URL).await?;
    let client = Client::new(provider, POLYGON)?;

    let request = CollectionIdRequest::builder()
        .parent_collection_id(B256::ZERO)
        .condition_id(B256::ZERO)
        .index_set(U256::from(1))
        .build();

    let response = client.collection_id(&request).await?;

    // Just verify we got a response
    assert_ne!(response.collection_id, B256::ZERO);

    Ok(())
}

#[tokio::test]
async fn get_position_id() -> anyhow::Result<()> {
    // This is a pure function so we can test it with a real provider
    let provider = ProviderBuilder::new().connect(RPC_URL).await?;
    let client = Client::new(provider, POLYGON)?;

    let usdc = address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174");

    let request = PositionIdRequest::builder()
        .collateral_token(usdc)
        .collection_id(B256::ZERO)
        .build();

    let response = client.position_id(&request).await?;

    // Just verify we got a response
    assert_ne!(response.position_id, U256::ZERO);

    Ok(())
}

#[tokio::test]
async fn client_creation() {
    let provider = ProviderBuilder::new().connect(RPC_URL).await.unwrap();

    // Test Polygon mainnet
    let client = Client::new(provider.clone(), POLYGON);
    client.unwrap();

    // Test Amoy testnet
    let client = Client::new(provider.clone(), polymarket_client_sdk::AMOY);
    client.unwrap();

    // Test invalid chain
    let client = Client::new(provider, 999);
    client.unwrap_err();
}

#[test]
fn request_builders() {
    // Test GetConditionIdRequest builder
    let request = ConditionIdRequest::builder()
        .oracle(address!("0x0000000000000000000000000000000000000001"))
        .question_id(B256::ZERO)
        .outcome_slot_count(U256::from(2))
        .build();

    assert_eq!(
        request.oracle,
        address!("0x0000000000000000000000000000000000000001")
    );

    // Test SplitPositionRequest builder with default parent_collection_id
    let request = SplitPositionRequest::builder()
        .collateral_token(address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"))
        .condition_id(B256::ZERO)
        .partition(vec![U256::from(1), U256::from(2)])
        .amount(U256::from(1_000_000))
        .build();

    assert_eq!(request.parent_collection_id, B256::ZERO);

    // Test MergePositionsRequest builder
    let request = MergePositionsRequest::builder()
        .collateral_token(address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"))
        .condition_id(B256::ZERO)
        .partition(vec![U256::from(1), U256::from(2)])
        .amount(U256::from(1_000_000))
        .build();

    assert_eq!(request.parent_collection_id, B256::ZERO);

    // Test RedeemPositionsRequest builder
    let request = RedeemPositionsRequest::builder()
        .collateral_token(address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"))
        .condition_id(B256::ZERO)
        .index_sets(vec![U256::from(1)])
        .build();

    assert_eq!(request.parent_collection_id, B256::ZERO);
}
#[test]
fn binary_market_convenience_methods() {
    let usdc = address!("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174");
    let condition_id = B256::ZERO;

    // Test SplitPositionRequest::for_binary_market
    let request =
        SplitPositionRequest::for_binary_market(usdc, condition_id, U256::from(1_000_000));

    assert_eq!(request.collateral_token, usdc);
    assert_eq!(request.condition_id, condition_id);
    assert_eq!(request.partition, vec![U256::from(1), U256::from(2)]);
    assert_eq!(request.amount, U256::from(1_000_000));
    assert_eq!(request.parent_collection_id, B256::ZERO);

    // Test MergePositionsRequest::for_binary_market
    let request =
        MergePositionsRequest::for_binary_market(usdc, condition_id, U256::from(1_000_000));

    assert_eq!(request.collateral_token, usdc);
    assert_eq!(request.condition_id, condition_id);
    assert_eq!(request.partition, vec![U256::from(1), U256::from(2)]);
    assert_eq!(request.amount, U256::from(1_000_000));

    // Test RedeemPositionsRequest::for_binary_market
    let request = RedeemPositionsRequest::for_binary_market(usdc, condition_id);

    assert_eq!(request.collateral_token, usdc);
    assert_eq!(request.condition_id, condition_id);
    assert_eq!(request.index_sets, vec![U256::from(1), U256::from(2)]);
}

#[test]
fn neg_risk_request_builder() {
    let condition_id = B256::ZERO;
    let amounts = vec![U256::from(500_000), U256::from(500_000)];

    let request = RedeemNegRiskRequest::builder()
        .condition_id(condition_id)
        .amounts(amounts.clone())
        .build();

    assert_eq!(request.condition_id, condition_id);
    assert_eq!(request.amounts, amounts);
}

#[tokio::test]
async fn client_with_neg_risk() -> anyhow::Result<()> {
    let provider = ProviderBuilder::new().connect(RPC_URL).await?;

    // Test creating client with NegRisk support
    let client = Client::with_neg_risk(provider, POLYGON);
    client.unwrap();

    Ok(())
}
