//! Authentication Module
//!
//! JWT authentication and authorization with HS256 algorithm

pub mod jwt;
pub mod middleware;

// Re-exports
pub use jwt::{Claims, JwtConfig, JwtManager};
pub use middleware::{AuthContext, AuthMiddleware, OptionalAuthMiddleware, get_auth_context};
