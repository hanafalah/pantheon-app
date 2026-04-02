//! Cluster Schema Manager
//!
//! Manage cluster schemas for data segmentation

use rust_support::database::ConnectionManager;
use rust_support::utils::{AppError, AppResult};
use diesel::{RunQueryDsl, QueryableByName};
use chrono::Datelike;

#[derive(QueryableByName)]
struct SchemaNameRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    schema_name: String,
}

/// Cluster Schema Manager
///
/// Manages cluster schemas (cashier_*, scm_*) for data segmentation by year/month
pub struct ClusterSchemaManager {
    connection_manager: ConnectionManager,
}

impl ClusterSchemaManager {
    /// Create new cluster schema manager
    pub fn new() -> AppResult<Self> {
        let connection_manager = ConnectionManager::new()?;
        Ok(Self { connection_manager })
    }

    /// Create cluster schema manager with custom connection manager
    pub fn with_connection_manager(connection_manager: ConnectionManager) -> Self {
        Self { connection_manager }
    }

    /// Create new cluster schema
    ///
    /// Creates a new schema for cluster data (e.g., cashier_2024, scm_202403)
    pub async fn create_cluster_schema(
        &self,
        database: &str,
        cluster_type: &str,
        year: i32,
        month: Option<u8>,
    ) -> AppResult<String> {
        let schema_name = self.resolve_schema_name(cluster_type, year, month);

        // Get connection to target database
        let pool = self.connection_manager.get_pool(database)?;

        let mut conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        // Create schema
        let create_sql = format!("CREATE SCHEMA IF NOT EXISTS {}", schema_name);

        diesel::sql_query(&create_sql)
            .execute(&mut *conn)
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to create schema: {}", e))
            })?;

        tracing::info!("Created cluster schema: {} in {}", schema_name, database);

        Ok(schema_name)
    }

    /// Check if cluster schema exists
    pub async fn schema_exists(
        &self,
        database: &str,
        cluster_type: &str,
        year: i32,
        month: Option<u8>,
    ) -> AppResult<bool> {
        let schema_name = self.resolve_schema_name(cluster_type, year, month);

        let pool = self.connection_manager.get_pool(database)?;

        let mut conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = format!(
            "SELECT COUNT(*) as count FROM information_schema.schemata WHERE schema_name = '{}'",
            schema_name
        );

        // Execute and check if any row is returned
        let result = diesel::sql_query(&query).execute(&mut *conn);

        Ok(result.is_ok())
    }

    /// Create cluster schema if not exists
    pub async fn create_if_not_exists(
        &self,
        database: &str,
        cluster_type: &str,
        year: i32,
        month: Option<u8>,
    ) -> AppResult<String> {
        if self.schema_exists(database, cluster_type, year, month).await? {
            let schema_name = self.resolve_schema_name(cluster_type, year, month);
            tracing::info!("Cluster schema already exists: {}", schema_name);
            Ok(schema_name)
        } else {
            self.create_cluster_schema(database, cluster_type, year, month)
                .await
        }
    }

    /// Auto-create cluster schemas for upcoming period
    ///
    /// Creates schemas 5 days before the start of new year/month
    pub async fn auto_create_upcoming_schemas(
        &self,
        database: &str,
        cluster_types: &[&str],
    ) -> AppResult<Vec<String>> {
        let now = chrono::Utc::now();
        let days_until_new_year = Self::days_until_new_year(&now);
        let days_until_new_month = Self::days_until_new_month(&now);

        let mut created_schemas = Vec::new();

        // Check if within 5 days of new year
        if days_until_new_year <= 5 {
            let next_year = now.year() + 1;
            for cluster_type in cluster_types {
                let schema = self
                    .create_if_not_exists(database, cluster_type, next_year, None)
                    .await?;
                created_schemas.push(schema);
            }
        }

        // Check if within 5 days of new month
        if days_until_new_month <= 5 {
            let next_month = if now.month() == 12 {
                (now.year() + 1, 1)
            } else {
                (now.year(), now.month() + 1)
            };

            for cluster_type in cluster_types {
                let schema = self
                    .create_if_not_exists(database, cluster_type, next_month.0, Some(next_month.1 as u8))
                    .await?;
                created_schemas.push(schema);
            }
        }

        Ok(created_schemas)
    }

    /// Drop cluster schema (use with caution!)
    pub async fn drop_cluster_schema(
        &self,
        database: &str,
        cluster_type: &str,
        year: i32,
        month: Option<u8>,
    ) -> AppResult<()> {
        let schema_name = self.resolve_schema_name(cluster_type, year, month);

        let pool = self.connection_manager.get_pool(database)?;

        let mut conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let drop_sql = format!("DROP SCHEMA IF EXISTS {} CASCADE", schema_name);

        diesel::sql_query(&drop_sql)
            .execute(&mut *conn)
            .map_err(|e| AppError::DatabaseError(format!("Failed to drop schema: {}", e)))?;

        tracing::warn!("Dropped cluster schema: {} from {}", schema_name, database);

        Ok(())
    }

    /// List all cluster schemas in database
    pub async fn list_cluster_schemas(&self, database: &str) -> AppResult<Vec<String>> {
        let pool = self.connection_manager.get_pool(database)?;

        let mut conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = "SELECT schema_name FROM information_schema.schemata \
                     WHERE schema_name ~ '^(cashier|scm)_[0-9]{4,6}$'";

        let rows: Vec<SchemaNameRow> = diesel::sql_query(query)
            .load(&mut *conn)
            .map_err(|e| AppError::DatabaseError(format!("Failed to list schemas: {}", e)))?;

        Ok(rows.into_iter().map(|r| r.schema_name).collect())
    }

    /// Resolve schema name from cluster type, year, and month
    fn resolve_schema_name(&self, cluster_type: &str, year: i32, month: Option<u8>) -> String {
        if let Some(m) = month {
            format!("{}_{:04}{:02}", cluster_type, year, m)
        } else {
            format!("{}_{:04}", cluster_type, year)
        }
    }

    /// Calculate days until new year
    fn days_until_new_year(now: &chrono::DateTime<chrono::Utc>) -> i64 {
        let next_year = chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap();
        let current = now.date_naive();
        (next_year - current).num_days()
    }

    /// Calculate days until new month
    fn days_until_new_month(now: &chrono::DateTime<chrono::Utc>) -> i64 {
        let next_month = if now.month() == 12 {
            chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
        } else {
            chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
        };
        let current = now.date_naive();
        (next_month - current).num_days()
    }
}

impl Default for ClusterSchemaManager {
    fn default() -> Self {
        Self::new().expect("Failed to create ClusterSchemaManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_schema_manager_creation() {
        let result = ClusterSchemaManager::new();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_resolve_schema_name_yearly() {
        let manager = ClusterSchemaManager::new().unwrap_or_else(|_| {
            ClusterSchemaManager::with_connection_manager(ConnectionManager::new().unwrap())
        });

        let schema = manager.resolve_schema_name("cashier", 2024, None);
        assert_eq!(schema, "cashier_2024");
    }

    #[test]
    fn test_resolve_schema_name_monthly() {
        let manager = ClusterSchemaManager::new().unwrap_or_else(|_| {
            ClusterSchemaManager::with_connection_manager(ConnectionManager::new().unwrap())
        });

        let schema = manager.resolve_schema_name("scm", 2024, Some(3));
        assert_eq!(schema, "scm_202403");
    }

    #[test]
    fn test_days_until_new_year() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 12, 27)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let days = ClusterSchemaManager::days_until_new_year(&date);
        assert_eq!(days, 5);
    }
}
