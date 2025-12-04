//! # Database Layer
//!
//! High-performance database layer with PostgreSQL, Redis, and Supabase integration.
//! Provides enterprise-grade database operations with connection pooling, caching,
//! migrations, and transaction support.

pub mod cache;
pub mod config;
pub mod connection;
pub mod database;
pub mod error;
pub mod migrations;
pub mod supabase;
pub mod types;

// Re-export commonly used types
pub use cache::*;
pub use config::*;
pub use connection::*;
pub use database::*;
pub use error::*;
pub use migrations::*;
pub use supabase::*;
pub use types::*;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
