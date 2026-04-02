//! Base Entity Trait
//!
//! Provides base functionality for all entities in the system.
//! Every entity must implement this trait to work with the framework.

use serde::Serialize;
use serde_json::Value;
use std::any::Any;
use uuid::Uuid;

/// Entity connection type
/// Determines which database connection the entity should use
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityConnection {
    /// Core database (pantheon_core)
    Core,
    /// HQ database (pantheon_hq)
    HQ,
    /// Group database (pantheon_group_{id})
    Group(Uuid),
    /// Tenant database (pantheon_tenant_{id})
    Tenant(Uuid),
    /// Cluster schema in core database (cashier_*, scm_*)
    Cluster {
        cluster_type: String,
        year: i32,
        month: Option<u8>,
    },
}

impl EntityConnection {
    /// Get connection name as string
    pub fn as_str(&self) -> &str {
        match self {
            EntityConnection::Core => "core",
            EntityConnection::HQ => "hq",
            EntityConnection::Group(_) => "group",
            EntityConnection::Tenant(_) => "tenant",
            EntityConnection::Cluster { cluster_type, .. } => cluster_type,
        }
    }
}

/// Base Entity Trait
///
/// All entities in the system must implement this trait.
/// Provides methods for:
/// - Getting resource representations (view/show)
/// - Converting to API responses
/// - Getting database connection information
/// - Getting table metadata
pub trait BaseEntity: Send + Sync {
    /// Get the view resource type for this entity (for list views)
    /// Returns a boxed ViewResource that can transform this entity
    fn get_view_resource(&self) -> Box<dyn Any>;

    /// Get the show resource type for this entity (for detail views)
    /// Returns a boxed ShowResource that can transform this entity
    fn get_show_resource(&self) -> Box<dyn Any>;

    /// Transform entity to API response using ViewResource
    /// Used for list/index endpoints
    fn to_view_api(&self) -> Result<Value, Box<dyn std::error::Error>>;

    /// Transform entity to API response using ShowResource
    /// Used for show/detail endpoints
    fn to_show_api(&self) -> Result<Value, Box<dyn std::error::Error>>;

    /// Get the database connection type for this entity
    /// Determines which database/schema the entity belongs to
    fn get_connection(&self) -> EntityConnection {
        // Default to core connection
        EntityConnection::Core
    }

    /// Get the table name for this entity
    fn table_name() -> &'static str
    where
        Self: Sized;

    /// Get the primary key field name
    fn primary_key() -> &'static str
    where
        Self: Sized,
    {
        "id"
    }

    /// Get the entity ID as UUID (if applicable)
    fn get_id(&self) -> Option<Uuid> {
        None
    }

    /// Check if entity is soft-deleted
    fn is_deleted(&self) -> bool {
        false
    }

    /// Get created_at timestamp
    fn created_at(&self) -> Option<chrono::NaiveDateTime> {
        None
    }

    /// Get updated_at timestamp
    fn updated_at(&self) -> Option<chrono::NaiveDateTime> {
        None
    }
}

/// Helper trait for entities that can be serialized
pub trait SerializableEntity: BaseEntity + Serialize {
    /// Convert entity to JSON value
    fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

// Automatically implement SerializableEntity for any type that implements both BaseEntity and Serialize
impl<T> SerializableEntity for T where T: BaseEntity + Serialize {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_connection_as_str() {
        assert_eq!(EntityConnection::Core.as_str(), "core");
        assert_eq!(EntityConnection::Tenant.as_str(), "tenant");
        assert_eq!(
            EntityConnection::Cluster {
                cluster_type: "cashier".to_string()
            }
            .as_str(),
            "cashier"
        );
    }

    #[test]
    fn test_entity_connection_equality() {
        let conn1 = EntityConnection::Core;
        let conn2 = EntityConnection::Core;
        assert_eq!(conn1, conn2);

        let conn3 = EntityConnection::Cluster {
            cluster_type: "cashier".to_string(),
        };
        let conn4 = EntityConnection::Cluster {
            cluster_type: "cashier".to_string(),
        };
        assert_eq!(conn3, conn4);
    }
}
