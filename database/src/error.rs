//! Database error types
//!
//! This module provides error types for database operations.

use thiserror::Error;

/// Database-related errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),
}

/// Type alias for database results
pub type DatabaseResult<T> = Result<T, DatabaseError>;
