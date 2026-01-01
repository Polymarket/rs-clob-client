//! CTF-specific error types.

use std::error::Error as StdError;
use std::fmt;

/// CTF-specific errors.
#[derive(Debug)]
pub enum CtfError {
    /// Contract call failed
    ContractCall(String),
}

impl fmt::Display for CtfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContractCall(msg) => write!(f, "CTF contract call failed: {msg}"),
        }
    }
}

impl StdError for CtfError {}

impl From<CtfError> for crate::error::Error {
    fn from(err: CtfError) -> Self {
        crate::error::Error::with_source(crate::error::Kind::Internal, err)
    }
}
