//! Configuration Module
//!
//! Configuration system with hierarchy merging and context resolution
//!
//! # Features
//! - Load TOML configuration files
//! - Deep merge multiple configurations
//! - Hierarchy support: base → repository → project → group → tenant
//! - Caching for performance
//! - Multiple array merge strategies

pub mod loader;
pub mod merger;
pub mod resolver;

// Re-exports
pub use loader::ConfigLoader;
pub use merger::{ArrayMergeStrategy, ConfigMerger};
pub use resolver::{ConfigContext, ConfigResolver};
