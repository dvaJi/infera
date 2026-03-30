use crate::config::AppConfig;
use crate::error::InfsError;
use crate::providers::registry::ProviderRegistry;
use crate::types::{AppDescriptor, ListOptions};

pub struct Catalog<'a> {
    registry: &'a ProviderRegistry,
    app_config: &'a AppConfig,
}

impl<'a> Catalog<'a> {
    pub fn new(registry: &'a ProviderRegistry, app_config: &'a AppConfig) -> Self {
        Catalog {
            registry,
            app_config,
        }
    }

    pub async fn list_apps_by_provider(
        &self,
        provider_id: &str,
    ) -> Result<Vec<AppDescriptor>, InfsError> {
        let provider = self.registry.find_provider(provider_id)?;
        let prov_config = self
            .app_config
            .providers
            .get(provider_id)
            .cloned()
            .unwrap_or_default();
        let options = ListOptions::default();
        provider.list_apps(&prov_config, &options).await
    }

    pub async fn find_app(&self, provider_id: &str, app_id: &str) -> Option<AppDescriptor> {
        self.list_apps_by_provider(provider_id)
            .await
            .ok()?
            .into_iter()
            .find(|app| app.id == app_id)
    }
}
