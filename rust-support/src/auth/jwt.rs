//! JWT Manager
//!
//! Handles JWT token generation and verification with HS256 algorithm

use crate::utils::{AppError, AppResult};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// Tenant ID (optional, for multi-tenant support)
    pub tenant_id: Option<Uuid>,
    /// Expiration time (as UTC timestamp)
    pub exp: i64,
    /// Issued at (as UTC timestamp)
    pub iat: i64,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Token type: "access" or "refresh"
    pub token_type: String,
}

impl Claims {
    /// Create new claims for access token
    pub fn new_access_token(user_id: Uuid, tenant_id: Option<Uuid>, expires_in: i64) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            tenant_id,
            exp: (now + Duration::seconds(expires_in)).timestamp(),
            iat: now.timestamp(),
            iss: "pantheon-app".to_string(),
            aud: "pantheon-users".to_string(),
            token_type: "access".to_string(),
        }
    }

    /// Create new claims for refresh token
    pub fn new_refresh_token(user_id: Uuid, expires_in: i64) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            tenant_id: None,
            exp: (now + Duration::seconds(expires_in)).timestamp(),
            iat: now.timestamp(),
            iss: "pantheon-app".to_string(),
            aud: "pantheon-users".to_string(),
            token_type: "refresh".to_string(),
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Check if this is an access token
    pub fn is_access_token(&self) -> bool {
        self.token_type == "access"
    }

    /// Check if this is a refresh token
    pub fn is_refresh_token(&self) -> bool {
        self.token_type == "refresh"
    }

    /// Get time until expiration in seconds
    pub fn time_until_expiration(&self) -> i64 {
        self.exp - Utc::now().timestamp()
    }
}

/// JWT Manager Configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for signing tokens
    pub secret: String,
    /// Access token expiry in seconds (default: 900 = 15 minutes)
    pub access_token_expiry: i64,
    /// Refresh token expiry in seconds (default: 604800 = 7 days)
    pub refresh_token_expiry: i64,
    /// Issuer
    pub issuer: String,
    /// Audience
    pub audience: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "YXYlGIbJ65VGjQnETWX23iCvssXg7PJu".to_string(), // From requirements
            access_token_expiry: 900,    // 15 minutes
            refresh_token_expiry: 604800, // 7 days
            issuer: "pantheon-app".to_string(),
            audience: "pantheon-users".to_string(),
        }
    }
}

/// JWT Manager
pub struct JwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtManager {
    /// Create new JWT manager with default config
    pub fn new() -> Self {
        Self::with_config(JwtConfig::default())
    }

    /// Create new JWT manager with custom config
    pub fn with_config(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        Self {
            config,
            encoding_key,
            decoding_key,
        }
    }

    /// Create JWT manager from environment
    pub fn from_env() -> AppResult<Self> {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "YXYlGIbJ65VGjQnETWX23iCvssXg7PJu".to_string());

        let access_token_expiry = std::env::var("JWT_ACCESS_TOKEN_EXPIRY")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(900);

        let refresh_token_expiry = std::env::var("JWT_REFRESH_TOKEN_EXPIRY")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(604800);

        let config = JwtConfig {
            secret,
            access_token_expiry,
            refresh_token_expiry,
            ..Default::default()
        };

