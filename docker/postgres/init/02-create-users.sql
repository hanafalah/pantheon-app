-- Create Users and Grant Permissions

-- Create admin user (for migrations and admin operations)
CREATE USER pantheon_admin WITH PASSWORD 'pantheon_admin_password';

-- Create app user (for application)
CREATE USER pantheon_app WITH PASSWORD 'pantheon_password';

-- Grant privileges to admin user
GRANT ALL PRIVILEGES ON DATABASE pantheon TO pantheon_admin;
GRANT ALL PRIVILEGES ON DATABASE pantheon_tenant_template TO pantheon_admin;
GRANT ALL PRIVILEGES ON DATABASE pantheon_tenant_00000000_0000_0000_0000_000000000001 TO pantheon_admin;

-- Grant privileges to app user
GRANT CONNECT ON DATABASE pantheon TO pantheon_app;
GRANT CONNECT ON DATABASE pantheon_tenant_template TO pantheon_app;
GRANT CONNECT ON DATABASE pantheon_tenant_00000000_0000_0000_0000_000000000001 TO pantheon_app;

-- Connect to each database and grant schema permissions
\c pantheon
GRANT ALL ON SCHEMA public TO pantheon_admin;
GRANT USAGE ON SCHEMA public TO pantheon_app;
GRANT CREATE ON SCHEMA public TO pantheon_app;
ALTER DEFAULT PRIVILEGES FOR USER pantheon_admin IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO pantheon_app;
ALTER DEFAULT PRIVILEGES FOR USER pantheon_admin IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO pantheon_app;

\c pantheon_tenant_template
GRANT ALL ON SCHEMA public TO pantheon_admin;
GRANT USAGE ON SCHEMA public TO pantheon_app;
GRANT CREATE ON SCHEMA public TO pantheon_app;
ALTER DEFAULT PRIVILEGES FOR USER pantheon_admin IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO pantheon_app;
ALTER DEFAULT PRIVILEGES FOR USER pantheon_admin IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO pantheon_app;

\c pantheon_tenant_00000000_0000_0000_0000_000000000001
GRANT ALL ON SCHEMA public TO pantheon_admin;
GRANT USAGE ON SCHEMA public TO pantheon_app;
GRANT CREATE ON SCHEMA public TO pantheon_app;
ALTER DEFAULT PRIVILEGES FOR USER pantheon_admin IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO pantheon_app;
ALTER DEFAULT PRIVILEGES FOR USER pantheon_admin IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO pantheon_app;
