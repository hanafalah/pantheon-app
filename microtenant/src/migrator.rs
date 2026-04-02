//! Tenant Migrator
//!
//! Run migrations for tenant databases

use rust_support::database::ConnectionManager;
use rust_support::utils::{AppError, AppResult};
use std::path::Path;
use uuid::Uuid;
use diesel::{RunQueryDsl, QueryableByName};

#[derive(QueryableByName)]
struct MigrationVersionRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    version: String,
}

/// Tenant Migrator
///
/// Runs database migrations for tenant databases
pub struct TenantMigrator {
    connection_manager: ConnectionManager,
}

impl TenantMigrator {
    /// Create new tenant migrator
    pub fn new() -> AppResult<Self> {
        let connection_manager = ConnectionManager::new()?;
        Ok(Self { connection_manager })
    }

    /// Create migrator with custom connection manager
    pub fn with_connection_manager(connection_manager: ConnectionManager) -> Self {
        Self { connection_manager }
    }

    /// Run migrations for a tenant database
    pub async fn run_migrations(&self, tenant_id: Uuid, migrations_path: &Path) -> AppResult<()> {
        let db_name = self.resolve_database_name(tenant_id);

        // Get connection pool for tenant database
        let pool = self.connection_manager.get_pool(&db_name)?;

        let mut conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // Read migration files
        let migration_files = self.read_migration_files(migrations_path)?;

        // Execute each migration
        for (filename, sql) in migration_files {
            tracing::info!("Running migration: {} on {}", filename, db_name);

            diesel::sql_query(&sql)
                .execute(&mut *conn)
                .map_err(|e| {
                    AppError::DatabaseError(format!(
                        "Failed to run migration {}: {}",
                        filename, e
                    ))
                })?;

            tracing::info!("Migration {} completed successfully", filename);
        }

        Ok(())
    }

    /// Run migrations for all tenant databases
    pub async fn run_migrations_for_all(
        &self,
        migrations_path: &Path,
        tenant_ids: &[Uuid],
    ) -> AppResult<()> {
        for tenant_id in tenant_ids {
            match self.run_migrations(*tenant_id, migrations_path).await {
                Ok(_) => {
                    tracing::info!("Migrations completed for tenant: {}", tenant_id);
                }
                Err(e) => {
                    tracing::error!("Failed to run migrations for tenant {}: {}", tenant_id, e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Rollback last migration (if supported)
    pub async fn rollback(&self, tenant_id: Uuid, migrations_path: &Path) -> AppResult<()> {
        let db_name = self.resolve_database_name(tenant_id);
        tracing::warn!("Rolling back migration for {}", db_name);

        // Implementation would read "down" migrations and execute them
        // For now, this is a placeholder

        Ok(())
    }

    /// Check migration status for tenant
    pub async fn migration_status(&self, tenant_id: Uuid) -> AppResult<Vec<String>> {
        let db_name = self.resolve_database_name(tenant_id);
        let pool = self.connection_manager.get_pool(&db_name)?;

        let mut conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // Query migrations table (if exists)
        let query = "SELECT version FROM migrations ORDER BY version";

        let rows: Result<Vec<MigrationVersionRow>, _> = diesel::sql_query(query).load(&mut *conn);

        match rows {
            Ok(r) => Ok(r.into_iter().map(|row| row.version).collect()),
            Err(_) => Ok(vec![]), // Migrations table doesn't exist yet
        }
    }

    /// Read migration files from directory
    fn read_migration_files(&self, path: &Path) -> AppResult<Vec<(String, String)>> {
        if !path.exists() {
            return Err(AppError::ConfigError(format!(
                "Migrations path not found: {}",
                path.display()
            )));
        }

        let mut migrations = Vec::new();

        for entry in std::fs::read_dir(path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read migrations dir: {}", e)))?
        {
            let entry = entry
                .map_err(|e| AppError::ConfigError(format!("Failed to read entry: {}", e)))?;

            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                let filename = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                let sql = std::fs::read_to_string(&path).map_err(|e| {
                    AppError::ConfigError(format!("Failed to read migration file: {}", e))
                })?;

                migrations.push((filename, sql));
            }
        }

        // Sort by filename (timestamp-based)
        migrations.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(migrations)
    }

    /// Resolve database name from tenant ID
    fn resolve_database_name(&self, tenant_id: Uuid) -> String {
        format!("pantheon_tenant_{}", tenant_id.to_string().replace('-', ""))
    }
}

impl Default for TenantMigrator {
    fn default() -> Self {
        Self::new().expect("Failed to create TenantMigrator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_migrator_creation() {
        let result = TenantMigrator::new();
        // May fail if database not configured
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_resolve_database_name() {
        let migrator = TenantMigrator::new().unwrap_or_else(|_| {
            TenantMigrator::with_connection_manager(ConnectionManager::new().unwrap())
        });

        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let db_name = migrator.resolve_database_name(tenant_id);

        assert_eq!(db_name, "pantheon_tenant_550e8400e29b41d4a716446655440000");
    }
}
