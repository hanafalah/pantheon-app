//! API Response Helpers
//!
//! Standardized API response structures

use actix_web::{HttpResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standard API Response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}

/// Metadata for responses
#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

/// Pagination metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: usize,
    pub count: usize,
    pub per_page: usize,
    pub current_page: usize,
    pub last_page: usize,
    pub from: usize,
    pub to: usize,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a success response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            meta: None,
        }
    }

    /// Create a success response with message
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.into()),
            meta: None,
        }
    }

    /// Create a success response with metadata
    pub fn success_with_meta(data: T, meta: Meta) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            meta: Some(meta),
        }
    }

    /// Convert to HttpResponse
    pub fn to_http_response(self, status: StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(self)
    }
}

/// API Error Response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub success: bool,
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ApiError {
    /// Create an error response
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
                details: None,
            },
        }
    }

    /// Create an error response with details
    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            success: false,
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
                details: Some(details),
            },
        }
    }

    /// Convert to HttpResponse
    pub fn to_http_response(self, status: StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(self)
    }
}

/// Paginated Response
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub meta: Meta,
}

impl<T: Serialize> PaginatedResponse<T> {
    /// Create a paginated response
    pub fn new(
        data: Vec<T>,
        total: usize,
        per_page: usize,
        current_page: usize,
    ) -> Self {
        let count = data.len();
        let last_page = (total as f64 / per_page as f64).ceil() as usize;
        let from = if count > 0 {
            (current_page - 1) * per_page + 1
        } else {
            0
        };
        let to = from + count - 1;

        Self {
            success: true,
            data,
            meta: Meta {
                pagination: Some(PaginationMeta {
                    total,
                    count,
                    per_page,
                    current_page,
                    last_page,
                    from,
                    to,
                }),
                timestamp: Some(chrono::Utc::now().timestamp()),
            },
        }
    }

    /// Convert to HttpResponse
    pub fn to_http_response(self) -> HttpResponse {
        HttpResponse::Ok().json(self)
    }
}

/// Response helpers
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// OK response (200)
    pub fn ok<T: Serialize>(data: T) -> HttpResponse {
        ApiResponse::success(data).to_http_response(StatusCode::OK)
    }

    /// Created response (201)
    pub fn created<T: Serialize>(data: T) -> HttpResponse {
        ApiResponse::success_with_message(data, "Resource created successfully")
            .to_http_response(StatusCode::CREATED)
    }

    /// No content response (204)
    pub fn no_content() -> HttpResponse {
        HttpResponse::NoContent().finish()
    }

    /// Bad request response (400)
    pub fn bad_request(message: impl Into<String>) -> HttpResponse {
        ApiError::new("bad_request", message).to_http_response(StatusCode::BAD_REQUEST)
    }

    /// Unauthorized response (401)
    pub fn unauthorized(message: impl Into<String>) -> HttpResponse {
        ApiError::new("unauthorized", message).to_http_response(StatusCode::UNAUTHORIZED)
    }

    /// Forbidden response (403)
    pub fn forbidden(message: impl Into<String>) -> HttpResponse {
        ApiError::new("forbidden", message).to_http_response(StatusCode::FORBIDDEN)
    }

    /// Not found response (404)
    pub fn not_found(message: impl Into<String>) -> HttpResponse {
        ApiError::new("not_found", message).to_http_response(StatusCode::NOT_FOUND)
    }

    /// Conflict response (409)
    pub fn conflict(message: impl Into<String>) -> HttpResponse {
        ApiError::new("conflict", message).to_http_response(StatusCode::CONFLICT)
    }

    /// Validation error response (422)
    pub fn validation_error(details: Value) -> HttpResponse {
        ApiError::with_details("validation_error", "Validation failed", details)
            .to_http_response(StatusCode::UNPROCESSABLE_ENTITY)
    }

    /// Internal server error response (500)
    pub fn internal_error(message: impl Into<String>) -> HttpResponse {
        ApiError::new("internal_error", message)
            .to_http_response(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        id: i32,
        name: String,
    }

    #[test]
    fn test_api_response_success() {
        let data = TestData {
            id: 1,
            name: "Test".to_string(),
        };
        let response = ApiResponse::success(data);
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.message.is_none());
    }

    #[test]
    fn test_api_response_with_message() {
        let data = TestData {
            id: 1,
            name: "Test".to_string(),
        };
        let response = ApiResponse::success_with_message(data, "Success!");
        assert!(response.success);
        assert_eq!(response.message, Some("Success!".to_string()));
    }

    #[test]
    fn test_api_error() {
        let error = ApiError::new("test_error", "Test error message");
        assert!(!error.success);
        assert_eq!(error.error.code, "test_error");
        assert_eq!(error.error.message, "Test error message");
    }

    #[test]
    fn test_paginated_response() {
        let data = vec![
            TestData {
                id: 1,
                name: "Test 1".to_string(),
            },
            TestData {
                id: 2,
                name: "Test 2".to_string(),
            },
        ];

        let response = PaginatedResponse::new(data, 10, 2, 1);
        assert!(response.success);
        assert_eq!(response.data.len(), 2);

        let pagination = response.meta.pagination.unwrap();
        assert_eq!(pagination.total, 10);
        assert_eq!(pagination.count, 2);
        assert_eq!(pagination.per_page, 2);
        assert_eq!(pagination.current_page, 1);
        assert_eq!(pagination.last_page, 5);
        assert_eq!(pagination.from, 1);
        assert_eq!(pagination.to, 2);
    }
}
