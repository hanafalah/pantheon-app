# microtenant

Multi-tenant management library for Pantheon App.

## Features

### Tenant Resolver
Resolve tenant information from HTTP requests:
- From JWT token claims (auth context)
- From X-Tenant-ID header
- From subdomain (e.g., tenant123.app.com)

### Database Creator
Auto-create tenant databases:
- Create new tenant database (pantheon_tenant_{uuid})
- Check if database exists
- Create if not exists
- Drop database (with caution)
- List all tenant databases

### Tenant Migrator
Run migrations for tenant databases:
- Run migrations for single tenant
- Run migrations for all tenants
- Rollback migrations
- Check migration status

### Cluster Schema Manager
Manage cluster schemas for data segmentation:
- Create cluster schema (cashier_2024, scm_202403)
- Auto-create upcoming schemas (5 days before new period)
- Drop cluster schema
- List all cluster schemas

### Tenant Seeder
Seed default data for new tenants:
- Seed default roles (admin, manager, staff, user)
- Seed default permissions
- Seed default settings
- Seed from custom SQL file

## Usage

### Resolve Tenant from Request

```rust
use microtenant::TenantResolver;

// In your Actix-web handler
async fn my_handler(req: HttpRequest) -> Result<HttpResponse> {
    let tenant_id = TenantResolver::resolve_tenant_id(&req)?;
    let db_name = TenantResolver::resolve_tenant_database(&req)?;

    // Use tenant_id and db_name...
    Ok(HttpResponse::Ok().finish())
}
```

### Create Tenant Database

```rust
use microtenant::DatabaseCreator;

let creator = DatabaseCreator::new()?;

// Create new tenant database
let db_name = creator.create_tenant_database(tenant_id).await?;

// Or create if not exists
let db_name = creator.create_if_not_exists(tenant_id).await?;
```

### Run Migrations

```rust
use microtenant::TenantMigrator;
use std::path::Path;

let migrator = TenantMigrator::new()?;

// Run migrations for single tenant
migrator.run_migrations(tenant_id, Path::new("migrations/tenant")).await?;

// Run for all tenants
let tenant_ids = vec![tenant1_id, tenant2_id];
migrator.run_migrations_for_all(Path::new("migrations/tenant"), &tenant_ids).await?;
```

### Manage Cluster Schemas

```rust
use microtenant::ClusterSchemaManager;

let manager = ClusterSchemaManager::new()?;

// Create cluster schema
let schema = manager.create_cluster_schema(
    "pantheon_core",
    "cashier",
    2024,
    Some(12)
).await?; // Creates "cashier_202412"

// Auto-create upcoming schemas
let schemas = manager.auto_create_upcoming_schemas(
    "pantheon_core",
    &["cashier", "scm"]
).await?;
```

### Seed Tenant Data

```rust
use microtenant::TenantSeeder;

let seeder = TenantSeeder::new()?;

// Seed default data
seeder.seed_tenant(tenant_id).await?;

// Or seed from custom file
seeder.seed_from_file(tenant_id, "seeds/custom.sql").await?;
```

## Architecture

### Database Naming Convention

- **Tenant Database**: `pantheon_tenant_{uuid_without_hyphens}`
  - Example: `pantheon_tenant_550e8400e29b41d4a716446655440000`

- **Cluster Schema**: `{cluster_type}_{year}` or `{cluster_type}_{year}{month}`
  - Yearly: `cashier_2024`
  - Monthly: `scm_202403`

### Tenant Resolution Priority

1. JWT token claims (from AuthContext)
2. X-Tenant-ID header
3. Subdomain extraction

### Automatic Schema Creation

Cluster schemas are auto-created 5 days before:
- Start of new year (for yearly schemas)
- Start of new month (for monthly schemas)

This can be triggered by a scheduled job (RabbitMQ consumer).

## Testing

```bash
cargo test -p microtenant
```

## Dependencies

- rust-support - Core infrastructure
- rust-core - Helper functions
- actix-web - HTTP framework
- diesel - Database ORM
- r2d2 - Connection pooling
