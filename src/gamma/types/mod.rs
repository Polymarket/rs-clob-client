//! Types for the Polymarket Gamma API.
//!
//! This module contains all types used by the Gamma API client, organized into:
//!
//! - **Common types**: Fundamental types like [`Address`], as well as enums
//!   for filtering and categorization.
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
//! use polymarket_client_sdk::gamma::types::{EventsRequest, MarketsRequest};
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

mod common;
mod requests;
mod responses;

pub use common::{Address, AddressError, ParentEntityType, RelatedTagsStatus};
pub use requests::{
    CommentsByIdRequest, CommentsByUserAddressRequest, CommentsRequest, EventByIdRequest,
    EventBySlugRequest, EventTagsRequest, EventsRequest, MarketByIdRequest, MarketBySlugRequest,
    MarketTagsRequest, MarketsInformationBody, MarketsRequest, PublicProfileRequest, QueryParams,
    RelatedTagsByIdRequest, RelatedTagsBySlugRequest, SearchRequest, SeriesByIdRequest,
    SeriesListRequest, TagByIdRequest, TagBySlugRequest, TagsRequest, TeamsRequest,
};
pub use responses::{
    Category, Chat, Collection, Comment, CommentPosition, CommentProfile, Count, Event,
    EventCreator, EventTweetCount, EventsPagination, HealthResponse, ImageOptimization, Market,
    MarketDescription, Pagination, Profile, PublicProfile, PublicProfileError, PublicProfileUser,
    Reaction, RelatedTag, SearchResults, SearchTag, Series, SeriesSummary,
    SportsMarketTypesResponse, SportsMetadata, Tag, Team, Template,
};
