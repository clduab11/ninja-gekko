//! # Ninja Gekko - Next-Generation Autonomous Trading Bot
//!
//! Ninja Gekko is a high-performance, memory-safe autonomous trading bot built in Rust
//! with native MCP (Model Context Protocol) integration and advanced neural network capabilities.
//!
//! ## Key Features
//!
//! - **🦀 Rust Performance**: Memory-safe, zero-cost abstractions
//! - **🧠 Neural Intelligence**: ruv-FANN integration with 84.8% accuracy
//! - **🤖 Swarm Intelligence**: flow-nexus distributed decision making
//! - **🎭 MCP Integration**: 70+ native protocol servers
//! - **⚡ GPU Acceleration**: CUDA and Metal Performance Shaders
//! - **🌐 WebAssembly**: Universal deployment capability
//!
//! ## Architecture
//!
//! The system is organized into modular crates:
//! - `core`: Core types, error handling, and trading system orchestration
//! - `database`: Database operations, caching, and real-time subscriptions
//! - `api`: REST and WebSocket APIs with comprehensive security
//!
//! ## Quick Start
//!
//! ```rust
//! use ninja_gekko::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the trading bot
//!     let bot = NinjaGekko::builder()
//!         .mode(OperationMode::Precision)
//!         .neural_backend(NeuralBackend::RuvFann)
//!         .mcp_servers(vec![
//!             "playwright".to_string(),
//!             "filesystem".to_string(),
//!             "github".to_string(),
//!         ])
//!         .build()
//!         .await?;
//!     
//!     // Start autonomous operation
//!     bot.start().await?;
//!     
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    unused_qualifications,
    missing_debug_implementations
)]

pub mod config;
pub mod core;
// pub mod mcp; // Legacy module disabled in favor of crates/mcp-client
pub mod neural;
pub mod swarm;
pub mod trading;
pub mod utils;

/// Re-exports for convenience
pub mod prelude {
    pub use crate::core::{NinjaGekko, OperationMode};
    // pub use crate::mcp::McpManager;
    pub use crate::neural::NeuralBackend;
    pub use crate::swarm::SwarmIntelligence;
    pub use crate::trading::{Strategy, TradingEngine};
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_INFO: &str = concat!(
    "Ninja Gekko v",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(BUILD_INFO.contains("Ninja Gekko"));
    }
}
