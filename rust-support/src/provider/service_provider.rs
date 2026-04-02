//! Service Provider Pattern
//! TODO: Full implementation

use async_trait::async_trait;

#[async_trait]
pub trait ServiceProvider: Send + Sync {
    async fn register(&self) -> Result<(), Box<dyn std::error::Error>>;
    async fn boot(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn provides(&self) -> Vec<String>;
}

pub struct ServiceProviderRegistry {
    providers: Vec<Box<dyn ServiceProvider>>,
}

impl ServiceProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
    
    pub fn add(&mut self, _provider: Box<dyn ServiceProvider>) {
        unimplemented!("ServiceProviderRegistry::add")
    }
}
