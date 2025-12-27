use std::backtrace::Backtrace;
use std::error::Error as StdError;
use std::fmt;

use alloy::primitives::ChainId;
use alloy::primitives::ruint::ParseError;
use hmac::digest::InvalidLength;
use reqwest::{Method, StatusCode, header};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Error related to non-successful HTTP call
    Status,
    /// Error related to invalid state within polymarket-client-sdk
    Validation,
    /// Error related to synchronization of authenticated clients logging in and out
    Synchronization,
    /// Internal error from dependencies
    Internal,
    /// Error related to WebSocket connections
    WebSocket,
}

#[derive(Debug)]
pub struct Error {
    kind: Kind,
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    backtrace: Backtrace,
}

impl Error {
    pub fn with_source<S: StdError + Send + Sync + 'static>(kind: Kind, source: S) -> Self {
        Self {
            kind,
            source: Some(Box::new(source)),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    pub fn inner(&self) -> Option<&(dyn StdError + Send + Sync + 'static)> {
        self.source.as_deref()
    }

    pub fn downcast_ref<E: StdError + 'static>(&self) -> Option<&E> {
        let e = self.source.as_deref()?;
        e.downcast_ref::<E>()
    }

    pub fn validation<S: Into<String>>(message: S) -> Self {
        Validation {
            reason: message.into(),
        }
        .into()
    }

    pub fn status<S: Into<String>>(
        status_code: StatusCode,
        method: Method,
        path: String,
        message: S,
    ) -> Self {
        Status {
            status_code,
            method,
            path,
            message: message.into(),
        }
        .into()
    }

    #[must_use]
    pub fn missing_contract_config(chain_id: ChainId, neg_risk: bool) -> Self {
        MissingContractConfig { chain_id, neg_risk }.into()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            Some(src) => write!(f, "{:?}: {}", self.kind, src),
            None => write!(f, "{:?}", self.kind),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn StdError + 'static))
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct Status {
    pub status_code: StatusCode,
    pub method: Method,
    pub path: String,
    pub message: String,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error({}) making {} call to {} with {}",
            self.status_code, self.method, self.path, self.message
        )
    }
}

impl StdError for Status {}

#[non_exhaustive]
#[derive(Debug)]
pub struct Validation {
    pub reason: String,
}

impl fmt::Display for Validation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid: {}", self.reason)
    }
}

impl StdError for Validation {}

#[non_exhaustive]
#[derive(Debug)]
pub struct Synchronization;

impl fmt::Display for Synchronization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "synchronization error: multiple threads are attempting to log in or log out"
        )
    }
}

impl StdError for Synchronization {}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct MissingContractConfig {
    pub chain_id: ChainId,
    pub neg_risk: bool,
}

impl fmt::Display for MissingContractConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "missing contract config for chain id {} with neg_risk = {}",
            self.chain_id, self.neg_risk,
        )
    }
}

impl std::error::Error for MissingContractConfig {}

impl From<MissingContractConfig> for Error {
    fn from(err: MissingContractConfig) -> Self {
        Error::with_source(Kind::Internal, err)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::with_source(Kind::Internal, e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::with_source(Kind::Internal, e)
    }
}

impl From<header::InvalidHeaderValue> for Error {
    fn from(e: header::InvalidHeaderValue) -> Self {
        Error::with_source(Kind::Internal, e)
    }
}

impl From<InvalidLength> for Error {
    fn from(e: InvalidLength) -> Self {
        Error::with_source(Kind::Internal, e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::with_source(Kind::Internal, e)
    }
}

impl From<alloy::signers::Error> for Error {
    fn from(e: alloy::signers::Error) -> Self {
        Error::with_source(Kind::Internal, e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Error::with_source(Kind::Internal, e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::with_source(Kind::Internal, e)
    }
}

impl From<Validation> for Error {
    fn from(err: Validation) -> Self {
        Error::with_source(Kind::Validation, err)
    }
}

impl From<Status> for Error {
    fn from(err: Status) -> Self {
        Error::with_source(Kind::Status, err)
    }
}

impl From<Synchronization> for Error {
    fn from(err: Synchronization) -> Self {
        Error::with_source(Kind::Synchronization, err)
    }
}
