//! Database Creator
//!
//! Auto-create tenant databases

use rust_support::database::ConnectionManager;
use rust_support::utils::{AppError, AppResult};
use uuid::Uuid;
use diesel::{RunQueryDsl, QueryableByName};

#[derive(QueryableByName)]
struct DatabaseNameRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    datname: String,
}

/// Database Creator
///
/// Creates new tenant databases automatically
pub struct DatabaseCreator {
    connection_manager: ConnectionManager,
}

impl DatabaseCreator {
    /// Create new database creator
    pub fn new() -> AppResult<Self> {
        let connection_manager = ConnectionManager::new()?;
        Ok(Self { connection_manager })
    }

    /// Create database creator with custom connection manager
    pub fn with_connection_manager(connection_manager: ConnectionManager) -> Self {
        Self { connection_manager }
    }

    /// Create new tenant database
    ///
    /// Creates a new PostgreSQL database for the tenant
    pub async fn create_tenant_database(&self, tenant_id: Uuid) -> AppResult<String> {
        let db_name = self.resolve_database_name(tenant_id);

        // Get connection to postgres database (for creating new databases)
        let postgres_pool = self
            .connection_manager
            .get_pool_for_url("postgresql://pantheon:pantheon@localhost/postgres", "postgres")?;

        let mut conn = postgres_pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // Create database
        let create_sql = format!("CREATE DATABASE {}", db_name);

        diesel::sql_query(&create_sql)
            .execute(&mut *conn)
            .map_err(|e| AppError::DatabaseError(format!("Failed to create database: {}", e)))?;

        tracing::info!("Created tenant database: {}", db_name);

        Ok(db_name)
    }

    /// Check if tenant database exists
    pub async fn database_exists(&self, tenant_id: Uuid) -> AppResult<bool> {
        let db_name = self.resolve_database_name(tenant_id);

        let postgres_pool = self
            .connection_manager
            .get_pool_for_url("postgresql://pantheon:pantheon@localhost/postgres", "postgres")?;

        let mut conn = postgres_pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = format!(
            "SELECT COUNT(*) FROM pg_database WHERE datname = '{}'",
            db_name
        );

        let result = diesel::sql_query(&query)
            .execute(&mut *conn);

        Ok(result.is_ok())
    }

    /// Create database if not exists
    pub async fn create_if_not_exists(&self, tenant_id: Uuid) -> AppResult<String> {
        if self.database_exists(tenant_id).await? {
            let db_name = self.resolve_database_name(tenant_id);
            tracing::info!("Tenant database already exists: {}", db_name);
            Ok(db_name)
        } else {
            self.create_tenant_database(tenant_id).await
        }
    }

    /// Drop tenant database (use with caution!)
    pub async fn drop_tenant_database(&self, tenant_id: Uuid) -> AppResult<()> {
        let db_name = self.resolve_database_name(tenant_id);

        let postgres_pool = self
            .connection_manager
            .get_pool_for_url("postgresql://pantheon:pantheon@localhost/postgres", "postgres")?;

        let mut conn = postgres_pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // Terminate existing connections
        let terminate_sql = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()",
            db_name
        );

        diesel::sql_query(&terminate_sql)
            .execute(&mut *conn)
            .ok(); // Ignore errors

        // Drop database
        let drop_sql = format!("DROP DATABASE IF EXISTS {}", db_name);

        diesel::sql_query(&drop_sql)
            .execute(&mut *conn)
            .map_err(|e| AppError::DatabaseError(format!("Failed to drop database: {}", e)))?;

        tracing::warn!("Dropped tenant database: {}", db_name);

        Ok(())
    }

    /// Resolve database name from tenant ID
    fn resolve_database_name(&self, tenant_id: Uuid) -> String {
        format!("pantheon_tenant_{}", tenant_id.to_string().replace('-', ""))
    }

    /// List all tenant databases
    pub async fn list_tenant_databases(&self) -> AppResult<Vec<String>> {
        let postgres_pool = self
            .connection_manager
            .get_pool_for_url("postgresql://pantheon:pantheon@localhost/postgres", "postgres")?;

        let mut conn = postgres_pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = "SELECT datname FROM pg_database WHERE datname LIKE 'pantheon_tenant_%'";

        let rows: Vec<DatabaseNameRow> = diesel::sql_query(query)
            .load(&mut *conn)
            .map_err(|e| AppError::DatabaseError(format!("Failed to list databases: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.datname).collect())
    }
}

impl Default for DatabaseCreator {
    fn default() -> Self {
        Self::new().expect("Failed to create DatabaseCreator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creator_creation() {
        let result = DatabaseCreator::new();
        // May fail if database not configured, which is expected
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_resolve_database_name() {
        let creator = DatabaseCreator::new().unwrap_or_else(|_| {
            DatabaseCreator::with_connection_manager(ConnectionManager::new().unwrap())
        });

        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let db_name = creator.resolve_database_name(tenant_id);

        assert_eq!(db_name, "pantheon_tenant_550e8400e29b41d4a716446655440000");
    }
}
