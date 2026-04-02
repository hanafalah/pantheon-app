//! Connection Manager
//!
//! Manages database connection pools with r2d2

use crate::utils::{AppError, AppResult};
use diesel::r2d2::{ConnectionManager as DieselConnectionManager, Pool};
use diesel::PgConnection;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Type alias for PostgreSQL connection pool
pub type PgPool = Pool<DieselConnectionManager<PgConnection>>;

/// Database Connection Configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Minimum number of connections in pool
    pub min_connections: u32,
    /// Maximum number of connections in pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl DatabaseConfig {
    /// Create new database config
    pub fn new(url: String) -> Self {
        Self {
            url,
            min_connections: 1,
            max_connections: 10,
            connection_timeout: 30,
        }
    }

    /// Create config with custom pool settings
    pub fn with_pool_settings(
        url: String,
        min_connections: u32,
        max_connections: u32,
        connection_timeout: u64,
    ) -> Self {
        Self {
            url,
            min_connections,
            max_connections,
            connection_timeout,
        }
    }
}

/// Connection Manager
///
/// Manages connection pools for multiple databases
pub struct ConnectionManager {
    /// Connection pools by database name
    pools: Arc<RwLock<HashMap<String, PgPool>>>,
    /// Default database config
    default_config: DatabaseConfig,
}

impl ConnectionManager {
    /// Create new connection manager
    pub fn new() -> AppResult<Self> {
        let default_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://pantheon:pantheon@localhost/pantheon_core".to_string());

        Ok(Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            default_config: DatabaseConfig::new(default_url),
        })
    }

    /// Create connection manager with custom default config
    pub fn with_default_config(config: DatabaseConfig) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            default_config: config,
        }
    }

    /// Get or create connection pool for database
    pub fn get_pool(&self, database_name: &str) -> AppResult<PgPool> {
        // Check if pool already exists
        {
            let pools = self.pools.read().map_err(|e| {
                AppError::DatabaseError(format!("Failed to read pools: {}", e))
            })?;

            if let Some(pool) = pools.get(database_name) {
                return Ok(pool.clone());
            }
        }

        // Create new pool
        let database_url = self.build_database_url(database_name);
        let config = DatabaseConfig::new(database_url);
        let pool = self.create_pool(&config)?;

        // Store the pool
        {
            let mut pools = self.pools.write().map_err(|e| {
                AppError::DatabaseError(format!("Failed to write pools: {}", e))
            })?;

            pools.insert(database_name.to_string(), pool.clone());
        }

        Ok(pool)
    }

    /// Get pool for a specific database URL
    pub fn get_pool_for_url(&self, database_url: &str, database_name: &str) -> AppResult<PgPool> {
        // Check if pool already exists
        {
            let pools = self.pools.read().map_err(|e| {
                AppError::DatabaseError(format!("Failed to read pools: {}", e))
            })?;

            if let Some(pool) = pools.get(database_name) {
                return Ok(pool.clone());
            }
        }

        // Create new pool with provided URL
        let config = DatabaseConfig::new(database_url.to_string());
        let pool = self.create_pool(&config)?;

        // Store the pool
        {
            let mut pools = self.pools.write().map_err(|e| {
                AppError::DatabaseError(format!("Failed to write pools: {}", e))
            })?;

            pools.insert(database_name.to_string(), pool.clone());
        }

        Ok(pool)
    }

    /// Create a new connection pool
    fn create_pool(&self, config: &DatabaseConfig) -> AppResult<PgPool> {
        let manager = DieselConnectionManager::<PgConnection>::new(&config.url);

        let pool = Pool::builder()
            .min_idle(Some(config.min_connections))
            .max_size(config.max_connections)
            .connection_timeout(std::time::Duration::from_secs(config.connection_timeout))
            .build(manager)
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to create connection pool: {}", e))
            })?;

        Ok(pool)
    }

    /// Build database URL from database name
    fn build_database_url(&self, database_name: &str) -> String {
        // Parse the default URL and replace the database name
        let base_url = &self.default_config.url;

        // Simple string replacement (assuming format: postgresql://user:pass@host/db)
        if let Some(pos) = base_url.rfind('/') {
            format!("{}/{}", &base_url[..pos], database_name)
        } else {
            base_url.clone()
        }
    }

    /// Test connection health for a database
    pub fn health_check(&self, database_name: &str) -> AppResult<bool> {
        let pool = self.get_pool(database_name)?;

        // Try to get a connection from the pool
        match pool.get() {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("Health check failed for {}: {}", database_name, e);
                Ok(false)
            }
        }
    }

    /// Get all database names with active pools
    pub fn list_databases(&self) -> AppResult<Vec<String>> {
        let pools = self.pools.read().map_err(|e| {
            AppError::DatabaseError(format!("Failed to read pools: {}", e))
        })?;

        Ok(pools.keys().cloned().collect())
    }

    /// Close pool for a specific database
    pub fn close_pool(&self, database_name: &str) -> AppResult<()> {
        let mut pools = self.pools.write().map_err(|e| {
            AppError::DatabaseError(format!("Failed to write pools: {}", e))
        })?;

        pools.remove(database_name);
        Ok(())
    }

    /// Close all pools
    pub fn close_all_pools(&self) -> AppResult<()> {
        let mut pools = self.pools.write().map_err(|e| {
            AppError::DatabaseError(format!("Failed to write pools: {}", e))
        })?;

        pools.clear();
        Ok(())
    }

    /// Get pool statistics
    pub fn get_pool_stats(&self, database_name: &str) -> AppResult<PoolStats> {
        let pool = self.get_pool(database_name)?;
        let state = pool.state();

        Ok(PoolStats {
            connections: state.connections,
            idle_connections: state.idle_connections,
        })
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ConnectionManager")
    }
}

/// Pool Statistics
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    /// Total number of connections
    pub connections: u32,
    /// Number of idle connections
    pub idle_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config() {
        let config = DatabaseConfig::new("postgresql://localhost/test".to_string());
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connection_timeout, 30);
    }

    #[test]
    fn test_database_config_custom() {
        let config = DatabaseConfig::with_pool_settings(
            "postgresql://localhost/test".to_string(),
            2,
            20,
            60,
        );
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.connection_timeout, 60);
    }

    #[test]
    fn test_connection_manager_creation() {
        let result = ConnectionManager::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_database_url() {
        let manager = ConnectionManager::with_default_config(DatabaseConfig::new(
            "postgresql://user:pass@localhost:5432/pantheon_core".to_string(),
        ));

        let url = manager.build_database_url("pantheon_tenant_123");
        assert_eq!(url, "postgresql://user:pass@localhost:5432/pantheon_tenant_123");
    }
}
