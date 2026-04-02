//! Database Manager
//!
//! Manages database connections for multi-tenant architecture

use crate::base::EntityConnection;
use crate::database::connection::{ConnectionManager, PgPool};
use crate::utils::AppResult;
use std::sync::Arc;
use uuid::Uuid;

/// Database Manager
///
/// Handles:
/// - Multi-database connections (core, hq, group, tenant)
/// - Entity-to-connection mapping
/// - Cluster schema management (cashier_*, scm_*)
/// - Connection pooling per database
pub struct DatabaseManager {
    /// Connection manager for pool management
    connection_manager: Arc<ConnectionManager>,
}

impl DatabaseManager {
    /// Create new database manager
    pub fn new() -> AppResult<Self> {
        let connection_manager = ConnectionManager::new()?;

        Ok(Self {
            connection_manager: Arc::new(connection_manager),
        })
    }

    /// Create database manager with custom connection manager
    pub fn with_connection_manager(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager: Arc::new(connection_manager),
        }
    }

    /// Get connection pool based on entity connection type
    pub fn get_pool(&self, connection_type: &EntityConnection) -> AppResult<PgPool> {
        match connection_type {
            EntityConnection::Core => {
                self.connection_manager.get_pool("pantheon_core")
            }
            EntityConnection::HQ => {
                self.connection_manager.get_pool("pantheon_hq")
            }
            EntityConnection::Group(group_id) => {
                let db_name = self.resolve_group_database(group_id);
                self.connection_manager.get_pool(&db_name)
            }
            EntityConnection::Tenant(tenant_id) => {
                let db_name = self.resolve_tenant_database(tenant_id);
                self.connection_manager.get_pool(&db_name)
            }
            EntityConnection::Cluster { cluster_type: _, year: _, month: _ } => {
                // Cluster schemas live in the core database
                // The schema name is determined by resolve_cluster_schema
                self.connection_manager.get_pool("pantheon_core")
            }
        }
    }

    /// Get connection pool for core database
    pub fn get_core_pool(&self) -> AppResult<PgPool> {
        self.get_pool(&EntityConnection::Core)
    }

    /// Get connection pool for HQ database
    pub fn get_hq_pool(&self) -> AppResult<PgPool> {
        self.get_pool(&EntityConnection::HQ)
    }

    /// Get connection pool for group database
    pub fn get_group_pool(&self, group_id: &Uuid) -> AppResult<PgPool> {
        self.get_pool(&EntityConnection::Group(*group_id))
    }

    /// Get connection pool for tenant database
    pub fn get_tenant_pool(&self, tenant_id: &Uuid) -> AppResult<PgPool> {
        self.get_pool(&EntityConnection::Tenant(*tenant_id))
    }

    /// Resolve tenant database name from tenant_id
    pub fn resolve_tenant_database(&self, tenant_id: &Uuid) -> String {
        // Format: pantheon_tenant_{uuid without hyphens}
        let tenant_id_str = tenant_id.to_string().replace('-', "");
        format!("pantheon_tenant_{}", tenant_id_str)
    }

    /// Resolve group database name from group_id
    pub fn resolve_group_database(&self, group_id: &Uuid) -> String {
        // Format: pantheon_group_{uuid without hyphens}
        let group_id_str = group_id.to_string().replace('-', "");
        format!("pantheon_group_{}", group_id_str)
    }

    /// Resolve cluster schema name
    ///
    /// # Arguments
    /// * `cluster_type` - Type of cluster (e.g., "cashier", "scm")
    /// * `year` - Year for the cluster
    /// * `month` - Optional month for the cluster
    ///
    /// # Examples
    /// ```ignore
    /// // Yearly cluster
    /// resolve_cluster_schema("cashier", 2024, None) // Returns "cashier_2024"
    ///
    /// // Monthly cluster
    /// resolve_cluster_schema("scm", 2024, Some(3)) // Returns "scm_202403"
    /// ```
    pub fn resolve_cluster_schema(&self, cluster_type: &str, year: i32, month: Option<u8>) -> String {
        if let Some(m) = month {
            format!("{}_{:04}{:02}", cluster_type, year, m)
        } else {
            format!("{}_{:04}", cluster_type, year)
        }
    }

    /// Test connection health for a specific connection type
    pub fn health_check(&self, connection_type: &EntityConnection) -> AppResult<bool> {
        let database_name = match connection_type {
            EntityConnection::Core => "pantheon_core".to_string(),
            EntityConnection::HQ => "pantheon_hq".to_string(),
            EntityConnection::Group(group_id) => self.resolve_group_database(group_id),
            EntityConnection::Tenant(tenant_id) => self.resolve_tenant_database(tenant_id),
            EntityConnection::Cluster { .. } => "pantheon_core".to_string(),
        };

        self.connection_manager.health_check(&database_name)
    }

    /// Get list of all active database connections
    pub fn list_active_databases(&self) -> AppResult<Vec<String>> {
        self.connection_manager.list_databases()
    }

    /// Close connection pool for a specific database
    pub fn close_pool(&self, database_name: &str) -> AppResult<()> {
        self.connection_manager.close_pool(database_name)
    }

    /// Close all connection pools
    pub fn close_all_pools(&self) -> AppResult<()> {
        self.connection_manager.close_all_pools()
    }

    /// Get reference to underlying connection manager
    pub fn connection_manager(&self) -> &ConnectionManager {
        &self.connection_manager
    }
}

impl Default for DatabaseManager {
    fn default() -> Self {
        Self::new().expect("Failed to create DatabaseManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_manager_creation() {
        let result = DatabaseManager::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_tenant_database() {
        let manager = DatabaseManager::new().unwrap();
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let db_name = manager.resolve_tenant_database(&tenant_id);
        assert_eq!(db_name, "pantheon_tenant_550e8400e29b41d4a716446655440000");
    }

    #[test]
    fn test_resolve_group_database() {
        let manager = DatabaseManager::new().unwrap();
        let group_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let db_name = manager.resolve_group_database(&group_id);
        assert_eq!(db_name, "pantheon_group_550e8400e29b41d4a716446655440000");
    }

    #[test]
    fn test_resolve_cluster_schema_yearly() {
        let manager = DatabaseManager::new().unwrap();

        let schema = manager.resolve_cluster_schema("cashier", 2024, None);
        assert_eq!(schema, "cashier_2024");
    }

    #[test]
    fn test_resolve_cluster_schema_monthly() {
        let manager = DatabaseManager::new().unwrap();

        let schema = manager.resolve_cluster_schema("scm", 2024, Some(3));
        assert_eq!(schema, "scm_202403");
    }

    #[test]
    fn test_resolve_cluster_schema_with_padding() {
        let manager = DatabaseManager::new().unwrap();

        let schema = manager.resolve_cluster_schema("cashier", 2024, Some(9));
        assert_eq!(schema, "cashier_202409");
    }
}
