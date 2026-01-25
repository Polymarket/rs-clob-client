#![cfg(feature = "clob")]

mod common;

use std::str::FromStr as _;

use alloy::signers::Signer as _;
use alloy::signers::local::LocalSigner;
use httpmock::MockServer;
use polymarket_client_sdk::POLYGON;
use polymarket_client_sdk::auth::{Credentials, ExposeSecret as _};
use polymarket_client_sdk::clob::{Client, Config};
use polymarket_client_sdk::error::{Kind, Synchronization, Validation};
use reqwest::StatusCode;
use serde_json::json;

use crate::common::{API_KEY, PASSPHRASE, POLY_ADDRESS, PRIVATE_KEY, SECRET, create_authenticated};

#[tokio::test]
async fn authenticate_with_explicit_credentials_should_succeed() -> anyhow::Result<()> {
    let server = MockServer::start();

    let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
    let client = Client::new(&server.base_url(), Config::default())?
        .authentication_builder(&signer)
        .credentials(Credentials::default())
        .authenticate()
        .await?;

    assert_eq!(signer.address(), client.address());

    Ok(())
}

#[tokio::test]
async fn authenticate_with_nonce_should_succeed() -> anyhow::Result<()> {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path("/auth/derive-api-key");
        then.status(StatusCode::OK).json_body(json!({
            "apiKey": API_KEY,
            "passphrase": PASSPHRASE,
            "secret": SECRET
        }));
    });

    let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
    let client = Client::new(&server.base_url(), Config::default())?
        .authentication_builder(&signer)
        .nonce(123)
        .authenticate()
        .await?;

    assert_eq!(signer.address(), client.address());

    mock.assert();

    Ok(())
}

#[tokio::test]
async fn authenticate_with_explicit_credentials_and_nonce_should_fail() -> anyhow::Result<()> {
    let server = MockServer::start();

    let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
    let err = Client::new(&server.base_url(), Config::default())?
        .authentication_builder(&signer)
        .nonce(123)
        .credentials(Credentials::default())
        .authenticate()
        .await
        .unwrap_err();

    let validation_err = err.downcast_ref::<Validation>().unwrap();

    assert_eq!(
        validation_err.reason,
        "Credentials and nonce are both set. If nonce is set, then you must not supply credentials"
    );

    Ok(())
}

#[tokio::test]
async fn authenticated_to_unauthenticated_should_succeed() -> anyhow::Result<()> {
    let server = MockServer::start();
    let client = create_authenticated(&server).await?;

    assert_eq!(client.host().as_str(), format!("{}/", server.base_url()));
    client.deauthenticate().await?;

    Ok(())
}

#[tokio::test]
async fn authenticate_with_multiple_strong_references_should_fail() -> anyhow::Result<()> {
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path("/auth/derive-api-key");
        then.status(StatusCode::OK).json_body(json!({
            "apiKey": API_KEY,
            "passphrase": PASSPHRASE,
            "secret": SECRET
        }));
    });

    let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
    let client = Client::new(&server.base_url(), Config::default())?;

    let _client_clone = client.clone();

    let err = client
        .authentication_builder(&signer)
        .authenticate()
        .await
        .unwrap_err();

    err.downcast_ref::<Synchronization>().unwrap();

    Ok(())
}

#[tokio::test]
async fn deauthenticated_with_multiple_strong_references_should_fail() -> anyhow::Result<()> {
    let server = MockServer::start();
    let client = create_authenticated(&server).await?;

    let _client_clone = client.clone();

    let err = client.deauthenticate().await.unwrap_err();
    let sync_error = err.downcast_ref::<Synchronization>().unwrap();
    assert_eq!(
        sync_error.to_string(),
        "synchronization error: multiple threads are attempting to log in or log out"
    );

    Ok(())
}

#[tokio::test]
async fn create_api_key_should_succeed() -> anyhow::Result<()> {
    let server = MockServer::start();
    let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
    let client = Client::new(&server.base_url(), Config::default())?;

    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::POST)
            .path("/auth/api-key")
            .header(POLY_ADDRESS, signer.address().to_string().to_lowercase());
        then.status(StatusCode::OK).json_body(json!({
            "apiKey": API_KEY.to_string(),
            "passphrase": PASSPHRASE,
            "secret": SECRET
        }));
    });

    let credentials = client.create_api_key(&signer, None).await?;

    assert_eq!(credentials.key(), API_KEY);
    mock.assert();

    Ok(())
}

#[tokio::test]
async fn derive_api_key_should_succeed() -> anyhow::Result<()> {
    let server = MockServer::start();
    let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
    let client = Client::new(&server.base_url(), Config::default())?;

    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path("/auth/derive-api-key")
            .header(POLY_ADDRESS, signer.address().to_string().to_lowercase());
        then.status(StatusCode::OK).json_body(json!({
            "apiKey": API_KEY.to_string(),
            "passphrase": PASSPHRASE,
            "secret": SECRET
        }));
    });

    let credentials = client.derive_api_key(&signer, None).await?;

    assert_eq!(credentials.key(), API_KEY);
    mock.assert();

    Ok(())
}

#[tokio::test]
async fn create_or_derive_api_key_should_succeed() -> anyhow::Result<()> {
    let server = MockServer::start();
    let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
    let client = Client::new(&server.base_url(), Config::default())?;

    let mock = server.mock(|when, then| {
        when.method(httpmock::Method::POST).path("/auth/api-key");
        then.status(StatusCode::NOT_FOUND);
    });
    let mock2 = server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path("/auth/derive-api-key")
            .header(POLY_ADDRESS, signer.address().to_string().to_lowercase());
        then.status(StatusCode::OK).json_body(json!({
            "apiKey": API_KEY.to_string(),
            "passphrase": PASSPHRASE,
            "secret": SECRET
        }));
    });

    let credentials = client.create_or_derive_api_key(&signer, None).await?;

    assert_eq!(credentials.key(), API_KEY);
    mock.assert();
    mock2.assert();

    Ok(())
}

#[tokio::test]
async fn create_or_derive_api_key_should_propagate_network_errors() -> anyhow::Result<()> {
    // Use an invalid host to simulate a network error (connection refused)
    let signer = LocalSigner::from_str(PRIVATE_KEY)?.with_chain_id(Some(POLYGON));
    let client = Client::new("http://127.0.0.1:1", Config::default())?;

    let err = client
        .create_or_derive_api_key(&signer, None)
        .await
        .expect_err("should fail with network error");

    // Network errors should be propagated as Internal errors, not swallowed
    assert_eq!(err.kind(), Kind::Internal);

    Ok(())
}

#[test]
fn credentials_secret_accessor_should_return_secret() {
    let credentials = Credentials::new(API_KEY, SECRET.to_owned(), PASSPHRASE.to_owned());
    assert_eq!(credentials.secret().expose_secret(), SECRET);
}

#[test]
fn credentials_passphrase_accessor_should_return_passphrase() {
    let credentials = Credentials::new(API_KEY, SECRET.to_owned(), PASSPHRASE.to_owned());
    assert_eq!(credentials.passphrase().expose_secret(), PASSPHRASE);
}

#[tokio::test]
async fn authenticated_client_should_expose_credentials() -> anyhow::Result<()> {
    let server = MockServer::start();
    let client = create_authenticated(&server).await?;

    let credentials = client.credentials();

    assert_eq!(credentials.key(), API_KEY);
    assert_eq!(credentials.secret().expose_secret(), SECRET);
    assert_eq!(credentials.passphrase().expose_secret(), PASSPHRASE);

    Ok(())
}
