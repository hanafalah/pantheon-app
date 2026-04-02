//! Base Controller Trait
//!
//! Provides standard CRUD operations for controllers

use actix_web::{HttpResponse, Result};
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// Base Controller Trait
///
/// Provides standard CRUD endpoints:
/// - index: GET /resource - List all resources
/// - show: GET /resource/{id} - Show single resource
/// - store: POST /resource - Create new resource
/// - update: PUT/PATCH /resource/{id} - Update resource
/// - destroy: DELETE /resource/{id} - Delete resource
#[async_trait::async_trait]
pub trait BaseController: Send + Sync {
    /// List resources (GET /resource)
    ///
    /// Supports pagination, filtering, and sorting
    async fn index(&self, _query: IndexQuery) -> Result<HttpResponse> {
        // Default implementation returns not implemented
        Ok(HttpResponse::NotImplemented().json(serde_json::json!({
            "error": "index not implemented"
        })))
    }

    /// Show single resource (GET /resource/{id})
    async fn show(&self, id: Uuid) -> Result<HttpResponse> {
        // Default implementation returns not implemented
        let _ = id;
        Ok(HttpResponse::NotImplemented().json(serde_json::json!({
            "error": "show not implemented"
        })))
    }

    /// Create new resource (POST /resource)
    async fn store<T: DeserializeOwned + Send>(&self, data: T) -> Result<HttpResponse> {
        // Default implementation returns not implemented
        let _ = data;
        Ok(HttpResponse::NotImplemented().json(serde_json::json!({
            "error": "store not implemented"
        })))
    }

    /// Update resource (PUT/PATCH /resource/{id})
    async fn update<T: DeserializeOwned + Send>(&self, id: Uuid, data: T) -> Result<HttpResponse> {
        // Default implementation returns not implemented
        let _ = (id, data);
        Ok(HttpResponse::NotImplemented().json(serde_json::json!({
            "error": "update not implemented"
        })))
    }

    /// Delete resource (DELETE /resource/{id})
    async fn destroy(&self, id: Uuid) -> Result<HttpResponse> {
        // Default implementation returns not implemented
        let _ = id;
        Ok(HttpResponse::NotImplemented().json(serde_json::json!({
            "error": "destroy not implemented"
        })))
    }
}

/// Query parameters for index endpoint
#[derive(Debug, Clone)]
pub struct IndexQuery {
    /// Page number (1-indexed)
    pub page: Option<usize>,
    /// Items per page
    pub per_page: Option<usize>,
    /// Sort field
    pub sort: Option<String>,
    /// Sort direction (asc/desc)
    pub order: Option<String>,
    /// Search query
    pub search: Option<String>,
    /// Additional filters as key-value pairs
    pub filters: std::collections::HashMap<String, String>,
}

impl Default for IndexQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            sort: None,
            order: Some("asc".to_string()),
            search: None,
            filters: std::collections::HashMap::new(),
        }
    }
}

impl IndexQuery {
    /// Get page number (default: 1)
    pub fn get_page(&self) -> usize {
        self.page.unwrap_or(1).max(1)
    }

    /// Get per_page (default: 20, max: 100)
    pub fn get_per_page(&self) -> usize {
        self.per_page.unwrap_or(20).min(100).max(1)
    }

    /// Get offset for SQL queries
    pub fn get_offset(&self) -> usize {
        (self.get_page() - 1) * self.get_per_page()
    }

    /// Get limit for SQL queries
    pub fn get_limit(&self) -> usize {
        self.get_per_page()
    }

    /// Check if sorting is descending
    pub fn is_desc(&self) -> bool {
        matches!(
            self.order.as_deref(),
            Some("desc") | Some("DESC") | Some("descending")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_query_defaults() {
        let query = IndexQuery::default();
        assert_eq!(query.get_page(), 1);
        assert_eq!(query.get_per_page(), 20);
        assert_eq!(query.get_offset(), 0);
        assert_eq!(query.get_limit(), 20);
        assert!(!query.is_desc());
    }

    #[test]
    fn test_index_query_pagination() {
        let query = IndexQuery {
            page: Some(3),
            per_page: Some(10),
            ..Default::default()
        };

        assert_eq!(query.get_page(), 3);
        assert_eq!(query.get_per_page(), 10);
        assert_eq!(query.get_offset(), 20); // (3-1) * 10
        assert_eq!(query.get_limit(), 10);
    }

    #[test]
    fn test_index_query_max_per_page() {
        let query = IndexQuery {
            per_page: Some(200), // Should be capped at 100
            ..Default::default()
        };

        assert_eq!(query.get_per_page(), 100);
    }

    #[test]
    fn test_index_query_is_desc() {
        let query_asc = IndexQuery {
            order: Some("asc".to_string()),
            ..Default::default()
        };
        assert!(!query_asc.is_desc());

        let query_desc = IndexQuery {
            order: Some("desc".to_string()),
            ..Default::default()
        };
        assert!(query_desc.is_desc());
    }
}
