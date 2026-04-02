//! Rust Package Generator
//!
//! CLI tool for generating new Rust modules

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rust_core::*;
use rust_stub::{StubGenerator, TemplateType};
use rust_stub::templates::TemplateContext;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "pkg-gen")]
#[command(about = "Generate new Rust packages/modules", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new module
    New {
        /// Module name (e.g., "users", "products")
        #[arg(short, long)]
        name: String,

        /// Module type (repository/project/group/tenant)
        #[arg(short, long, default_value = "repository")]
        type_: String,

        /// Struct name (e.g., "User", "Product")
        #[arg(short, long)]
        struct_name: Option<String>,

        /// Table name (defaults to module name)
        #[arg(short = 't', long)]
        table: Option<String>,
    },

    /// Generate entity files for existing module
    Entity {
        /// Module path
        #[arg(short, long)]
        module: String,

        /// Entity name (e.g., "User", "Product")
        #[arg(short, long)]
        name: String,

        /// Table name (defaults to snake_case of entity name)
        #[arg(short, long)]
        table: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            type_,
            struct_name,
            table,
        } => generate_new_module(&name, &type_, struct_name, table)?,
        Commands::Entity {
            module,
            name,
            table,
        } => generate_entity(&module, &name, table)?,
    }

    Ok(())
}

fn generate_new_module(
    name: &str,
    type_: &str,
    struct_name: Option<String>,
    table: Option<String>,
) -> Result<()> {
    println!("Generating new {} module: {}", type_, name);

    // Determine base path
    let base_path = match type_ {
        "repository" => PathBuf::from("repositories"),
        "project" => PathBuf::from("projects"),
        "group" => PathBuf::from("groups"),
        "tenant" => PathBuf::from("tenants"),
        _ => anyhow::bail!("Invalid module type: {}", type_),
    };

    let module_path = base_path.join(name);

    // Create module directory structure
    create_module_structure(&module_path)?;

    // Determine struct name
    let struct_name = struct_name.unwrap_or_else(|| str_pascal(name));

    // Determine table name
    let table_name = table.unwrap_or_else(|| str_snake(name));

    // Generate Cargo.toml
    generate_cargo_toml(&module_path, name)?;

    // Generate all files
    generate_all_files(&module_path, name, &struct_name, &table_name)?;

    println!("✅ Module generated successfully at: {}", module_path.display());
    println!("\nNext steps:");
    println!("1. Update workspace Cargo.toml to include this module");
    println!("2. Implement the TODO sections in generated files");
    println!("3. Run: cargo build -p {}", name);

    Ok(())
}

fn generate_entity(module: &str, name: &str, table: Option<String>) -> Result<()> {
    println!("Generating entity {} for module {}", name, module);

    let module_path = PathBuf::from(module);
    if !module_path.exists() {
        anyhow::bail!("Module not found: {}", module);
    }

    let table_name = table.unwrap_or_else(|| str_snake(name));

    // Generate entity files
    let generator = StubGenerator::new()?;
    let context = TemplateContext::new()
        .with_module_name(&str_snake(name))
        .with_table_name(&table_name)
        .with_struct_name(name);

    // Generate entity
    let entity_code = generator.generate(TemplateType::Entity, &context)?;
    let entity_path = module_path.join("src/entities").join(format!("{}.rs", str_snake(name)));
    fs::write(&entity_path, entity_code).context("Failed to write entity file")?;

    // Generate resources
    let view_resource_code = generator.generate(TemplateType::ViewResource, &context)?;
    let view_path = module_path
        .join("src/resources")
        .join(format!("{}_view_resource.rs", str_snake(name)));
    fs::write(&view_path, view_resource_code).context("Failed to write view resource file")?;

    let show_resource_code = generator.generate(TemplateType::ShowResource, &context)?;
    let show_path = module_path
        .join("src/resources")
        .join(format!("{}_show_resource.rs", str_snake(name)));
    fs::write(&show_path, show_resource_code).context("Failed to write show resource file")?;

    println!("✅ Entity generated successfully!");
    println!("\nGenerated files:");
    println!("  - {}", entity_path.display());
    println!("  - {}", view_path.display());
    println!("  - {}", show_path.display());

    Ok(())
}

