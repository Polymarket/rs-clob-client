//! Types for the Polymarket Data API.
//!
//! This module contains all types used by the Data API client, organized into:
//!
//! - **Common types**: Fundamental types like [`Address`], [`Hash64`], [`EventId`],
//!   as well as enums for sorting, filtering, and pagination.
//!
//! - **Request types**: Builder-pattern structs for each API endpoint
//!   (e.g., [`PositionsRequest`], [`TradesRequest`]).
//!
//! - **Response types**: Structs representing API responses
//!   (e.g., [`Position`], [`Trade`], [`Activity`]).
//!
//! # Request Building
//!
//! All request types use the builder pattern via the [`bon`](https://docs.rs/bon) crate:
//!
//! ```
//! use polymarket_client_sdk::data_api::types::{PositionsRequest, Address, PositionSortBy, SortDirection};
//!
//! let request = PositionsRequest::builder()
//!     .user(Address::new("0x56687bf447db6ffa42ffe2204a05edaa20f55839").unwrap())
//!     .sort_by(PositionSortBy::CashPnl)
//!     .sort_direction(SortDirection::Desc)
//!     .build();
//! ```
//!
//! # Type Safety
//!
//! The types in this module provide compile-time validation where possible:
//!
//! - [`Address`] validates Ethereum address format (0x + 40 hex chars)
//! - [`Hash64`] validates condition ID format (0x + 64 hex chars)
//! - [`EventId`] ensures event IDs are >= 1
//! - Bounded integer types (e.g., [`PositionsLimit`]) enforce API limits

mod common;
mod requests;
mod responses;

pub use common::{
    ActivityLimit, ActivityOffset, ActivitySortBy, ActivityType, Address, AddressError,
    BoundedIntError, BuilderLeaderboardLimit, BuilderLeaderboardOffset, ClosedPositionSortBy,
    ClosedPositionsLimit, ClosedPositionsOffset, EventId, EventIdError, FilterType, Hash64,
    Hash64Error, HoldersLimit, HoldersMinBalance, LeaderboardCategory, LeaderboardOrderBy,
    MarketFilter, PositionSortBy, PositionsLimit, PositionsOffset, Side, SortDirection, TimePeriod,
    Title, TitleError, TradeFilter, TradeFilterError, TraderLeaderboardLimit,
    TraderLeaderboardOffset, TradesLimit, TradesOffset,
};
pub use requests::{
    ActivityRequest, BuilderLeaderboardRequest, BuilderVolumeRequest, ClosedPositionsRequest,
    HoldersRequest, LiveVolumeRequest, OpenInterestRequest, PositionsRequest, QueryParams,
    TradedRequest, TraderLeaderboardRequest, TradesRequest, ValueRequest,
};
pub use responses::{
    Activity, BuilderLeaderboardEntry, BuilderVolumeEntry, ClosedPosition, ErrorResponse,
    HealthResponse, Holder, LiveVolume, MarketVolume, MetaHolder, OpenInterest, Position, Trade,
    Traded, TraderLeaderboardEntry, Value,
};
