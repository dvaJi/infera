use std::collections::HashMap;
use crate::error::InfsError;
use super::{Provider, openrouter::OpenRouterProvider, falai::FalAiProvider, replicate::ReplicateProvider, wavespeed::WavespeedProvider};

pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        ProviderRegistry {
            providers: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, provider: Box<dyn Provider>) {
        let id = provider.descriptor().id.clone();
        self.providers.insert(id, provider);
    }
    
    pub fn find_provider(&self, id: &str) -> Result<&dyn Provider, InfsError> {
        self.providers.get(id)
            .map(|p| p.as_ref())
            .ok_or_else(|| InfsError::ProviderNotFound(id.to_string()))
    }
    
    pub fn list_providers(&self) -> Vec<&dyn Provider> {
        let mut providers: Vec<&dyn Provider> = self.providers.values().map(|p| p.as_ref()).collect();
        providers.sort_by(|a, b| a.descriptor().id.cmp(&b.descriptor().id));
        providers
    }
}

pub fn build_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(OpenRouterProvider::new()));
    registry.register(Box::new(FalAiProvider::new()));
    registry.register(Box::new(ReplicateProvider::new()));
    registry.register(Box::new(WavespeedProvider::new()));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_find_provider() {
        let registry = build_registry();
        let provider = registry.find_provider("openrouter").unwrap();
        assert_eq!(provider.descriptor().id, "openrouter");
    }
    
    #[test]
    fn test_registry_find_provider_not_found() {
        let registry = build_registry();
        assert!(registry.find_provider("nonexistent").is_err());
    }
    
    #[test]
    fn test_registry_list_providers() {
        let registry = build_registry();
        let providers = registry.list_providers();
        assert!(providers.len() >= 4);
        
        let ids: Vec<&str> = providers.iter().map(|p| p.descriptor().id.as_str()).collect();
        assert!(ids.contains(&"openrouter"));
        assert!(ids.contains(&"falai"));
        assert!(ids.contains(&"replicate"));
        assert!(ids.contains(&"wavespeed"));
    }
}