fn create_module_structure(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    fs::create_dir_all(path.join("src/entities"))?;
    fs::create_dir_all(path.join("src/resources"))?;
    fs::create_dir_all(path.join("src/controllers"))?;
    fs::create_dir_all(path.join("src/services"))?;
    fs::create_dir_all(path.join("migrations"))?;

    Ok(())
}

fn generate_cargo_toml(path: &Path, name: &str) -> Result<()> {
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
name = "{}"
path = "src/lib.rs"

[dependencies]
rust-support = {{ path = "../../rust-support" }}
rust-core = {{ path = "../../rust-core" }}

actix-web = {{ workspace = true }}
diesel = {{ workspace = true }}
uuid = {{ workspace = true }}
chrono = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
async-trait = {{ workspace = true }}
"#,
        name,
        name.replace('-', "_")
    );

    fs::write(path.join("Cargo.toml"), cargo_toml)?;

    Ok(())
}

fn generate_all_files(
    path: &Path,
    module_name: &str,
    struct_name: &str,
    table_name: &str,
) -> Result<()> {
    let generator = StubGenerator::new()?;
    let context = TemplateContext::new()
        .with_module_name(module_name)
        .with_table_name(table_name)
        .with_struct_name(struct_name);

    // Generate lib.rs
    let lib_rs = format!(
        r#"//! {} Module

pub mod entities;
pub mod resources;
pub mod controllers;
pub mod services;

// Re-exports
pub use entities::*;
pub use resources::*;
"#,
        struct_name
    );
    fs::write(path.join("src/lib.rs"), lib_rs)?;

    // Generate entities/mod.rs
    let entities_mod = format!(
        "//! Entities\n\npub mod {};\n\npub use {}::{};\n",
        str_snake(struct_name),
        str_snake(struct_name),
        struct_name
    );
    fs::write(path.join("src/entities/mod.rs"), entities_mod)?;

    // Generate resources/mod.rs
    let resources_mod = format!(
        "//! Resources\n\npub mod {}_view_resource;\npub mod {}_show_resource;\n\npub use {}_view_resource::{}ViewResource;\npub use {}_show_resource::{}ShowResource;\n",
        str_snake(struct_name),
        str_snake(struct_name),
        str_snake(struct_name),
        struct_name,
        str_snake(struct_name),
        struct_name
    );
    fs::write(path.join("src/resources/mod.rs"), resources_mod)?;

    // Generate controllers/mod.rs
    fs::write(path.join("src/controllers/mod.rs"), "//! Controllers\n")?;

    // Generate services/mod.rs
    fs::write(path.join("src/services/mod.rs"), "//! Services\n")?;

    // Generate entity
    let entity_code = generator.generate(TemplateType::Entity, &context)?;
    fs::write(
        path.join("src/entities").join(format!("{}.rs", str_snake(struct_name))),
        entity_code,
    )?;

    // Generate resources
    let view_resource = generator.generate(TemplateType::ViewResource, &context)?;
    fs::write(
        path.join("src/resources")
            .join(format!("{}_view_resource.rs", str_snake(struct_name))),
        view_resource,
    )?;

    let show_resource = generator.generate(TemplateType::ShowResource, &context)?;
    fs::write(
        path.join("src/resources")
            .join(format!("{}_show_resource.rs", str_snake(struct_name))),
        show_resource,
    )?;

    // Generate migration
    let migration = generator.generate(TemplateType::Migration, &context)?;
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    fs::write(
        path.join("migrations")
            .join(format!("{}_{}.sql", timestamp, table_name)),
        migration,
    )?;

    // Generate README
    let readme = format!(
        r#"# {}

Auto-generated module by rust-package-generator.

## Structure

- `src/entities/` - Database models
- `src/resources/` - API resource transformations
- `src/controllers/` - HTTP controllers
- `src/services/` - Business logic
- `migrations/` - Database migrations

## Usage

```rust
use {}::{{{}ViewResource, {}ShowResource}};
```
"#,
        struct_name, module_name, struct_name, struct_name
    );
    fs::write(path.join("README.md"), readme)?;

    Ok(())
}
