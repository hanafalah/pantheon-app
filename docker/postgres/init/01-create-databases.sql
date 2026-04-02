-- Create Databases for Pantheon App
-- This script runs automatically when postgres container starts

-- Create core database
CREATE DATABASE pantheon;

-- Create tenant template database
CREATE DATABASE pantheon_tenant_template;

-- Create test tenant database
CREATE DATABASE pantheon_tenant_00000000_0000_0000_0000_000000000001;

COMMENT ON DATABASE pantheon IS 'Core database for Pantheon App';
COMMENT ON DATABASE pantheon_tenant_template IS 'Template database for tenant creation';
COMMENT ON DATABASE pantheon_tenant_00000000_0000_0000_0000_000000000001 IS 'Test tenant database';
