//! Polymarket Gamma API client and types.
//!
//! This module provides a client for interacting with the Polymarket Gamma API,
//! which offers HTTP endpoints for querying events, markets, tags, series,
//! comments, profiles, and search functionality.
//!
//! # Overview
//!
//! The Gamma API provides market and event metadata for Polymarket. It is
//! separate from the CLOB (Central Limit Order Book) API which handles trading.
//!
//! ## Available Endpoints
//!
//! | Endpoint | Description |
//! |----------|-------------|
//! | `/status` | Health check |
//! | `/teams` | List sports teams |
//! | `/sports` | Get sports metadata |
//! | `/sports/market-types` | Get valid sports market types |
//! | `/tags` | List tags |
//! | `/tags/{id}` | Get tag by ID |
//! | `/tags/slug/{slug}` | Get tag by slug |
//! | `/tags/{id}/related-tags` | Get related tag relationships |
//! | `/events` | List events |
//! | `/events/{id}` | Get event by ID |
//! | `/events/slug/{slug}` | Get event by slug |
//! | `/events/{id}/tags` | Get event tags |
//! | `/markets` | List markets |
//! | `/markets/{id}` | Get market by ID |
//! | `/markets/slug/{slug}` | Get market by slug |
//! | `/markets/{id}/tags` | Get market tags |
//! | `/series` | List series |
//! | `/series/{id}` | Get series by ID |
//! | `/comments` | List comments |
//! | `/comments/{id}` | Get comments by ID |
//! | `/public-profile` | Get public profile |
//! | `/public-search` | Search markets, events, and profiles |
//!
//! # Example
//!
//! ```no_run
//! use polymarket_client_sdk::gamma::{Client, types::EventsRequest};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client with the default endpoint
//! let client = Client::default();
//!
//! // Build a request for active events
//! let request = EventsRequest::builder()
//!     .active(true)
//!     .limit(10)
//!     .build();
//!
//! // Fetch events
//! let events = client.events(&request).await?;
//!
//! for event in events {
//!     println!("{}: {:?}", event.id, event.title);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # API Base URL
//!
//! The default API endpoint is `https://gamma-api.polymarket.com`.

pub mod client;
pub mod types;

pub use client::Client;
