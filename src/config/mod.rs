use crate::error::InfsError;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Service name used for all keyring entries.
const KEYRING_SERVICE: &str = "infs";

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
    /// Names of credential keys whose values are stored in the OS keychain.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keychain_credentials: Vec<String>,
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

// ---------------------------------------------------------------------------
// Keyring helpers
// ---------------------------------------------------------------------------

/// Build the keyring username string for a given provider + credential key.
fn keyring_username(provider_id: &str, cred_key: &str) -> String {
    format!("{}/{}", provider_id, cred_key)
}

/// Store a single credential value in the OS keychain.
/// Returns `Ok(true)` on success, `Ok(false)` when the keychain is unavailable.
pub fn keyring_set(provider_id: &str, cred_key: &str, value: &str) -> Result<bool, InfsError> {
    let username = keyring_username(provider_id, cred_key);
    match keyring::Entry::new(KEYRING_SERVICE, &username) {
        Ok(entry) => match entry.set_password(value) {
            Ok(()) => Ok(true),
            Err(keyring::Error::NoStorageAccess(_)) | Err(keyring::Error::PlatformFailure(_)) => {
                Ok(false)
            }
            Err(e) => Err(InfsError::ConfigError(format!(
                "Keychain write failed for {}/{}: {}",
                provider_id, cred_key, e
            ))),
        },
        Err(keyring::Error::NoStorageAccess(_)) | Err(keyring::Error::PlatformFailure(_)) => {
            Ok(false)
        }
        Err(e) => Err(InfsError::ConfigError(format!(
            "Keychain entry creation failed for {}/{}: {}",
            provider_id, cred_key, e
        ))),
    }
}

/// Retrieve a single credential value from the OS keychain.
/// Returns `Ok(Some(value))` on success, `Ok(None)` when the entry or keychain is unavailable.
pub fn keyring_get(provider_id: &str, cred_key: &str) -> Result<Option<String>, InfsError> {
    let username = keyring_username(provider_id, cred_key);
    match keyring::Entry::new(KEYRING_SERVICE, &username) {
        Ok(entry) => match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(keyring::Error::NoStorageAccess(_)) | Err(keyring::Error::PlatformFailure(_)) => {
                Ok(None)
            }
            Err(e) => Err(InfsError::ConfigError(format!(
                "Keychain read failed for {}/{}: {}",
                provider_id, cred_key, e
            ))),
        },
        Err(keyring::Error::NoStorageAccess(_)) | Err(keyring::Error::PlatformFailure(_)) => {
            Ok(None)
        }
        Err(e) => Err(InfsError::ConfigError(format!(
            "Keychain entry creation failed for {}/{}: {}",
            provider_id, cred_key, e
        ))),
    }
}

/// Delete a single credential from the OS keychain.
/// Silently succeeds when the entry or keychain is unavailable.
pub fn keyring_delete(provider_id: &str, cred_key: &str) -> Result<(), InfsError> {
    let username = keyring_username(provider_id, cred_key);
    match keyring::Entry::new(KEYRING_SERVICE, &username) {
        Ok(entry) => match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(keyring::Error::NoStorageAccess(_)) | Err(keyring::Error::PlatformFailure(_)) => {
                Ok(())
            }
            Err(e) => Err(InfsError::ConfigError(format!(
                "Keychain delete failed for {}/{}: {}",
                provider_id, cred_key, e
            ))),
        },
        Err(keyring::Error::NoStorageAccess(_)) | Err(keyring::Error::PlatformFailure(_)) => Ok(()),
        Err(e) => Err(InfsError::ConfigError(format!(
            "Keychain entry creation failed for {}/{}: {}",
            provider_id, cred_key, e
        ))),
    }
}

