//! Config Resolver
//!
//! Resolves configuration based on context (repository, project, group, tenant)

use crate::config::{ConfigLoader, ConfigMerger};
use crate::utils::{AppError, AppResult};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Configuration Context
/// Defines the hierarchy for configuration resolution
#[derive(Debug, Clone, Default)]
pub struct ConfigContext {
    /// Repository identifier
    pub repository: Option<String>,
    /// Project identifier
    pub project: Option<String>,
    /// Group ID
    pub group_id: Option<Uuid>,
    /// Tenant ID
    pub tenant_id: Option<Uuid>,
}

impl ConfigContext {
    /// Create new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set repository
    pub fn with_repository(mut self, repository: impl Into<String>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Set project
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Set group ID
    pub fn with_group_id(mut self, group_id: Uuid) -> Self {
        self.group_id = Some(group_id);
        self
    }

    /// Set tenant ID
    pub fn with_tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
}

/// Configuration Resolver
/// Resolves final configuration by merging base + hierarchy configs
pub struct ConfigResolver {
    loader: ConfigLoader,
    merger: ConfigMerger,
    /// Cache for resolved configurations
    cache: HashMap<String, JsonValue>,
}

impl ConfigResolver {
    /// Create new config resolver with default paths
    pub fn new() -> Self {
        Self {
            loader: ConfigLoader::new(),
            merger: ConfigMerger::new(),
            cache: HashMap::new(),
        }
    }

    /// Create resolver with custom base directory
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self {
            loader: ConfigLoader::with_base_dir(base_dir),
            merger: ConfigMerger::new(),
            cache: HashMap::new(),
        }
    }

    /// Resolve configuration for a given context
    pub fn resolve(&mut self, context: &ConfigContext) -> AppResult<JsonValue> {
        // Generate cache key
        let cache_key = self.generate_cache_key(context);

        // Check cache
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Load base configuration
        let base = self.load_base_config()?;

        // Load repository config if specified
        let repository = if let Some(repo) = &context.repository {
            self.load_repository_config(repo)?
        } else {
            None
        };

        // Load project config if specified
        let project = if let Some(proj) = &context.project {
            self.load_project_config(proj)?
        } else {
            None
        };

        // Load group config if specified
        let group = if let Some(group_id) = context.group_id {
            self.load_group_config(group_id)?
        } else {
            None
        };

        // Load tenant config if specified
        let tenant = if let Some(tenant_id) = context.tenant_id {
            self.load_tenant_config(tenant_id)?
        } else {
            None
        };

        // Merge with hierarchy
        let resolved = self.merger.merge_hierarchy(
            Some(base),
            repository,
            project,
            group,
            tenant,
        )?;

        // Cache the result
        self.cache.insert(cache_key, resolved.clone());

        Ok(resolved)
    }

    /// Resolve a specific configuration file with context
    pub fn resolve_file(&mut self, filename: &str, context: &ConfigContext) -> AppResult<JsonValue> {
        // Load base config file
        let base = self.loader.load_from_file(filename)?;

        // Try to load hierarchy-specific versions
        let repository = if let Some(repo) = &context.repository {
            self.load_repository_config_file(repo, filename).ok()
        } else {
            None
        };

        let project = if let Some(proj) = &context.project {
            self.load_project_config_file(proj, filename).ok()
        } else {
            None
        };

        let group = if let Some(group_id) = context.group_id {
            self.load_group_config_file(group_id, filename).ok()
        } else {
            None
        };

        let tenant = if let Some(tenant_id) = context.tenant_id {
            self.load_tenant_config_file(tenant_id, filename).ok()
        } else {
            None
        };

        // Merge with hierarchy
        self.merger.merge_hierarchy(
            Some(base),
            repository,
            project,
            group,
            tenant,
        )
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.loader.clear_cache();
    }

    /// Generate cache key from context
    fn generate_cache_key(&self, context: &ConfigContext) -> String {
        format!(
            "repo:{}_proj:{}_group:{}_tenant:{}",
            context.repository.as_deref().unwrap_or("_"),
            context.project.as_deref().unwrap_or("_"),
            context.group_id.map(|id| id.to_string()).unwrap_or_else(|| "_".to_string()),
            context.tenant_id.map(|id| id.to_string()).unwrap_or_else(|| "_".to_string())
        )
    }

    /// Load base configuration (merges all base config files)
    fn load_base_config(&mut self) -> AppResult<JsonValue> {
        // Load common config files
        let configs = self.loader.load_multiple(&[
            "app.toml",
            "database.toml",
            "jwt.toml",
        ]).unwrap_or_else(|_| vec![]);

        if configs.is_empty() {
            return Err(AppError::ConfigError("No base configuration files found".to_string()));
        }

        self.merger.merge(configs)
    }

    /// Load repository-specific configuration
    fn load_repository_config(&mut self, repository: &str) -> AppResult<Option<JsonValue>> {
        let filename = format!("repositories/{}.toml", repository);
        match self.loader.load_from_file(&filename) {
            Ok(config) => Ok(Some(config)),
            Err(_) => Ok(None), // Repository config is optional
        }
    }

    /// Load project-specific configuration
    fn load_project_config(&mut self, project: &str) -> AppResult<Option<JsonValue>> {
        let filename = format!("projects/{}.toml", project);
        match self.loader.load_from_file(&filename) {
            Ok(config) => Ok(Some(config)),
            Err(_) => Ok(None), // Project config is optional
        }
    }

