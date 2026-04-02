//! Base Service Trait
//!
//! Provides business logic layer with dependency injection support

use std::sync::Arc;
use uuid::Uuid;

/// Base Service Trait
///
/// Services contain business logic and are injected into controllers.
/// Should be stateless and thread-safe.
#[async_trait::async_trait]
pub trait BaseService: Send + Sync {
    /// Service name for dependency injection
    fn service_name(&self) -> &'static str;

    /// Initialize service
    /// Called during application startup
    async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Cleanup service
    /// Called during application shutdown
    async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Service container for dependency injection
pub struct ServiceContainer {
    services: std::collections::HashMap<String, Arc<dyn BaseService>>,
}

impl ServiceContainer {
    /// Create new service container
    pub fn new() -> Self {
        Self {
            services: std::collections::HashMap::new(),
        }
    }

    /// Register a service
    pub fn register<T: BaseService + 'static>(&mut self, service: T) {
        let name = service.service_name().to_string();
        self.services.insert(name, Arc::new(service));
    }

    /// Get a service by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn BaseService>> {
        self.services.get(name).cloned()
    }

    /// Check if service exists
    pub fn has(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    /// Get all registered service names
    pub fn services(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Pagination parameters
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub page: usize,
    pub per_page: usize,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

impl PaginationParams {
    pub fn new(page: usize, per_page: usize) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.min(100).max(1),
        }
    }

    pub fn offset(&self) -> usize {
        (self.page - 1) * self.per_page
    }

    pub fn limit(&self) -> usize {
        self.per_page
    }
}

/// Sort parameters
#[derive(Debug, Clone)]
pub struct SortParams {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "desc" | "descending" => Self::Desc,
            _ => Self::Asc,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Filter parameters
#[derive(Debug, Clone)]
pub struct FilterParams {
    pub filters: std::collections::HashMap<String, FilterValue>,
}

#[derive(Debug, Clone)]
pub enum FilterValue {
    String(String),
    Number(i64),
    Boolean(bool),
    Uuid(Uuid),
    Array(Vec<String>),
}

impl FilterParams {
    pub fn new() -> Self {
        Self {
            filters: std::collections::HashMap::new(),
        }
    }

    pub fn add(&mut self, key: String, value: FilterValue) {
        self.filters.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&FilterValue> {
        self.filters.get(key)
    }

    pub fn has(&self, key: &str) -> bool {
        self.filters.contains_key(key)
    }
}

impl Default for FilterParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestService;

    #[async_trait::async_trait]
    impl BaseService for TestService {
        fn service_name(&self) -> &'static str {
            "test_service"
        }
    }

    #[test]
    fn test_service_container() {
        let mut container = ServiceContainer::new();
        container.register(TestService);

        assert!(container.has("test_service"));
        assert!(!container.has("unknown_service"));

        let service = container.get("test_service");
        assert!(service.is_some());
        assert_eq!(service.unwrap().service_name(), "test_service");
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new(3, 10);
        assert_eq!(params.page, 3);
        assert_eq!(params.per_page, 10);
        assert_eq!(params.offset(), 20);
        assert_eq!(params.limit(), 10);
    }

    #[test]
    fn test_pagination_params_limits() {
        // Test max per_page
        let params = PaginationParams::new(1, 200);
        assert_eq!(params.per_page, 100);

        // Test min page
        let params = PaginationParams::new(0, 10);
        assert_eq!(params.page, 1);
    }

    #[test]
    fn test_sort_direction() {
        assert_eq!(SortDirection::from_str("desc"), SortDirection::Desc);
        assert_eq!(SortDirection::from_str("DESC"), SortDirection::Desc);
        assert_eq!(SortDirection::from_str("asc"), SortDirection::Asc);
        assert_eq!(SortDirection::from_str("ASC"), SortDirection::Asc);

        assert_eq!(SortDirection::Asc.as_str(), "ASC");
        assert_eq!(SortDirection::Desc.as_str(), "DESC");
    }

    #[test]
    fn test_filter_params() {
        let mut params = FilterParams::new();
        params.add("status".to_string(), FilterValue::String("active".to_string()));
        params.add("count".to_string(), FilterValue::Number(10));

        assert!(params.has("status"));
        assert!(params.has("count"));
        assert!(!params.has("unknown"));

        match params.get("status") {
            Some(FilterValue::String(s)) => assert_eq!(s, "active"),
            _ => panic!("Expected string filter value"),
        }
    }
}
