//! Integration test suite entry point
//!
//! Run with: cargo test --test integration

#[path = "integration/db_tests.rs"]
mod db_tests;

#[path = "integration/redis_tests.rs"]
mod redis_tests;