    /// Load group-specific configuration
    fn load_group_config(&mut self, group_id: Uuid) -> AppResult<Option<JsonValue>> {
        let filename = format!("groups/{}.toml", group_id);
        match self.loader.load_from_file(&filename) {
            Ok(config) => Ok(Some(config)),
            Err(_) => Ok(None), // Group config is optional
        }
    }

    /// Load tenant-specific configuration
    fn load_tenant_config(&mut self, tenant_id: Uuid) -> AppResult<Option<JsonValue>> {
        let filename = format!("tenants/{}.toml", tenant_id);
        match self.loader.load_from_file(&filename) {
            Ok(config) => Ok(Some(config)),
            Err(_) => Ok(None), // Tenant config is optional
        }
    }

    /// Load repository-specific config file
    fn load_repository_config_file(&mut self, repository: &str, filename: &str) -> AppResult<JsonValue> {
        let path = format!("repositories/{}/{}", repository, filename);
        self.loader.load_from_file(&path)
    }

    /// Load project-specific config file
    fn load_project_config_file(&mut self, project: &str, filename: &str) -> AppResult<JsonValue> {
        let path = format!("projects/{}/{}", project, filename);
        self.loader.load_from_file(&path)
    }

    /// Load group-specific config file
    fn load_group_config_file(&mut self, group_id: Uuid, filename: &str) -> AppResult<JsonValue> {
        let path = format!("groups/{}/{}", group_id, filename);
        self.loader.load_from_file(&path)
    }

    /// Load tenant-specific config file
    fn load_tenant_config_file(&mut self, tenant_id: Uuid, filename: &str) -> AppResult<JsonValue> {
        let path = format!("tenants/{}/{}", tenant_id, filename);
        self.loader.load_from_file(&path)
    }

    /// Get reference to loader
    pub fn loader(&self) -> &ConfigLoader {
        &self.loader
    }

    /// Get mutable reference to loader
    pub fn loader_mut(&mut self) -> &mut ConfigLoader {
        &mut self.loader
    }

    /// Get reference to merger
    pub fn merger(&self) -> &ConfigMerger {
        &self.merger
    }

    /// Get mutable reference to merger
    pub fn merger_mut(&mut self) -> &mut ConfigMerger {
        &mut self.merger
    }
}

impl Default for ConfigResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_configs(dir: &std::path::Path) {
        // Base configs
        fs::write(
            dir.join("app.toml"),
            r#"
            [app]
            name = "Base App"
            port = 8000
            "#,
        )
        .unwrap();

        fs::write(
            dir.join("database.toml"),
            r#"
            [database]
            host = "localhost"
            "#,
        )
        .unwrap();

        fs::write(
            dir.join("jwt.toml"),
            r#"
            [jwt]
            secret = "base-secret"
            "#,
        )
        .unwrap();

        // Repository config
        fs::create_dir_all(dir.join("repositories")).unwrap();
        fs::write(
            dir.join("repositories/test-repo.toml"),
            r#"
            [app]
            name = "Repo App"
            "#,
        )
        .unwrap();

        // Tenant config
        let tenant_id = Uuid::new_v4();
        fs::create_dir_all(dir.join("tenants")).unwrap();
        fs::write(
            dir.join(format!("tenants/{}.toml", tenant_id)),
            r#"
            [app]
            port = 9000
            "#,
        )
        .unwrap();
    }

    #[test]
    fn test_config_context() {
        let tenant_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();

        let context = ConfigContext::new()
            .with_repository("test-repo")
            .with_project("test-project")
            .with_group_id(group_id)
            .with_tenant_id(tenant_id);

        assert_eq!(context.repository, Some("test-repo".to_string()));
        assert_eq!(context.project, Some("test-project".to_string()));
        assert_eq!(context.group_id, Some(group_id));
        assert_eq!(context.tenant_id, Some(tenant_id));
    }

    #[test]
    fn test_resolve_base_only() {
        let temp_dir = TempDir::new().unwrap();
        create_test_configs(temp_dir.path());

        let mut resolver = ConfigResolver::with_base_dir(temp_dir.path().to_path_buf());
        let context = ConfigContext::new();

        let config = resolver.resolve(&context).unwrap();

        assert_eq!(config["app"]["name"], "Base App");
        assert_eq!(config["app"]["port"], 8000);
        assert_eq!(config["database"]["host"], "localhost");
        assert_eq!(config["jwt"]["secret"], "base-secret");
    }

    #[test]
    fn test_cache_key_generation() {
        let resolver = ConfigResolver::new();
        let tenant_id = Uuid::new_v4();

        let context = ConfigContext::new()
            .with_repository("repo1")
            .with_tenant_id(tenant_id);

        let key = resolver.generate_cache_key(&context);
        assert!(key.contains("repo1"));
        assert!(key.contains(&tenant_id.to_string()));
    }

    #[test]
    fn test_clear_cache() {
        let temp_dir = TempDir::new().unwrap();
        create_test_configs(temp_dir.path());

        let mut resolver = ConfigResolver::with_base_dir(temp_dir.path().to_path_buf());
        let context = ConfigContext::new();

        // First resolve (populates cache)
        let _ = resolver.resolve(&context).unwrap();
        assert!(!resolver.cache.is_empty());

        // Clear cache
        resolver.clear_cache();
        assert!(resolver.cache.is_empty());
    }
}
