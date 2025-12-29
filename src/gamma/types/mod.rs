//! Types for the Polymarket Gamma API.
//!
//! This module contains all types used by the Gamma API client, organized into:
//!
//! - **Common types**: Shared data structures used across requests and responses,
//!   as well as enums for filtering and categorization.
//!
//! - **Request types**: Builder-pattern structs for each API endpoint
//!   (e.g., [`EventsRequest`], [`MarketsRequest`]).
//!
//! - **Response types**: Structs representing API responses
//!   (e.g., [`Event`], [`Market`], [`Tag`]).
//!
//! # Request Building
//!
//! All request types use the builder pattern via the [`bon`](https://docs.rs/bon) crate:
//!
//! ```
//! use polymarket_client_sdk::gamma::types::request::{EventsRequest, MarketsRequest};
//!
//! // Simple request with defaults
//! let events = EventsRequest::builder().build();
//!
//! // Request with filters
//! let markets = MarketsRequest::builder()
//!     .limit(10)
//!     .closed(false)
//!     .build();
//! ```

use serde::{Deserialize, Serialize};

pub mod request;
pub mod response;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[non_exhaustive]
pub enum RelatedTagsStatus {
    Active,
    Closed,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum_macros::Display)]
#[non_exhaustive]
pub enum ParentEntityType {
    Event,
    Series,
    #[serde(rename = "market")]
    #[strum(serialize = "market")]
    Market,
}
