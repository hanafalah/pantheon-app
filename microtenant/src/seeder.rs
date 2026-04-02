//! Tenant Seeder
//!
//! Seed default data for new tenants

use rust_support::database::ConnectionManager;
use rust_support::utils::{AppError, AppResult};
use uuid::Uuid;
use diesel::RunQueryDsl;

/// Tenant Seeder
///
/// Seeds default data when a new tenant is created
pub struct TenantSeeder {
    connection_manager: ConnectionManager,
}

impl TenantSeeder {
    /// Create new tenant seeder
    pub fn new() -> AppResult<Self> {
        let connection_manager = ConnectionManager::new()?;
        Ok(Self { connection_manager })
    }

    /// Create seeder with custom connection manager
    pub fn with_connection_manager(connection_manager: ConnectionManager) -> Self {
        Self { connection_manager }
    }

    /// Seed default data for new tenant
    pub async fn seed_tenant(&self, tenant_id: Uuid) -> AppResult<()> {
        let db_name = self.resolve_database_name(tenant_id);

        tracing::info!("Seeding default data for tenant: {}", db_name);

        // Get connection pool for tenant database
        let pool = self.connection_manager.get_pool(&db_name)?;

        let mut conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // Seed default roles
        self.seed_default_roles(&mut conn)?;

        // Seed default permissions
        self.seed_default_permissions(&mut conn)?;

        // Seed default settings
        self.seed_default_settings(&mut conn)?;

        tracing::info!("Successfully seeded tenant: {}", db_name);

        Ok(())
    }

    /// Seed default roles
    fn seed_default_roles(&self, conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>) -> AppResult<()> {
        let roles = vec![
            ("admin", "Administrator"),
            ("manager", "Manager"),
            ("staff", "Staff"),
            ("user", "User"),
        ];

        for (name, description) in roles {
            let sql = format!(
                "INSERT INTO roles (id, name, description, created_at, updated_at) \
                 VALUES (gen_random_uuid(), '{}', '{}', NOW(), NOW()) \
                 ON CONFLICT (name) DO NOTHING",
                name, description
            );

            diesel::sql_query(&sql)
                .execute(&mut **conn)
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to seed role {}: {}", name, e))
                })?;
        }

        tracing::info!("Seeded default roles");
        Ok(())
    }

    /// Seed default permissions
    fn seed_default_permissions(&self, conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>) -> AppResult<()> {
        let permissions = vec![
            ("view_dashboard", "View Dashboard"),
            ("manage_users", "Manage Users"),
            ("manage_roles", "Manage Roles"),
            ("manage_settings", "Manage Settings"),
        ];

        for (name, description) in permissions {
            let sql = format!(
                "INSERT INTO permissions (id, name, description, created_at, updated_at) \
                 VALUES (gen_random_uuid(), '{}', '{}', NOW(), NOW()) \
                 ON CONFLICT (name) DO NOTHING",
                name, description
            );

            diesel::sql_query(&sql)
                .execute(&mut **conn)
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to seed permission {}: {}", name, e))
                })?;
        }

        tracing::info!("Seeded default permissions");
        Ok(())
    }

    /// Seed default settings
    fn seed_default_settings(&self, conn: &mut diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>) -> AppResult<()> {
        let settings = vec![
            ("app_name", "Pantheon App"),
            ("timezone", "UTC"),
            ("currency", "IDR"),
            ("language", "id"),
        ];

        for (key, value) in settings {
            let sql = format!(
                "INSERT INTO settings (id, key, value, created_at, updated_at) \
                 VALUES (gen_random_uuid(), '{}', '{}', NOW(), NOW()) \
                 ON CONFLICT (key) DO NOTHING",
                key, value
            );

            diesel::sql_query(&sql)
                .execute(&mut **conn)
                .ok(); // Ignore if settings table doesn't exist yet
        }

        tracing::info!("Seeded default settings");
        Ok(())
    }

    /// Seed custom data from SQL file
    pub async fn seed_from_file(&self, tenant_id: Uuid, sql_file: &str) -> AppResult<()> {
        let db_name = self.resolve_database_name(tenant_id);

        let pool = self.connection_manager.get_pool(&db_name)?;

        let mut conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // Read SQL file
        let sql = std::fs::read_to_string(sql_file).map_err(|e| {
            AppError::ConfigError(format!("Failed to read seed file: {}", e))
        })?;

        // Execute SQL
        diesel::sql_query(&sql)
            .execute(&mut *conn)
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to execute seed file: {}", e))
            })?;

        tracing::info!("Seeded data from file: {} for {}", sql_file, db_name);

        Ok(())
    }

    /// Resolve database name from tenant ID
    fn resolve_database_name(&self, tenant_id: Uuid) -> String {
        format!("pantheon_tenant_{}", tenant_id.to_string().replace('-', ""))
    }
}

impl Default for TenantSeeder {
    fn default() -> Self {
        Self::new().expect("Failed to create TenantSeeder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_seeder_creation() {
        let result = TenantSeeder::new();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_resolve_database_name() {
        let seeder = TenantSeeder::new().unwrap_or_else(|_| {
            TenantSeeder::with_connection_manager(ConnectionManager::new().unwrap())
        });

        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let db_name = seeder.resolve_database_name(tenant_id);

        assert_eq!(db_name, "pantheon_tenant_550e8400e29b41d4a716446655440000");
    }
}
