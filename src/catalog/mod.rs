use crate::config::AppConfig;
use crate::error::InfsError;
use crate::providers::registry::ProviderRegistry;
use crate::types::{AppCategory, AppDescriptor};

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

    pub async fn list_all_apps(&self) -> Vec<AppDescriptor> {
        let mut apps = Vec::new();
        for provider in self.registry.list_providers() {
            let prov_config = self
                .app_config
                .providers
                .get(provider.descriptor().id.as_str())
                .cloned()
                .unwrap_or_default();
            match provider.list_apps(&prov_config).await {
                Ok(provider_apps) => apps.extend(provider_apps),
                Err(e) => {
                    tracing::warn!(
                        "Failed to list apps from {}: {}",
                        provider.descriptor().id,
                        e
                    );
                }
            }
        }
        apps
    }

    pub async fn list_apps_by_category(&self, category: &AppCategory) -> Vec<AppDescriptor> {
        self.list_all_apps()
            .await
            .into_iter()
            .filter(|app| &app.category == category)
            .collect()
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
        provider.list_apps(&prov_config).await
    }

    pub async fn find_app(&self, provider_id: &str, app_id: &str) -> Option<AppDescriptor> {
        self.list_apps_by_provider(provider_id)
            .await
            .ok()?
            .into_iter()
            .find(|app| app.id == app_id)
    }
}
