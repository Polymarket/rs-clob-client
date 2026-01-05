//! Core WebSocket infrastructure.
//!
//! This module provides generic connection management that can be
//! specialized for different WebSocket services using traits and the strategy pattern.
//!
//! # Architecture
//!
//! - [`ConnectionManager`]: Generic WebSocket connection handler with heartbeat and reconnection
//! - [`MessageParser`]: Trait for parsing incoming WebSocket messages
//!
//! # Example
//!
//! ```ignore
//! // Define your message type
//! #[derive(Clone, Debug, Deserialize)]
//! enum MyMessage { /* ... */ }
//!
//! let connection = ConnectionManager::new(endpoint, config, SimpleParser)?;
//! let subscriptions = SubscriptionManager::new(connection);
//! ```

pub mod config;
pub mod connection;
pub mod traits;

pub use connection::ConnectionManager;
pub use traits::*;
