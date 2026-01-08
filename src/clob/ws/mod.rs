#![expect(
    clippy::module_name_repetitions,
    reason = "Re-exported names intentionally match their modules for API clarity"
)]

pub mod client;
pub mod interest;
pub mod subscription;
pub mod types;

// Re-export commonly used types
pub use client::Client;
pub use subscription::{ChannelType, SubscriptionInfo, SubscriptionTarget};
pub use types::request::SubscriptionRequest;
pub use types::response::{
    BestBidAsk, BookUpdate, EventMessage, LastTradePrice, MakerOrder, MarketResolved,
    MidpointUpdate, NewMarket, OrderMessage, OrderStatus, PriceChange, PriceChangeBatchEntry,
    TickSizeChange, TradeMessage, WsMessage,
};

pub use crate::ws::WsError;
