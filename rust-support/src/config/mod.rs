//! Configuration Module
//!
//! Configuration system with hierarchy merging
//! TODO: Full implementation

pub mod loader;
pub mod merger;
pub mod resolver;

// Re-exports
pub use loader::ConfigLoader;
pub use merger::ConfigMerger;
pub use resolver::ConfigResolver;
