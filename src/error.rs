use std::backtrace::Backtrace;
use std::error::Error as StdError;
use std::fmt;

use alloy::primitives::ChainId;
use alloy::primitives::ruint::ParseError;
use hmac::digest::InvalidLength;
/// HTTP method type, re-exported for use with error inspection.
pub use reqwest::Method;
/// HTTP status code type, re-exported for use with error inspection.
pub use reqwest::StatusCode;
use reqwest::header;

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
    /// Error related to geographic restrictions blocking access
    Geoblock,
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

/// Error indicating that the user is blocked from accessing Polymarket due to geographic
/// restrictions.
///
/// This error contains information about the user's detected location.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Geoblock {
    /// The detected IP address
    pub ip: String,
    /// ISO 3166-1 alpha-2 country code
    pub country: String,
    /// Region/state code
    pub region: String,
}

impl fmt::Display for Geoblock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "access blocked from country: {}, region: {}, ip: {}",
            self.country, self.region, self.ip
        )
    }
}

impl StdError for Geoblock {}

impl From<Geoblock> for Error {
    fn from(err: Geoblock) -> Self {
        Error::with_source(Kind::Geoblock, err)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geoblock_display_should_succeed() {
        let geoblock = Geoblock {
            ip: "192.168.1.1".to_owned(),
            country: "US".to_owned(),
            region: "NY".to_owned(),
        };

        assert_eq!(
            geoblock.to_string(),
            "access blocked from country: US, region: NY, ip: 192.168.1.1"
        );
    }

    #[test]
    fn geoblock_into_error_should_succeed() {
        let geoblock = Geoblock {
            ip: "10.0.0.1".to_owned(),
            country: "CU".to_owned(),
            region: "HAV".to_owned(),
        };

        let error: Error = geoblock.into();

        assert_eq!(error.kind(), Kind::Geoblock);
        assert!(error.to_string().contains("CU"));
    }
}
