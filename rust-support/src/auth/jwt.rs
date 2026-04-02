//! JWT Manager
//! TODO: Full implementation

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub tenant_id: Option<Uuid>,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtManager;

impl JwtManager {
    pub fn generate_access_token(&self, _user_id: Uuid, _tenant_id: Option<Uuid>) -> Result<String, Box<dyn std::error::Error>> {
        unimplemented!("JwtManager::generate_access_token")
    }
    
    pub fn verify_token(&self, _token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        unimplemented!("JwtManager::verify_token")
    }
}
