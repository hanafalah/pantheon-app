//! Template Definitions

use std::collections::HashMap;

/// Template types for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateType {
    /// Entity/Model template
    Entity,
    /// Resource (View) template
    ViewResource,
    /// Resource (Show) template
    ShowResource,
    /// Controller template
    Controller,
    /// Service template
    Service,
    /// Migration template
    Migration,
    /// ServiceProvider template
    ServiceProvider,
}

impl TemplateType {
    /// Get template name
    pub fn name(&self) -> &'static str {
        match self {
            TemplateType::Entity => "entity",
            TemplateType::ViewResource => "view_resource",
            TemplateType::ShowResource => "show_resource",
            TemplateType::Controller => "controller",
            TemplateType::Service => "service",
            TemplateType::Migration => "migration",
            TemplateType::ServiceProvider => "service_provider",
        }
    }

    /// Get file suffix
    pub fn suffix(&self) -> &'static str {
        match self {
            TemplateType::Entity => "entity.rs",
            TemplateType::ViewResource => "view_resource.rs",
            TemplateType::ShowResource => "show_resource.rs",
            TemplateType::Controller => "controller.rs",
            TemplateType::Service => "service.rs",
            TemplateType::Migration => "migration.rs",
            TemplateType::ServiceProvider => "provider.rs",
        }
    }

    /// Get all template types
    pub fn all() -> Vec<TemplateType> {
        vec![
            TemplateType::Entity,
            TemplateType::ViewResource,
            TemplateType::ShowResource,
            TemplateType::Controller,
            TemplateType::Service,
            TemplateType::Migration,
            TemplateType::ServiceProvider,
        ]
    }
}

/// Template context for rendering
#[derive(Debug, Clone)]
pub struct TemplateContext {
    /// Context variables
    vars: HashMap<String, String>,
}

impl TemplateContext {
    /// Create new template context
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    /// Add variable to context
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    /// Get variable from context
    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    /// Get all variables
    pub fn vars(&self) -> &HashMap<String, String> {
        &self.vars
    }

    /// Set module name (common variable)
    pub fn with_module_name(self, name: impl Into<String>) -> Self {
        self.with_var("module_name", name)
    }

    /// Set table name (common variable)
    pub fn with_table_name(self, name: impl Into<String>) -> Self {
        self.with_var("table_name", name)
    }

    /// Set struct name (common variable)
    pub fn with_struct_name(self, name: impl Into<String>) -> Self {
        self.with_var("struct_name", name)
    }
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_type_name() {
        assert_eq!(TemplateType::Entity.name(), "entity");
        assert_eq!(TemplateType::Controller.name(), "controller");
    }

    #[test]
    fn test_template_type_suffix() {
        assert_eq!(TemplateType::Entity.suffix(), "entity.rs");
        assert_eq!(TemplateType::Controller.suffix(), "controller.rs");
    }

    #[test]
    fn test_template_type_all() {
        let all = TemplateType::all();
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn test_template_context() {
        let context = TemplateContext::new()
            .with_var("name", "Test")
            .with_var("value", "123");

        assert_eq!(context.get("name").unwrap(), "Test");
        assert_eq!(context.get("value").unwrap(), "123");
        assert_eq!(context.get("missing"), None);
    }

    #[test]
    fn test_template_context_builders() {
        let context = TemplateContext::new()
            .with_module_name("users")
            .with_table_name("users")
            .with_struct_name("User");

        assert_eq!(context.get("module_name").unwrap(), "users");
        assert_eq!(context.get("table_name").unwrap(), "users");
        assert_eq!(context.get("struct_name").unwrap(), "User");
    }
}
