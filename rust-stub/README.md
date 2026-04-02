# rust-stub

Template engine and stub files for code generation - Laravel Stub equivalent.

## Features

- Tera template engine for flexible code generation
- Pre-built templates for common patterns:
  - Entity/Model
  - ViewResource and ShowResource
  - Controller with CRUD operations
  - Service with business logic
  - SQL Migration
  - Service Provider

## Template Types

- **Entity**: Database model with BaseEntity trait implementation
- **ViewResource**: List view transformation
- **ShowResource**: Detail view transformation
- **Controller**: HTTP controller with routes and handlers
- **Service**: Business logic layer
- **Migration**: SQL schema definition
- **ServiceProvider**: Module initialization

## Usage

```rust
use rust_stub::{StubGenerator, TemplateContext, TemplateType};

// Create generator
let generator = StubGenerator::new()?;

// Create context
let context = TemplateContext::new()
    .with_module_name("users")
    .with_table_name("users")
    .with_struct_name("User");

// Generate code
let code = generator.generate(TemplateType::Entity, &context)?;
println!("{}", code);
```

## Template Variables

Common variables available in all templates:

- `{{ module_name }}` - Module name (e.g., "users")
- `{{ table_name }}` - Database table name (e.g., "users")
- `{{ struct_name }}` - Struct name (e.g., "User")

## Custom Templates

You can add custom variables to the context:

```rust
let context = TemplateContext::new()
    .with_var("author", "Your Name")
    .with_var("version", "1.0.0");
```

## Template Files

Templates are located in `templates/`:

- `entity.tera` - Entity/Model template
- `view_resource.tera` - ViewResource template
- `show_resource.tera` - ShowResource template
- `controller.tera` - Controller template
- `service.tera` - Service template
- `migration.tera` - Migration template
- `service_provider.tera` - ServiceProvider template

## Testing

```bash
cargo test -p rust-stub
```
