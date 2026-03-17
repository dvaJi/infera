use crate::error::InfsError;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    pub auth_method: Option<String>,
    #[serde(default)]
    pub credentials: HashMap<String, String>,
    #[serde(default)]
    pub connected: bool,
}

impl ProviderConfig {
    pub fn get_api_key(&self) -> Option<&str> {
        self.credentials.get("api_key").map(|s| s.as_str())
    }
}

pub fn get_config_dir() -> Result<PathBuf, InfsError> {
    let project_dirs = ProjectDirs::from("ai", "infs", "infs").ok_or_else(|| {
        InfsError::ConfigError("Could not determine config directory".to_string())
    })?;
    Ok(project_dirs.config_dir().to_path_buf())
}

pub fn get_config_path() -> Result<PathBuf, InfsError> {
    Ok(get_config_dir()?.join("config.toml"))
}

pub fn get_credentials_path() -> Result<PathBuf, InfsError> {
    Ok(get_config_dir()?.join("credentials.toml"))
}

pub fn load_config() -> Result<AppConfig, InfsError> {
    let config_path = get_config_path()?;

    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| InfsError::ConfigError(format!("Failed to read config: {}", e)))?;
        toml::from_str(&content)
            .map_err(|e| InfsError::ConfigError(format!("Failed to parse config: {}", e)))?
    } else {
        AppConfig::default()
    };

    // Merge credentials from separate file (even if config.toml was absent)
    let creds_path = get_credentials_path()?;
    if creds_path.exists() {
        let creds_content = std::fs::read_to_string(&creds_path)
            .map_err(|e| InfsError::ConfigError(format!("Failed to read credentials: {}", e)))?;

        let creds: HashMap<String, ProviderConfig> = toml::from_str(&creds_content)
            .map_err(|e| InfsError::ConfigError(format!("Failed to parse credentials: {}", e)))?;

        for (provider_id, cred_config) in creds {
            let provider_config = config.providers.entry(provider_id).or_default();
            for (key, value) in cred_config.credentials {
                provider_config.credentials.insert(key, value);
            }
        }
    }

    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<(), InfsError> {
    let config_dir = get_config_dir()?;
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| InfsError::ConfigError(format!("Failed to create config dir: {}", e)))?;

    // Save main config (without credentials)
    let mut config_without_creds = config.clone();
    for provider in config_without_creds.providers.values_mut() {
        provider.credentials.clear();
    }

    let config_content = toml::to_string_pretty(&config_without_creds)
        .map_err(|e| InfsError::ConfigError(format!("Failed to serialize config: {}", e)))?;

    std::fs::write(get_config_path()?, config_content)
        .map_err(|e| InfsError::ConfigError(format!("Failed to write config: {}", e)))?;

    // Save credentials separately
    let mut creds: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (provider_id, provider_config) in &config.providers {
        if !provider_config.credentials.is_empty() {
            creds.insert(provider_id.clone(), provider_config.credentials.clone());
        }
    }

    // Build a ProviderConfig-like structure for TOML
    #[derive(Serialize)]
    struct CredStore {
        credentials: HashMap<String, String>,
    }

    let cred_store: HashMap<String, CredStore> = creds
        .into_iter()
        .map(|(k, v)| (k, CredStore { credentials: v }))
        .collect();

    let creds_content = toml::to_string_pretty(&cred_store)
        .map_err(|e| InfsError::ConfigError(format!("Failed to serialize credentials: {}", e)))?;

    write_credentials_file(&get_credentials_path()?, &creds_content)?;

    Ok(())
}

/// Write a file containing sensitive data (API keys) with restrictive permissions (0600 on Unix).
fn write_credentials_file(path: &std::path::Path, content: &str) -> Result<(), InfsError> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| {
                InfsError::ConfigError(format!("Failed to open credentials file: {}", e))
            })?;
        file.write_all(content.as_bytes())
            .map_err(|e| InfsError::ConfigError(format!("Failed to write credentials: {}", e)))?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, content)
            .map_err(|e| InfsError::ConfigError(format!("Failed to write credentials: {}", e)))
    }
}

pub fn save_provider_credentials(
    provider_id: &str,
    credentials: HashMap<String, String>,
) -> Result<(), InfsError> {
    let mut config = load_config()?;
    let provider_config = config.providers.entry(provider_id.to_string()).or_default();
    provider_config.credentials = credentials;
    provider_config.connected = true;
    save_config(&config)
}

pub fn remove_provider_credentials(provider_id: &str) -> Result<(), InfsError> {
    let mut config = load_config()?;
    if let Some(provider_config) = config.providers.get_mut(provider_id) {
        provider_config.credentials.clear();
        provider_config.connected = false;
    }
    save_config(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert!(config.providers.is_empty());
    }

    #[test]
    fn test_provider_config_get_api_key() {
        let mut config = ProviderConfig::default();
        assert!(config.get_api_key().is_none());

        config
            .credentials
            .insert("api_key".to_string(), "test-key".to_string());
        assert_eq!(config.get_api_key(), Some("test-key"));
    }

    #[test]
    fn test_config_roundtrip_toml() {
        let mut config = AppConfig::default();
        let mut prov = ProviderConfig::default();
        prov.connected = true;
        prov.auth_method = Some("api_key".to_string());
        config.providers.insert("openrouter".to_string(), prov);

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();

        assert!(deserialized.providers.contains_key("openrouter"));
        assert!(deserialized.providers["openrouter"].connected);
    }
}
