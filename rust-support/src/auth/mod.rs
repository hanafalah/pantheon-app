//! Authentication Module
//! TODO: Full implementation

pub mod jwt;
pub mod middleware;

// Re-exports
pub use jwt::{JwtManager, Claims};
pub use middleware::AuthMiddleware;
