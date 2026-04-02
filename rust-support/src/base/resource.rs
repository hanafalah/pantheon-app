//! Base Resource Trait
//!
//! Resources are used to transform entities into API responses.
//! Two main types:
//! - ViewResource: For list/index views (minimal data)
//! - ShowResource: For detail views (complete data, extends ViewResource)

use serde::Serialize;
use serde_json::Value;
use std::any::Any;

/// Base Resource Trait
///
/// All resources must implement this trait to transform entities into API responses
pub trait BaseResource: Send + Sync {
    /// Transform the given entity into a JSON value
    fn transform(&self, entity: &dyn Any) -> Result<Value, Box<dyn std::error::Error>>;

    /// Get resource name
    fn resource_name(&self) -> &'static str;

    /// Check if this is a collection resource
    fn is_collection(&self) -> bool {
        false
    }
}

/// ViewResource - For list/index API responses
///
/// Contains minimal fields needed for list views.
/// Should be lightweight and fast to serialize.
///
/// Example:
/// ```ignore
/// #[derive(Serialize)]
/// pub struct UserViewResource {
///     pub id: Uuid,
///     pub email: String,
///     pub first_name: String,
///     pub last_name: String,
///     pub is_active: bool,
/// }
/// ```
pub trait ViewResource: BaseResource + Serialize {
    /// Convert to JSON value
    fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

/// ShowResource - For detail/show API responses
///
/// Contains complete entity data including relationships.
/// Extends ViewResource with additional fields.
///
/// Example:
/// ```ignore
/// #[derive(Serialize)]
/// pub struct UserShowResource {
///     // Include all ViewResource fields
///     #[serde(flatten)]
///     pub view: UserViewResource,
///
///     // Additional fields for detail view
///     pub phone: Option<String>,
///     pub created_at: NaiveDateTime,
///     pub updated_at: NaiveDateTime,
///     pub roles: Vec<RoleResource>,
///     pub permissions: Vec<PermissionResource>,
/// }
/// ```
pub trait ShowResource: BaseResource + Serialize {
    /// Convert to JSON value
    fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

/// Resource Collection
///
/// Wraps a collection of resources with metadata
#[derive(Debug, Serialize)]
pub struct ResourceCollection<T: Serialize> {
    pub data: Vec<T>,
    pub meta: Option<CollectionMeta>,
}

/// Collection metadata
#[derive(Debug, Serialize)]
pub struct CollectionMeta {
    pub total: usize,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_page: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_page: Option<usize>,
}

impl<T: Serialize> ResourceCollection<T> {
    /// Create a new resource collection
    pub fn new(data: Vec<T>) -> Self {
        let count = data.len();
        Self {
            data,
            meta: Some(CollectionMeta {
                total: count,
                count,
                per_page: None,
                current_page: None,
                last_page: None,
            }),
        }
    }

    /// Create a paginated resource collection
    pub fn paginated(
        data: Vec<T>,
        total: usize,
        per_page: usize,
        current_page: usize,
    ) -> Self {
        let count = data.len();
        let last_page = (total as f64 / per_page as f64).ceil() as usize;

        Self {
            data,
            meta: Some(CollectionMeta {
                total,
                count,
                per_page: Some(per_page),
                current_page: Some(current_page),
                last_page: Some(last_page),
            }),
        }
    }

    /// Create a collection without metadata
    pub fn without_meta(data: Vec<T>) -> Self {
        Self { data, meta: None }
    }
}

/// Helper macro to implement ViewResource for a struct
#[macro_export]
macro_rules! impl_view_resource {
    ($struct_name:ident, $resource_name:expr) => {
        impl BaseResource for $struct_name {
            fn transform(&self, _entity: &dyn Any) -> Result<Value, Box<dyn std::error::Error>> {
                Ok(serde_json::to_value(self)?)
            }

            fn resource_name(&self) -> &'static str {
                $resource_name
            }
        }

        impl ViewResource for $struct_name {}
    };
}

/// Helper macro to implement ShowResource for a struct
#[macro_export]
macro_rules! impl_show_resource {
    ($struct_name:ident, $resource_name:expr, $view_type:ty) => {
        impl BaseResource for $struct_name {
            fn transform(&self, _entity: &dyn Any) -> Result<Value, Box<dyn std::error::Error>> {
                Ok(serde_json::to_value(self)?)
            }

            fn resource_name(&self) -> &'static str {
                $resource_name
            }
        }

        impl ShowResource for $struct_name {
            fn get_view_resource(&self) -> Box<dyn ViewResource> {
                // Implementation would extract view fields
                unimplemented!("Implement get_view_resource for {}", stringify!($struct_name))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestResource {
        id: i32,
        name: String,
    }

    #[test]
    fn test_resource_collection_new() {
        let resources = vec![
            TestResource {
                id: 1,
                name: "Test 1".to_string(),
            },
            TestResource {
                id: 2,
                name: "Test 2".to_string(),
            },
        ];

        let collection = ResourceCollection::new(resources);
        assert_eq!(collection.data.len(), 2);
        assert!(collection.meta.is_some());
        assert_eq!(collection.meta.unwrap().count, 2);
    }

    #[test]
    fn test_resource_collection_paginated() {
        let resources = vec![
            TestResource {
                id: 1,
                name: "Test 1".to_string(),
            },
            TestResource {
                id: 2,
                name: "Test 2".to_string(),
            },
        ];

        let collection = ResourceCollection::paginated(resources, 10, 2, 1);
        assert_eq!(collection.data.len(), 2);

        let meta = collection.meta.unwrap();
        assert_eq!(meta.total, 10);
        assert_eq!(meta.count, 2);
        assert_eq!(meta.per_page, Some(2));
        assert_eq!(meta.current_page, Some(1));
        assert_eq!(meta.last_page, Some(5));
    }
}