// ---------------------------------------------------------------------------
// Config load / save
// ---------------------------------------------------------------------------

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

    // Load credentials: keychain first (for keys recorded in keychain_credentials),
    // then fall back to credentials.toml for anything not yet migrated.
    for (provider_id, provider_config) in config.providers.iter_mut() {
        for cred_key in &provider_config.keychain_credentials {
            if let Some(value) = keyring_get(provider_id, cred_key)? {
                provider_config.credentials.insert(cred_key.clone(), value);
            }
        }
    }

    // Merge credentials from separate file (even if config.toml was absent or for
    // providers whose keys are not yet in keychain_credentials).
    let creds_path = get_credentials_path()?;
    if creds_path.exists() {
        let creds_content = std::fs::read_to_string(&creds_path)
            .map_err(|e| InfsError::ConfigError(format!("Failed to read credentials: {}", e)))?;

        let creds: HashMap<String, ProviderConfig> = toml::from_str(&creds_content)
            .map_err(|e| InfsError::ConfigError(format!("Failed to parse credentials: {}", e)))?;

        for (provider_id, cred_config) in creds {
            let provider_config = config.providers.entry(provider_id).or_default();
            for (key, value) in cred_config.credentials {
                // Don't overwrite a value already loaded from keychain.
                provider_config.credentials.entry(key).or_insert(value);
            }
        }
    }

    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<(), InfsError> {
    let config_dir = get_config_dir()?;
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| InfsError::ConfigError(format!("Failed to create config dir: {}", e)))?;

    // Determine which credentials go to the keychain vs the fallback file.
    let mut file_creds: HashMap<String, HashMap<String, String>> = HashMap::new();
    // Collect keychain_credentials per provider so config_without_creds can be
    // built correctly in a single pass before any file write.
    let mut keychain_keys_per_provider: HashMap<String, Vec<String>> = HashMap::new();

    for (provider_id, provider_config) in &config.providers {
        // Delete keychain entries for keys that are no longer in credentials
        // (e.g. after a key rotation or partial credential removal).
        let current_cred_keys: std::collections::HashSet<&str> = provider_config
            .credentials
            .keys()
            .map(|s| s.as_str())
            .collect();
        for old_key in &provider_config.keychain_credentials {
            if !current_cred_keys.contains(old_key.as_str()) {
                keyring_delete(provider_id, old_key)?;
            }
        }

        if provider_config.credentials.is_empty() {
            // Record an empty list so any previously-stale metadata is cleared.
            keychain_keys_per_provider.insert(provider_id.clone(), Vec::new());
            continue;
        }

        let mut stored_in_keychain: Vec<String> = Vec::new();
        let mut fallback: HashMap<String, String> = HashMap::new();

        for (cred_key, cred_value) in &provider_config.credentials {
            if keyring_set(provider_id, cred_key, cred_value)? {
                stored_in_keychain.push(cred_key.clone());
            } else {
                fallback.insert(cred_key.clone(), cred_value.clone());
            }
        }

        // Sort for stable, deterministic config output.
        stored_in_keychain.sort();

        // Always record the (possibly empty) keychain key list so that
        // metadata accurately reflects the current storage location.
        keychain_keys_per_provider.insert(provider_id.clone(), stored_in_keychain);

        if !fallback.is_empty() {
            file_creds.insert(provider_id.clone(), fallback);
        }
    }

    // Build config_without_creds once, with up-to-date keychain_credentials metadata.
    let mut config_without_creds = config.clone();
    for (provider_id, provider) in config_without_creds.providers.iter_mut() {
        provider.credentials.clear();
        // Always overwrite keychain_credentials with the freshly computed list
        // so stale entries can never take precedence on a subsequent load.
        provider.keychain_credentials = keychain_keys_per_provider
            .get(provider_id)
            .cloned()
            .unwrap_or_default();
    }

    // Write config.toml (single write).
    let config_content = toml::to_string_pretty(&config_without_creds)
        .map_err(|e| InfsError::ConfigError(format!("Failed to serialize config: {}", e)))?;
    std::fs::write(get_config_path()?, config_content)
        .map_err(|e| InfsError::ConfigError(format!("Failed to write config: {}", e)))?;

    // Write remaining (non-keychain) credentials to credentials.toml.
    #[derive(Serialize)]
    struct CredStore {
        credentials: HashMap<String, String>,
    }

    let cred_store: HashMap<String, CredStore> = file_creds
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
        // Remove all keychain-backed credentials.
        let keys_to_delete: Vec<String> = provider_config.keychain_credentials.clone();
        for cred_key in &keys_to_delete {
            keyring_delete(provider_id, cred_key)?;
        }
        provider_config.keychain_credentials.clear();
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

    #[test]
    fn test_keychain_credentials_not_serialized_when_empty() {
        let prov = ProviderConfig::default();
        let serialized = toml::to_string_pretty(&prov).unwrap();
        // keychain_credentials should be absent when empty
        assert!(!serialized.contains("keychain_credentials"));
    }

    #[test]
    fn test_keychain_credentials_serialized_when_present() {
        let mut prov = ProviderConfig::default();
        prov.keychain_credentials = vec!["api_key".to_string()];
        let serialized = toml::to_string_pretty(&prov).unwrap();
        assert!(serialized.contains("keychain_credentials"));

        let deserialized: ProviderConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.keychain_credentials, vec!["api_key"]);
    }

    #[test]
    fn test_keyring_username_format() {
        assert_eq!(
            keyring_username("openrouter", "api_key"),
            "openrouter/api_key"
        );
        assert_eq!(keyring_username("falai", "token"), "falai/token");
    }

    #[test]
    #[ignore = "requires a real OS keychain (run with `cargo test -- --ignored`)"]
    fn test_keyring_set_get_delete() {
        // Exercise the keyring helpers end-to-end.
        //
        // On CI / headless environments the keyring backend may be unavailable
        // (keyring_set returns Ok(false)) or may accept writes but not persist
        // them reliably (e.g. a stub D-Bus secret service).  Both outcomes are
        // valid graceful-fallback paths; only actual Err(_) results are failures.
        //
        // A unique provider/key pair is used to avoid collisions with other test
        // runs and to ensure any leftover keychain entries are scoped to this test.
        let test_id = format!(
            "infs_test_{}",
            std::time::UNIX_EPOCH
                .elapsed()
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        );
        let provider = test_id.as_str();
        let cred_key = "test_api_key";
        let cred_value = "test_secret_value";

        let set_result = keyring_set(provider, cred_key, cred_value);
        assert!(set_result.is_ok(), "keyring_set must not return an error");

        let stored = set_result.unwrap();
        if stored {
            // Keychain claimed to accept the write — try a round-trip.
            let get_result = keyring_get(provider, cred_key);
            assert!(get_result.is_ok(), "keyring_get must not return an error");

            if get_result.unwrap() == Some(cred_value.to_string()) {
                // Full round-trip succeeded — verify delete too.
                let del_result = keyring_delete(provider, cred_key);
                assert!(
                    del_result.is_ok(),
                    "keyring_delete must not return an error"
                );

                let get_after = keyring_get(provider, cred_key);
                assert!(
                    get_after.is_ok(),
                    "keyring_get after delete must not return an error"
                );
                assert_eq!(get_after.unwrap(), None);
            } else {
                // Keychain accepted the write but didn't persist (stub backend) — clean up.
                let _ = keyring_delete(provider, cred_key);
            }
        } else {
            // No keychain available — get/delete must still succeed gracefully.
            let get_result = keyring_get(provider, cred_key);
            assert!(get_result.is_ok(), "keyring_get must not return an error");
            assert_eq!(get_result.unwrap(), None);

            let del_result = keyring_delete(provider, cred_key);
            assert!(
                del_result.is_ok(),
                "keyring_delete must not return an error"
            );
        }
    }
}
