//! Built-in strategy implementations
//!
//! This module provides ready-to-use trading strategies implementing the
//! `StrategyExecutor` trait. Strategies can be used directly or as templates
//! for custom implementations.

pub mod momentum_strategy;

pub use momentum_strategy::MomentumStrategy;
