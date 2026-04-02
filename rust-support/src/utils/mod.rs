//! Utilities Module
//!
//! Common utilities for the application

pub mod error;
pub mod response;
pub mod validator;

// Re-export commonly used types
pub use error::{AppError, AppResult, ValidationErrorDetail};
pub use response::{
    ApiResponse, ApiError, PaginatedResponse, Meta, PaginationMeta, ErrorDetail, ResponseBuilder,
};
pub use validator::Validator;
