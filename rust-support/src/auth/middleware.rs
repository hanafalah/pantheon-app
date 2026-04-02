//! Auth Middleware
//!
//! Actix-web middleware for JWT authentication and tenant resolution

use crate::auth::jwt::{Claims, JwtManager};
use crate::utils::AppError;
use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::AUTHORIZATION,
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};
use uuid::Uuid;

/// Auth Context
/// Stored in request extensions after successful authentication
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub claims: Claims,
}

impl AuthContext {
    pub fn new(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            tenant_id: claims.tenant_id,
            claims,
        }
    }
}

/// Auth Middleware
pub struct AuthMiddleware {
    jwt_manager: Rc<JwtManager>,
}

impl AuthMiddleware {
    /// Create new auth middleware
    pub fn new() -> Self {
        Self {
            jwt_manager: Rc::new(JwtManager::new()),
        }
    }

    /// Create auth middleware with custom JWT manager
    pub fn with_jwt_manager(jwt_manager: JwtManager) -> Self {
        Self {
            jwt_manager: Rc::new(jwt_manager),
        }
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            jwt_manager: self.jwt_manager.clone(),
        }))
    }
}

/// Auth Middleware Service
pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    jwt_manager: Rc<JwtManager>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let jwt_manager = self.jwt_manager.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let token = match extract_token_from_header(&req) {
                Ok(token) => token,
                Err(e) => {
                    let error_response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": {
                            "type": "unauthorized",
                            "message": e.to_string()
                        }
                    }));
                    return Ok(req.into_response(error_response.map_into_boxed_body()));
                }
            };

            // Verify token
            let claims = match jwt_manager.validate_access_token(&token) {
                Ok(claims) => claims,
                Err(e) => {
                    let error_response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": {
                            "type": "unauthorized",
                            "message": e.to_string()
                        }
                    }));
                    return Ok(req.into_response(error_response.map_into_boxed_body()));
                }
            };

            // Store auth context in request extensions
            let auth_context = AuthContext::new(claims);
            req.extensions_mut().insert(auth_context);

            // Continue to next middleware/handler
            let res = service.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

/// Extract token from Authorization header
fn extract_token_from_header(req: &ServiceRequest) -> Result<String, AppError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or_else(|| AppError::AuthError("Missing Authorization header".to_string()))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AppError::AuthError("Invalid Authorization header format".to_string()))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AppError::AuthError(
            "Invalid Authorization header format. Expected: Bearer <token>".to_string(),
        ));
    }

    let token = auth_str.trim_start_matches("Bearer ").to_string();

    if token.is_empty() {
        return Err(AppError::AuthError("Empty token".to_string()));
    }

    Ok(token)
}

/// Helper function to get auth context from request
pub fn get_auth_context(req: &ServiceRequest) -> Result<AuthContext, AppError> {
    req.extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or_else(|| AppError::AuthError("Authentication required".to_string()))
}

/// Optional Auth Middleware (doesn't fail if token is missing)
pub struct OptionalAuthMiddleware {
    jwt_manager: Rc<JwtManager>,
}

impl OptionalAuthMiddleware {
    pub fn new() -> Self {
        Self {
            jwt_manager: Rc::new(JwtManager::new()),
        }
    }
}

impl Default for OptionalAuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for OptionalAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = OptionalAuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(OptionalAuthMiddlewareService {
            service: Rc::new(service),
            jwt_manager: self.jwt_manager.clone(),
        }))
    }
}

pub struct OptionalAuthMiddlewareService<S> {
    service: Rc<S>,
    jwt_manager: Rc<JwtManager>,
}

impl<S, B> Service<ServiceRequest> for OptionalAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let jwt_manager = self.jwt_manager.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Try to extract and verify token, but don't fail if missing
            if let Ok(token) = extract_token_from_header(&req) {
                if let Ok(claims) = jwt_manager.validate_access_token(&token) {
                    let auth_context = AuthContext::new(claims);
                    req.extensions_mut().insert(auth_context);
                }
            }

            // Continue regardless of auth status
            let res = service.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[test]
    fn test_extract_token_from_header_valid() {
        let req = test::TestRequest::default()
            .insert_header(("Authorization", "Bearer valid_token_here"))
            .to_srv_request();

        let result = extract_token_from_header(&req);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "valid_token_here");
    }

    #[test]
    fn test_extract_token_from_header_missing() {
        let req = test::TestRequest::default().to_srv_request();

        let result = extract_token_from_header(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_from_header_invalid_format() {
        let req = test::TestRequest::default()
            .insert_header(("Authorization", "InvalidFormat"))
            .to_srv_request();

        let result = extract_token_from_header(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_from_header_empty_token() {
        let req = test::TestRequest::default()
            .insert_header(("Authorization", "Bearer "))
            .to_srv_request();

        let result = extract_token_from_header(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_context_creation() {
        let user_id = Uuid::new_v4();
        let tenant_id = Some(Uuid::new_v4());
        let claims = Claims::new_access_token(user_id, tenant_id, 900);

        let context = AuthContext::new(claims.clone());
        assert_eq!(context.user_id, user_id);
        assert_eq!(context.tenant_id, tenant_id);
        assert_eq!(context.claims.sub, user_id);
    }
}
