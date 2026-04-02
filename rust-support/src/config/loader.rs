//! Config Loader
//!
//! Loads TOML configuration files and converts them to JSON values

use crate::utils::{AppError, AppResult};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

/// Configuration Loader
/// Loads TOML files from disk and converts to JSON
pub struct ConfigLoader {
    /// Base directory for configuration files
    base_dir: PathBuf,
    /// Cached configurations
    cache: HashMap<String, JsonValue>,
}

impl ConfigLoader {
    /// Create new config loader with default base directory (./config)
    pub fn new() -> Self {
        Self {
            base_dir: PathBuf::from("config"),
            cache: HashMap::new(),
        }
    }

    /// Create config loader with custom base directory
    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cache: HashMap::new(),
        }
    }

    /// Load configuration from TOML file
    pub fn load_from_file(&mut self, filename: &str) -> AppResult<JsonValue> {
        // Check cache first
        if let Some(cached) = self.cache.get(filename) {
            return Ok(cached.clone());
        }

        // Build full path
        let file_path = self.base_dir.join(filename);

        // Read file
        let content = fs::read_to_string(&file_path).map_err(|e| {
            AppError::ConfigError(format!("Failed to read config file {}: {}", filename, e))
        })?;

        // Parse TOML
        let toml_value: TomlValue = toml::from_str(&content).map_err(|e| {
            AppError::ConfigError(format!("Failed to parse TOML in {}: {}", filename, e))
        })?;

        // Convert TOML to JSON
        let json_value = Self::toml_to_json(toml_value)?;

        // Cache the result
        self.cache.insert(filename.to_string(), json_value.clone());

        Ok(json_value)
    }

    /// Load multiple configuration files
    pub fn load_multiple(&mut self, filenames: &[&str]) -> AppResult<Vec<JsonValue>> {
        filenames
            .iter()
            .map(|filename| self.load_from_file(filename))
            .collect()
    }

    /// Load all configuration files from a directory
    pub fn load_directory<P: AsRef<Path>>(&mut self, dir: P) -> AppResult<Vec<JsonValue>> {
        let dir_path = self.base_dir.join(dir);

        let entries = fs::read_dir(&dir_path).map_err(|e| {
            AppError::ConfigError(format!(
                "Failed to read config directory {:?}: {}",
                dir_path, e
            ))
        })?;

        let mut configs = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                AppError::ConfigError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            // Only process .toml files
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    let config = self.load_from_file(filename)?;
                    configs.push(config);
                }
            }
        }

        Ok(configs)
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Reload a specific config file (bypassing cache)
    pub fn reload(&mut self, filename: &str) -> AppResult<JsonValue> {
        self.cache.remove(filename);
        self.load_from_file(filename)
    }

    /// Convert TOML value to JSON value
    fn toml_to_json(toml: TomlValue) -> AppResult<JsonValue> {
        let json_str = serde_json::to_string(&toml).map_err(|e| {
            AppError::ConfigError(format!("Failed to convert TOML to JSON: {}", e))
        })?;

        let json_value: JsonValue = serde_json::from_str(&json_str).map_err(|e| {
            AppError::ConfigError(format!("Failed to parse JSON: {}", e))
        })?;

        Ok(json_value)
    }

    /// Get the base directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(dir: &Path, filename: &str, content: &str) {
        fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config(
            temp_dir.path(),
            "test.toml",
            r#"
            [app]
            name = "Test"
            port = 8000
            "#,
        );

        let mut loader = ConfigLoader::with_base_dir(temp_dir.path());
        let config = loader.load_from_file("test.toml").unwrap();

        assert!(config.is_object());
        assert_eq!(config["app"]["name"], "Test");
        assert_eq!(config["app"]["port"], 8000);
    }

    #[test]
    fn test_cache() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config(temp_dir.path(), "test.toml", r#"[app]\nname = "Test""#);

        let mut loader = ConfigLoader::with_base_dir(temp_dir.path());

        // First load
        let config1 = loader.load_from_file("test.toml").unwrap();
        // Second load (from cache)
        let config2 = loader.load_from_file("test.toml").unwrap();

        assert_eq!(config1, config2);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let mut loader = ConfigLoader::with_base_dir(temp_dir.path());

        let result = loader.load_from_file("nonexistent.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_multiple() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config(temp_dir.path(), "app.toml", r#"[app]\nname = "Test""#);
        create_test_config(temp_dir.path(), "db.toml", r#"[database]\nhost = "localhost""#);

        let mut loader = ConfigLoader::with_base_dir(temp_dir.path());
        let configs = loader.load_multiple(&["app.toml", "db.toml"]).unwrap();

        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0]["app"]["name"], "Test");
        assert_eq!(configs[1]["database"]["host"], "localhost");
    }

    #[test]
    fn test_reload() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config(temp_dir.path(), "test.toml", r#"[app]\nname = "Test1""#);

        let mut loader = ConfigLoader::with_base_dir(temp_dir.path());
        let config1 = loader.load_from_file("test.toml").unwrap();
        assert_eq!(config1["app"]["name"], "Test1");

        // Update file
        create_test_config(temp_dir.path(), "test.toml", r#"[app]\nname = "Test2""#);

        // Without reload (should get cached value)
        let config2 = loader.load_from_file("test.toml").unwrap();
        assert_eq!(config2["app"]["name"], "Test1");

        // With reload
        let config3 = loader.reload("test.toml").unwrap();
        assert_eq!(config3["app"]["name"], "Test2");
    }
}