        Ok(Self::with_config(config))
    }

    /// Generate access token
    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        tenant_id: Option<Uuid>,
    ) -> AppResult<String> {
        let claims = Claims::new_access_token(user_id, tenant_id, self.config.access_token_expiry);
        self.encode_token(&claims)
    }

    /// Generate refresh token
    pub fn generate_refresh_token(&self, user_id: Uuid) -> AppResult<String> {
        let claims = Claims::new_refresh_token(user_id, self.config.refresh_token_expiry);
        self.encode_token(&claims)
    }

    /// Encode claims into JWT token
    fn encode_token(&self, claims: &Claims) -> AppResult<String> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| AppError::AuthError(format!("Failed to encode token: {}", e)))
    }

    /// Verify and decode token
    pub fn verify_token(&self, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::AuthError("Token has expired".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    AppError::AuthError("Invalid token signature".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    AppError::AuthError("Invalid token format".to_string())
                }
                _ => AppError::AuthError(format!("Token verification failed: {}", e)),
            })?;

        Ok(token_data.claims)
    }

    /// Decode token without verification (useful for reading expired tokens)
    pub fn decode_token_unsafe(&self, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::AuthError(format!("Failed to decode token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Extract tenant ID from token
    pub fn extract_tenant_id(&self, token: &str) -> AppResult<Option<Uuid>> {
        let claims = self.verify_token(token)?;
        Ok(claims.tenant_id)
    }

    /// Extract user ID from token
    pub fn extract_user_id(&self, token: &str) -> AppResult<Uuid> {
        let claims = self.verify_token(token)?;
        Ok(claims.sub)
    }

    /// Validate access token
    pub fn validate_access_token(&self, token: &str) -> AppResult<Claims> {
        let claims = self.verify_token(token)?;

        if !claims.is_access_token() {
            return Err(AppError::AuthError(
                "Token is not an access token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Validate refresh token
    pub fn validate_refresh_token(&self, token: &str) -> AppResult<Claims> {
        let claims = self.verify_token(token)?;

        if !claims.is_refresh_token() {
            return Err(AppError::AuthError(
                "Token is not a refresh token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Get access token expiry duration
    pub fn access_token_expiry(&self) -> i64 {
        self.config.access_token_expiry
    }

    /// Get refresh token expiry duration
    pub fn refresh_token_expiry(&self) -> i64 {
        self.config.refresh_token_expiry
    }
}

impl Default for JwtManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let user_id = Uuid::new_v4();
        let tenant_id = Some(Uuid::new_v4());
        let claims = Claims::new_access_token(user_id, tenant_id, 900);

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.tenant_id, tenant_id);
        assert_eq!(claims.token_type, "access");
        assert!(!claims.is_expired());
        assert!(claims.is_access_token());
        assert!(!claims.is_refresh_token());
    }

    #[test]
    fn test_jwt_manager_generate_tokens() {
        let manager = JwtManager::new();
        let user_id = Uuid::new_v4();
        let tenant_id = Some(Uuid::new_v4());

        // Test access token generation
        let access_token = manager.generate_access_token(user_id, tenant_id);
        assert!(access_token.is_ok());

        // Test refresh token generation
        let refresh_token = manager.generate_refresh_token(user_id);
        assert!(refresh_token.is_ok());
    }

    #[test]
    fn test_jwt_manager_verify_token() {
        let manager = JwtManager::new();
        let user_id = Uuid::new_v4();
        let tenant_id = Some(Uuid::new_v4());

        let token = manager
            .generate_access_token(user_id, tenant_id)
            .unwrap();
        let claims = manager.verify_token(&token);

        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.tenant_id, tenant_id);
    }

    #[test]
    fn test_jwt_manager_extract_tenant_id() {
        let manager = JwtManager::new();
        let user_id = Uuid::new_v4();
        let tenant_id = Some(Uuid::new_v4());

        let token = manager
            .generate_access_token(user_id, tenant_id)
            .unwrap();
        let extracted_tenant_id = manager.extract_tenant_id(&token);

        assert!(extracted_tenant_id.is_ok());
        assert_eq!(extracted_tenant_id.unwrap(), tenant_id);
    }

    #[test]
    fn test_jwt_manager_extract_user_id() {
        let manager = JwtManager::new();
        let user_id = Uuid::new_v4();

        let token = manager.generate_access_token(user_id, None).unwrap();
        let extracted_user_id = manager.extract_user_id(&token);

        assert!(extracted_user_id.is_ok());
        assert_eq!(extracted_user_id.unwrap(), user_id);
    }

    #[test]
    fn test_jwt_manager_validate_access_token() {
        let manager = JwtManager::new();
        let user_id = Uuid::new_v4();

        let access_token = manager.generate_access_token(user_id, None).unwrap();
        let result = manager.validate_access_token(&access_token);
        assert!(result.is_ok());

        let refresh_token = manager.generate_refresh_token(user_id).unwrap();
        let result = manager.validate_access_token(&refresh_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_manager_validate_refresh_token() {
        let manager = JwtManager::new();
        let user_id = Uuid::new_v4();

        let refresh_token = manager.generate_refresh_token(user_id).unwrap();
        let result = manager.validate_refresh_token(&refresh_token);
        assert!(result.is_ok());

        let access_token = manager.generate_access_token(user_id, None).unwrap();
        let result = manager.validate_refresh_token(&access_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_token() {
        let manager = JwtManager::new();
        let result = manager.verify_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_with_wrong_signature() {
        let manager1 = JwtManager::with_config(JwtConfig {
            secret: "secret1".to_string(),
            ..Default::default()
        });
        let manager2 = JwtManager::with_config(JwtConfig {
            secret: "secret2".to_string(),
            ..Default::default()
        });

        let user_id = Uuid::new_v4();
        let token = manager1.generate_access_token(user_id, None).unwrap();

        // Try to verify with different secret
        let result = manager2.verify_token(&token);
        assert!(result.is_err());
    }
}
