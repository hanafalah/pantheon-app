//! Stub Generator

use crate::templates::{TemplateContext, TemplateType};
use anyhow::{Context as _, Result};
use std::collections::HashMap;
use std::path::Path;
use tera::Tera;

/// Stub Generator
pub struct StubGenerator {
    /// Tera template engine
    tera: Tera,
}

impl StubGenerator {
    /// Create new stub generator
    pub fn new() -> Result<Self> {
        // Load templates from embedded strings
        let mut tera = Tera::default();

        // Register built-in templates
        Self::register_templates(&mut tera)?;

        Ok(Self { tera })
    }

    /// Create generator with custom template directory
    pub fn with_template_dir(template_dir: &Path) -> Result<Self> {
        let pattern = format!("{}/**/*.tera", template_dir.display());
        let tera = Tera::new(&pattern).context("Failed to load templates")?;

        Ok(Self { tera })
    }

    /// Generate code from template
    pub fn generate(
        &self,
        template_type: TemplateType,
        context: &TemplateContext,
    ) -> Result<String> {
        let template_name = template_type.name();

        // Convert context to tera context
        let mut tera_context = tera::Context::new();
        for (key, value) in context.vars() {
            tera_context.insert(key, value);
        }

        // Render template
        self.tera
            .render(template_name, &tera_context)
            .context(format!("Failed to render template: {}", template_name))
    }

    /// Generate multiple templates
    pub fn generate_all(
        &self,
        templates: &[(TemplateType, TemplateContext)],
    ) -> Result<HashMap<TemplateType, String>> {
        let mut results = HashMap::new();

        for (template_type, context) in templates {
            let code = self.generate(*template_type, context)?;
            results.insert(*template_type, code);
        }

        Ok(results)
    }

    /// Register built-in templates
    fn register_templates(tera: &mut Tera) -> Result<()> {
        // Entity template
        tera.add_raw_template(
            "entity",
            include_str!("../templates/entity.tera"),
        )?;

        // View Resource template
        tera.add_raw_template(
            "view_resource",
            include_str!("../templates/view_resource.tera"),
        )?;

        // Show Resource template
        tera.add_raw_template(
            "show_resource",
            include_str!("../templates/show_resource.tera"),
        )?;

        // Controller template
        tera.add_raw_template(
            "controller",
            include_str!("../templates/controller.tera"),
        )?;

        // Service template
        tera.add_raw_template(
            "service",
            include_str!("../templates/service.tera"),
        )?;

        // Migration template
        tera.add_raw_template(
            "migration",
            include_str!("../templates/migration.tera"),
        )?;

        // ServiceProvider template
        tera.add_raw_template(
            "service_provider",
            include_str!("../templates/service_provider.tera"),
        )?;

        Ok(())
    }
}

impl Default for StubGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create StubGenerator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_generator_creation() {
        let result = StubGenerator::new();
        assert!(result.is_ok());
    }
}
