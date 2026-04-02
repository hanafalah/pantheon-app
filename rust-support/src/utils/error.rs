//! Error Handling
//!
//! Centralized error types for the application

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use std::fmt;

/// Application Error Type
#[derive(Debug)]
pub enum AppError {
    /// Database error
    DatabaseError(String),
    /// Authentication error
    AuthError(String),
    /// Authorization error (permission denied)
    AuthorizationError(String),
    /// Validation error
    ValidationError(Vec<ValidationErrorDetail>),
    /// Not found error
    NotFound(String),
    /// Conflict error (e.g., duplicate entry)
    Conflict(String),
    /// Bad request error
    BadRequest(String),
    /// Internal server error
    InternalError(String),
    /// External service error
    ExternalServiceError(String),
    /// Configuration error
    ConfigError(String),
    /// Migration error
    MigrationError(String),
    /// Connection error
    ConnectionError(String),
}

/// Validation error detail
#[derive(Debug, serde::Serialize)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            AppError::AuthorizationError(msg) => write!(f, "Authorization error: {}", msg),
            AppError::ValidationError(errors) => {
                write!(f, "Validation error: {} fields", errors.len())
            }
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::ExternalServiceError(msg) => write!(f, "External service error: {}", msg),
            AppError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            AppError::MigrationError(msg) => write!(f, "Migration error: {}", msg),
            AppError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type, message, details) = match self {
            AppError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database_error",
                msg.clone(),
                None,
            ),
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, "auth_error", msg.clone(), None),
            AppError::AuthorizationError(msg) => {
                (StatusCode::FORBIDDEN, "authorization_error", msg.clone(), None)
            }
            AppError::ValidationError(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_error",
                "Validation failed".to_string(),
                Some(serde_json::to_value(errors).unwrap()),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone(), None),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone(), None),
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "bad_request", msg.clone(), None)
            }
            AppError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg.clone(),
                None,
            ),
            AppError::ExternalServiceError(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "external_service_error",
                msg.clone(),
                None,
            ),
            AppError::ConfigError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_error",
                msg.clone(),
                None,
            ),
            AppError::MigrationError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "migration_error",
                msg.clone(),
                None,
            ),
            AppError::ConnectionError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "connection_error",
                msg.clone(),
                None,
            ),
        };

        let mut body = serde_json::json!({
            "error": {
                "type": error_type,
                "message": message,
            }
        });

        if let Some(details) = details {
            body["error"]["details"] = details;
        }

        HttpResponse::build(status).json(body)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthError(_) => StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError(_) => StatusCode::FORBIDDEN,
            AppError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalServiceError(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::MigrationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ConnectionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// Conversions from other error types

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => AppError::NotFound("Record not found".to_string()),
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

impl From<r2d2::Error> for AppError {
    fn from(err: r2d2::Error) -> Self {
        AppError::ConnectionError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::InternalError(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::InternalError(format!("IO error: {}", err))
    }
}

impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        AppError::ConfigError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::AuthError(format!("JWT error: {}", err))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::InternalError(format!("Bcrypt error: {}", err))
    }
}

/// Result type alias
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_display() {
        let error = AppError::NotFound("User not found".to_string());
        assert_eq!(error.to_string(), "Not found: User not found");

        let error = AppError::ValidationError(vec![ValidationErrorDetail {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
        }]);
        assert_eq!(error.to_string(), "Validation error: 1 fields");
    }

    #[test]
    fn test_app_error_status_code() {
        assert_eq!(
            AppError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::AuthError("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::AuthorizationError("test".to_string()).status_code(),
            StatusCode::FORBIDDEN
        );
    }
}
