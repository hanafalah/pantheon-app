# Pantheon App - Backend

Multi-tenant Business Management System built with Rust.

## Overview

Pantheon App is a comprehensive business management system designed to handle multiple tenants, each with their own isolated database. The system is built using Rust for high performance and type safety.

## Architecture

- **Multi-tenant**: Each tenant has its own database (`pantheon_tenant_{id}`)
- **Multi-database**: Core database + tenant databases + cluster databases
- **Monolith with Multi-repo**: Single deployment with modular repository structure
- **Service Provider Pattern**: Module initialization similar to Laravel
- **Configuration Hierarchy**: repositories → projects → groups → tenants

## Project Structure

```
pantheon-app/
├── Cargo.toml              # Workspace manifest
├── rust-support/           # Core support library (base traits, database manager, auth, etc.)
├── repositories/           # Reusable modules (module-user, module-item, etc.)
├── projects/              # Project-level applications (pantheon-business)
├── groups/                # Group-level customizations (created on demand)
├── tenants/               # Tenant-level customizations (created on demand)
├── config/                # Configuration files
├── migrations/            # Database migrations
└── docker/                # Docker configuration
```

## Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- PostgreSQL 15+
- RabbitMQ 3.12+
- Docker & Docker Compose (optional, for containerized setup)

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/hanafalah/pantheon-app.git
cd pantheon-app
```

### 2. Setup environment

```bash
cp .env.example .env
# Edit .env with your database credentials
```

### 3. Install dependencies

```bash
cargo build
```

### 4. Run migrations

```bash
diesel migration run
```

### 5. Start the server

```bash
cargo run --package pantheon-business
```

The server will start on `http://localhost:8000`.

## Development

### Running tests

```bash
cargo test
```

### Running with watch mode

```bash
cargo install cargo-watch
cargo watch -x run
```

### Code formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Docker

### Build and run with Docker Compose

```bash
cd docker
docker-compose up -d
```

This will start:
- PostgreSQL database
- RabbitMQ message broker
- Pantheon App backend

## API Documentation

API documentation is available via Swagger UI:
- Development: http://localhost:8000/swagger-ui/
- OpenAPI spec: http://localhost:8000/api/openapi.json

Full documentation app: [pantheon-docs](https://github.com/hanafalah/pantheon-docs)

## Related Repositories

- [pantheon-nuxt](https://github.com/hanafalah/pantheon-nuxt) - Frontend application
- [pantheon-docs](https://github.com/hanafalah/pantheon-docs) - Documentation

## License

MIT

## Authors

Pantheon Team <hamzahnafalah@gmail.com>
