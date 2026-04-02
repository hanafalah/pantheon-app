//! Config Merger
//!
//! Merges multiple configurations with deep merge support

use crate::utils::{AppError, AppResult};
use serde_json::{Map, Value as JsonValue};

/// Configuration Merger
/// Merges multiple JSON configurations with hierarchy support
pub struct ConfigMerger {
    /// Strategy for array merging
    array_strategy: ArrayMergeStrategy,
}

/// Strategy for merging arrays
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayMergeStrategy {
    /// Replace the entire array with the newer one
    Replace,
    /// Concatenate arrays (older + newer)
    Concat,
    /// Merge arrays by index
    MergeByIndex,
}

impl ConfigMerger {
    /// Create new config merger with default strategy (Replace)
    pub fn new() -> Self {
        Self {
            array_strategy: ArrayMergeStrategy::Replace,
        }
    }

    /// Create merger with custom array strategy
    pub fn with_array_strategy(strategy: ArrayMergeStrategy) -> Self {
        Self {
            array_strategy: strategy,
        }
    }

    /// Merge multiple configurations
    /// Later configs override earlier ones (left to right)
    pub fn merge(&self, configs: Vec<JsonValue>) -> AppResult<JsonValue> {
        if configs.is_empty() {
            return Ok(JsonValue::Object(Map::new()));
        }

        let mut result = configs[0].clone();

        for config in configs.iter().skip(1) {
            result = self.merge_two(&result, config)?;
        }

        Ok(result)
    }

    /// Merge two configurations
    pub fn merge_two(&self, base: &JsonValue, overlay: &JsonValue) -> AppResult<JsonValue> {
        match (base, overlay) {
            // Both are objects - deep merge
            (JsonValue::Object(base_map), JsonValue::Object(overlay_map)) => {
                let mut merged = base_map.clone();

                for (key, overlay_value) in overlay_map {
                    if let Some(base_value) = base_map.get(key) {
                        // Key exists in both - recursively merge
                        merged.insert(key.clone(), self.merge_two(base_value, overlay_value)?);
                    } else {
                        // Key only in overlay - add it
                        merged.insert(key.clone(), overlay_value.clone());
                    }
                }

                Ok(JsonValue::Object(merged))
            }

            // Both are arrays - merge based on strategy
            (JsonValue::Array(base_arr), JsonValue::Array(overlay_arr)) => {
                let merged = match self.array_strategy {
                    ArrayMergeStrategy::Replace => overlay_arr.clone(),
                    ArrayMergeStrategy::Concat => {
                        let mut result = base_arr.clone();
                        result.extend(overlay_arr.clone());
                        result
                    }
                    ArrayMergeStrategy::MergeByIndex => {
                        let mut result = base_arr.clone();
                        for (i, overlay_item) in overlay_arr.iter().enumerate() {
                            if i < result.len() {
                                result[i] = self.merge_two(&result[i], overlay_item)?;
                            } else {
                                result.push(overlay_item.clone());
                            }
                        }
                        result
                    }
                };

                Ok(JsonValue::Array(merged))
            }

            // Different types or primitives - overlay wins
            _ => Ok(overlay.clone()),
        }
    }

    /// Merge with hierarchy
    /// Configs are merged in order: base → repository → project → group → tenant
    pub fn merge_hierarchy(
        &self,
        base: Option<JsonValue>,
        repository: Option<JsonValue>,
        project: Option<JsonValue>,
        group: Option<JsonValue>,
        tenant: Option<JsonValue>,
    ) -> AppResult<JsonValue> {
        let mut configs = Vec::new();

        if let Some(config) = base {
            configs.push(config);
        }
        if let Some(config) = repository {
            configs.push(config);
        }
        if let Some(config) = project {
            configs.push(config);
        }
        if let Some(config) = group {
            configs.push(config);
        }
        if let Some(config) = tenant {
            configs.push(config);
        }

        self.merge(configs)
    }

    /// Get array merge strategy
    pub fn array_strategy(&self) -> ArrayMergeStrategy {
        self.array_strategy
    }

    /// Set array merge strategy
    pub fn set_array_strategy(&mut self, strategy: ArrayMergeStrategy) {
        self.array_strategy = strategy;
    }
}

