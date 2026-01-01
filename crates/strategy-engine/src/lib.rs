//! Strategy engine crate providing WASM sandboxed execution for user-defined strategies.

pub mod event_bridge;
pub mod indicators;
pub mod runner;
pub mod sandbox;
pub mod strategies;
pub mod traits;

pub use event_bridge::StrategyEventBridge;
pub use runner::ThreadSafeStrategyRunner;
pub use sandbox::{WasmStrategyConfig, WasmStrategyInstance, WasmStrategyModule};
pub use strategies::MomentumStrategy;
pub use traits::{
    MarketSnapshot, StrategyContext, StrategyDecision, StrategyError, StrategyExecutor,
    StrategyInitContext, StrategyMetrics,
};

#[cfg(test)]
mod tests;
