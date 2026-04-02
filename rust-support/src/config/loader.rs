//! Config Loader
//! TODO: Full implementation

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn new() -> Self {
        Self
    }
    
    pub fn load_from_file(&self, _path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        unimplemented!("ConfigLoader::load_from_file")
    }
}