impl Default for ConfigMerger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects() {
        let merger = ConfigMerger::new();

        let base = json!({
            "app": {
                "name": "Base",
                "port": 8000
            }
        });

        let overlay = json!({
            "app": {
                "name": "Overlay",
                "host": "localhost"
            }
        });

        let result = merger.merge_two(&base, &overlay).unwrap();

        assert_eq!(result["app"]["name"], "Overlay"); // overridden
        assert_eq!(result["app"]["port"], 8000); // preserved
        assert_eq!(result["app"]["host"], "localhost"); // added
    }

    #[test]
    fn test_merge_arrays_replace() {
        let merger = ConfigMerger::with_array_strategy(ArrayMergeStrategy::Replace);

        let base = json!({ "items": [1, 2, 3] });
        let overlay = json!({ "items": [4, 5] });

        let result = merger.merge_two(&base, &overlay).unwrap();
        assert_eq!(result["items"], json!([4, 5]));
    }

    #[test]
    fn test_merge_arrays_concat() {
        let merger = ConfigMerger::with_array_strategy(ArrayMergeStrategy::Concat);

        let base = json!({ "items": [1, 2, 3] });
        let overlay = json!({ "items": [4, 5] });

        let result = merger.merge_two(&base, &overlay).unwrap();
        assert_eq!(result["items"], json!([1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_merge_arrays_by_index() {
        let merger = ConfigMerger::with_array_strategy(ArrayMergeStrategy::MergeByIndex);

        let base = json!({ "items": [{"id": 1, "name": "A"}, {"id": 2, "name": "B"}] });
        let overlay = json!({ "items": [{"name": "C"}, {"id": 3}] });

        let result = merger.merge_two(&base, &overlay).unwrap();
        assert_eq!(result["items"][0]["id"], 1);
        assert_eq!(result["items"][0]["name"], "C");
        assert_eq!(result["items"][1]["id"], 3);
        assert_eq!(result["items"][1]["name"], "B");
    }

    #[test]
    fn test_merge_multiple() {
        let merger = ConfigMerger::new();

        let configs = vec![
            json!({"a": 1, "b": 2}),
            json!({"b": 3, "c": 4}),
            json!({"c": 5, "d": 6}),
        ];

        let result = merger.merge(configs).unwrap();

        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 3);
        assert_eq!(result["c"], 5);
        assert_eq!(result["d"], 6);
    }

    #[test]
    fn test_merge_hierarchy() {
        let merger = ConfigMerger::new();

        let base = json!({"app": {"name": "Base", "port": 8000}});
        let repository = json!({"app": {"name": "Repo"}});
        let project = json!({"app": {"host": "localhost"}});
        let tenant = json!({"app": {"port": 9000}});

        let result = merger
            .merge_hierarchy(
                Some(base),
                Some(repository),
                Some(project),
                None,
                Some(tenant),
            )
            .unwrap();

        assert_eq!(result["app"]["name"], "Repo");
        assert_eq!(result["app"]["port"], 9000);
        assert_eq!(result["app"]["host"], "localhost");
    }

    #[test]
    fn test_merge_empty() {
        let merger = ConfigMerger::new();
        let result = merger.merge(vec![]).unwrap();
        assert!(result.is_object());
        assert!(result.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_merge_primitives() {
        let merger = ConfigMerger::new();

        let base = json!({"value": 42});
        let overlay = json!({"value": "text"});

        let result = merger.merge_two(&base, &overlay).unwrap();
        assert_eq!(result["value"], "text");
    }

    #[test]
    fn test_deep_nested_merge() {
        let merger = ConfigMerger::new();

        let base = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "a": 1,
                        "b": 2
                    }
                }
            }
        });

        let overlay = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "b": 3,
                        "c": 4
                    }
                }
            }
        });

        let result = merger.merge_two(&base, &overlay).unwrap();
        assert_eq!(result["level1"]["level2"]["level3"]["a"], 1);
        assert_eq!(result["level1"]["level2"]["level3"]["b"], 3);
        assert_eq!(result["level1"]["level2"]["level3"]["c"], 4);
    }
}
