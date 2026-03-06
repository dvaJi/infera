use crate::providers::registry::ProviderRegistry;
use crate::types::{AppCategory, AppDescriptor};

pub struct Catalog<'a> {
    registry: &'a ProviderRegistry,
}

impl<'a> Catalog<'a> {
    pub fn new(registry: &'a ProviderRegistry) -> Self {
        Catalog { registry }
    }
    
    pub fn list_all_apps(&self) -> Vec<AppDescriptor> {
        let mut apps = Vec::new();
        for provider in self.registry.list_providers() {
            apps.extend(provider.list_apps());
        }
        apps
    }
    
    pub fn list_apps_by_category(&self, category: &AppCategory) -> Vec<AppDescriptor> {
        self.list_all_apps()
            .into_iter()
            .filter(|app| &app.category == category)
            .collect()
    }
    
    pub fn list_apps_by_provider(&self, provider_id: &str) -> Vec<AppDescriptor> {
        self.list_all_apps()
            .into_iter()
            .filter(|app| app.provider_id == provider_id)
            .collect()
    }
    
    pub fn find_app(&self, provider_id: &str, app_id: &str) -> Option<AppDescriptor> {
        self.list_apps_by_provider(provider_id)
            .into_iter()
            .find(|app| app.id == app_id)
    }
}
