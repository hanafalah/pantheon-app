//! Tenant Resolver
//!
//! Resolve tenant information from HTTP requests

use actix_web::{HttpRequest, HttpMessage};
use rust_support::utils::{AppError, AppResult};
use uuid::Uuid;

/// Tenant Resolver
///
/// Resolves tenant information from incoming HTTP requests
pub struct TenantResolver;

impl TenantResolver {
    /// Resolve tenant ID from request
    ///
    /// Attempts to resolve tenant ID from:
    /// 1. JWT token claims (if authenticated)
    /// 2. X-Tenant-ID header
    /// 3. Subdomain (e.g., tenant123.app.com)
    pub fn resolve_tenant_id(req: &HttpRequest) -> AppResult<Uuid> {
        // 1. Try from JWT token (auth context)
        if let Some(tenant_id) = Self::from_auth_context(req) {
            return Ok(tenant_id);
        }

        // 2. Try from header
        if let Some(tenant_id) = Self::from_header(req)? {
            return Ok(tenant_id);
        }

        // 3. Try from subdomain
        if let Some(tenant_id) = Self::from_subdomain(req)? {
            return Ok(tenant_id);
        }

        Err(AppError::AuthorizationError(
            "Tenant ID not found in request".to_string(),
        ))
    }

    /// Get tenant ID from auth context (JWT claims)
    fn from_auth_context(req: &HttpRequest) -> Option<Uuid> {
        // This would be populated by auth middleware
        req.extensions()
            .get::<rust_support::auth::AuthContext>()
            .and_then(|ctx| ctx.tenant_id)
    }

    /// Get tenant ID from X-Tenant-ID header
    fn from_header(req: &HttpRequest) -> AppResult<Option<Uuid>> {
        if let Some(header_value) = req.headers().get("X-Tenant-ID") {
            let tenant_id_str = header_value
                .to_str()
                .map_err(|_| AppError::ValidationError(vec![]))?;

            let tenant_id = Uuid::parse_str(tenant_id_str)
                .map_err(|_| AppError::ValidationError(vec![]))?;

            return Ok(Some(tenant_id));
        }

        Ok(None)
    }

    /// Get tenant ID from subdomain
    fn from_subdomain(req: &HttpRequest) -> AppResult<Option<Uuid>> {
        if let Some(host) = req.headers().get("Host") {
            let host_str = host
                .to_str()
                .map_err(|_| AppError::ValidationError(vec![]))?;

            // Extract subdomain (e.g., "tenant123" from "tenant123.app.com")
            if let Some(subdomain) = host_str.split('.').next() {
                // Try to parse as UUID
                if let Ok(tenant_id) = Uuid::parse_str(subdomain) {
                    return Ok(Some(tenant_id));
                }

                // Or lookup subdomain in database (implementation would go here)
                // For now, return None
            }
        }

        Ok(None)
    }

    /// Resolve tenant database name from request
    pub fn resolve_tenant_database(req: &HttpRequest) -> AppResult<String> {
        let tenant_id = Self::resolve_tenant_id(req)?;
        let db_name = format!("pantheon_tenant_{}", tenant_id.to_string().replace('-', ""));
        Ok(db_name)
    }

    /// Check if request is for a specific tenant
    pub fn is_tenant_request(req: &HttpRequest) -> bool {
        Self::resolve_tenant_id(req).is_ok()
    }

    /// Get tenant ID or return error
    pub fn require_tenant_id(req: &HttpRequest) -> AppResult<Uuid> {
        Self::resolve_tenant_id(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[test]
    fn test_resolve_from_header() {
        let tenant_id = Uuid::new_v4();
        let req = test::TestRequest::default()
            .insert_header(("X-Tenant-ID", tenant_id.to_string()))
            .to_http_request();

        let resolved = TenantResolver::resolve_tenant_id(&req).unwrap();
        assert_eq!(resolved, tenant_id);
    }

    #[test]
    fn test_resolve_tenant_database() {
        let tenant_id = Uuid::new_v4();
        let req = test::TestRequest::default()
            .insert_header(("X-Tenant-ID", tenant_id.to_string()))
            .to_http_request();

        let db_name = TenantResolver::resolve_tenant_database(&req).unwrap();
        assert!(db_name.starts_with("pantheon_tenant_"));
    }

    #[test]
    fn test_is_tenant_request() {
        let tenant_id = Uuid::new_v4();
        let req = test::TestRequest::default()
            .insert_header(("X-Tenant-ID", tenant_id.to_string()))
            .to_http_request();

        assert!(TenantResolver::is_tenant_request(&req));
    }

    #[test]
    fn test_no_tenant() {
        let req = test::TestRequest::default().to_http_request();

        let result = TenantResolver::resolve_tenant_id(&req);
        assert!(result.is_err());
    }
}
