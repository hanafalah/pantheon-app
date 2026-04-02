# rust-package-generator

CLI tool for generating new Rust modules - Laravel Package Generator equivalent.

## Features

- Generate complete module structure with one command
- Auto-generate Cargo.toml with dependencies
- Create entity, resources, controllers, services folders
- Generate migration files with timestamps
- Support for repository/project/group/tenant modules

## Installation

```bash
cargo install --path .
```

## Usage

### Generate New Module

```bash
# Generate a new repository module
pkg-gen new --name users --type repository

# Generate with custom struct name
pkg-gen new --name users --struct-name User --table users

# Generate project module
pkg-gen new --name billing --type project
```

### Generate Entity for Existing Module

```bash
# Add new entity to existing module
pkg-gen entity --module repositories/users --name Role

# With custom table name
pkg-gen entity --module repositories/users --name Permission --table user_permissions
```

## Module Structure

Generated modules follow this structure:

```
module-name/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── entities/
│   │   ├── mod.rs
│   │   └── entity_name.rs
│   ├── resources/
│   │   ├── mod.rs
│   │   ├── entity_view_resource.rs
│   │   └── entity_show_resource.rs
│   ├── controllers/
│   │   └── mod.rs
│   └── services/
│       └── mod.rs
└── migrations/
    └── 20240101000000_create_table.sql
```

## Generated Files

1. **Entity** - Database model with BaseEntity trait
2. **ViewResource** - List view transformation
3. **ShowResource** - Detail view transformation
4. **Migration** - SQL schema definition
5. **Cargo.toml** - Package manifest with dependencies
6. **lib.rs** - Module entry point
7. **README.md** - Module documentation

## Module Types

- `repository` - Foundation modules (users, regional, etc.)
- `project` - Project-specific modules
- `group` - Group-specific modules
- `tenant` - Tenant-specific modules

## Examples

```bash
# Generate users module
pkg-gen new --name users --struct-name User

# Generate products module for tenant
pkg-gen new --name products --type tenant --struct-name Product

# Add Role entity to users module
pkg-gen entity --module repositories/users --name Role
```

## Next Steps After Generation

1. Add module to workspace Cargo.toml
2. Implement TODO sections in generated files
3. Run database migrations
4. Implement business logic in services
5. Add API routes in controllers

## Testing

```bash
cargo test
```
