//! Rust Support Library for Pantheon App
//!
//! This library provides core functionality for the Pantheon multi-tenant application:
//! - Base traits for entities, resources, controllers, and services
//! - Database management with multi-tenant support
//! - Configuration system with hierarchy
//! - JWT authentication and authorization
//! - Service provider pattern for module initialization
//! - Utilities for API responses, error handling, validation, and logging

pub mod base;
pub mod database;
pub mod config;
pub mod auth;
pub mod provider;
pub mod utils;
pub mod queue;

// Re-export commonly used types
pub use base::{
    entity::{BaseEntity, EntityConnection},
    resource::{BaseResource, ViewResource, ShowResource},
    controller::BaseController,
    service::BaseService,
};

pub use database::{
    manager::DatabaseManager,
    connection::ConnectionManager,
};

pub use config::{
    loader::ConfigLoader,
    merger::ConfigMerger,
    resolver::ConfigResolver,
};

pub use auth::{
    jwt::{JwtManager, Claims},
    middleware::AuthMiddleware,
};

pub use provider::{
    service_provider::{ServiceProvider, ServiceProviderRegistry},
};

pub use utils::{
    response::{ApiResponse, ApiError, PaginatedResponse},
    error::AppError,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize tracing/logging
pub fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_support=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
