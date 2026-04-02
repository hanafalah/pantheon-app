//! Database Module
//!
//! Database management with multi-tenant support and connection pooling
//!
//! # Features
//! - Multi-database support (core, hq, group, tenant)
//! - R2D2 connection pooling
//! - Cluster schema management
//! - Health checks and statistics

pub mod connection;
pub mod manager;

// Re-exports
pub use connection::{ConnectionManager, DatabaseConfig, PgPool, PoolStats};
pub use manager::DatabaseManager;
