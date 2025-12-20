#![expect(
    clippy::module_name_repetitions,
    reason = "Re-exported names intentionally match their modules for API clarity"
)]

pub mod client;
pub mod types;

pub use client::GammaClient;
