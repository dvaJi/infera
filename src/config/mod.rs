use crate::error::InfsError;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Directory name used for infs' user-level configuration.
const INFS_CONFIG_DIR: &str = "infs";

/// Configuration directory name used by the official WaveSpeed Node.js CLI.
const WAVESPEED_CLI_CONFIG_DIR: &str = "wavespeed-nodejs";

/// Maximum number of parent directories to search for .env files.
const MAX_ENV_PARENT_DEPTH: usize = 3;

/// Environment variable patterns for provider credentials.
/// Format: (provider_id, environment variable, credential_key)
const PROVIDER_ENV_PATTERNS: &[(&str, &str, &str)] = &[
    ("openrouter", "OPENROUTER_API_KEY", "api_key"),
    ("falai", "FALAI_API_KEY", "api_key"),
    ("replicate", "REPLICATE_API_KEY", "api_key"),
    ("replicate", "REPLICATE_API_TOKEN", "api_key"),
    ("wavespeed", "WAVESPEED_API_KEY", "api_key"),
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    #[serde(default)]
    pub auth_method: Option<String>,
    #[serde(default)]
    pub credentials: HashMap<String, String>,
    #[serde(default)]
    pub connected: bool,
    /// Credential keys loaded from an official provider CLI config.
    /// These are available at runtime but must not be written to infs' config.
    #[serde(skip)]
    pub external_credentials: Vec<String>,
}

impl ProviderConfig {
    pub fn get_api_key(&self) -> Option<&str> {
        self.credentials.get("api_key").map(String::as_str)
    }
}

pub fn get_config_dir() -> Result<PathBuf, InfsError> {
    let base_dirs = BaseDirs::new().ok_or_else(|| {
        InfsError::ConfigError("Could not determine config directory".to_string())
    })?;
    Ok(base_dirs.config_dir().join(INFS_CONFIG_DIR))
}

pub fn get_config_path() -> Result<PathBuf, InfsError> {
    Ok(get_config_dir()?.join("config.json"))
}

// ---------------------------------------------------------------------------
// Official provider CLI credential compatibility
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct WavespeedCliConfig {
    #[serde(rename = "apiKey")]
    api_key: Option<String>,
}

/// Return the path used by `@wavespeed/cli`'s `conf` store.
///
/// `conf` uses `env-paths` with the default `nodejs` suffix. This results in
/// `~/.config/wavespeed-nodejs/config.json` on Linux,
/// `~/Library/Preferences/wavespeed-nodejs/config.json` on macOS, and
/// `%APPDATA%\wavespeed-nodejs\Config\config.json` on Windows.
fn wavespeed_cli_config_path() -> Option<PathBuf> {
    let base_dirs = BaseDirs::new()?;
    let mut path = if cfg!(target_os = "macos") {
        base_dirs
            .home_dir()
            .join("Library")
            .join("Preferences")
            .join(WAVESPEED_CLI_CONFIG_DIR)
    } else {
        base_dirs.config_dir().join(WAVESPEED_CLI_CONFIG_DIR)
    };

    #[cfg(windows)]
    path.push("Config");

    path.push("config.json");
    Some(path)
}

fn parse_wavespeed_cli_api_key(content: &str) -> Option<String> {
    let config: WavespeedCliConfig = serde_json::from_str(content).ok()?;
    let api_key = config.api_key?.trim().to_string();
    (!api_key.is_empty()).then_some(api_key)
}

#[derive(Debug, Deserialize)]
struct ReplicateCliHost {
    token: Option<String>,
}

/// Return the path used by the official Replicate CLI's auth config.
///
/// The Go CLI uses `$XDG_CONFIG_HOME/replicate/hosts` when that variable is
/// set, otherwise it uses `~/.config/replicate/hosts` on every platform.
fn replicate_cli_config_path() -> Option<PathBuf> {
    let config_dir = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(path) => PathBuf::from(path),
        None => BaseDirs::new()?.home_dir().join(".config"),
    };

    Some(config_dir.join("replicate").join("hosts"))
}

fn parse_replicate_cli_api_token(content: &str) -> Option<String> {
    let config: HashMap<String, ReplicateCliHost> = serde_yaml::from_str(content).ok()?;
    let token = config
        .get("api.replicate.com")?
        .token
        .as_deref()?
        .trim()
        .to_string();
    (!token.is_empty()).then_some(token)
}

/// Read the API key persisted by the official WaveSpeed CLI, if present.
fn wavespeed_cli_api_key() -> Option<String> {
    let path = wavespeed_cli_config_path()?;
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            tracing::debug!(path = ?path, %error, "could not read WaveSpeed CLI config");
            return None;
        }
    };

    let api_key = parse_wavespeed_cli_api_key(&content);
    if api_key.is_some() {
        tracing::debug!(path = ?path, "using API key from WaveSpeed CLI config");
    }
    api_key
}

/// Read the API token persisted by the official Replicate CLI, if present.
fn replicate_cli_api_token() -> Option<String> {
    let path = replicate_cli_config_path()?;
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            tracing::debug!(path = ?path, %error, "could not read Replicate CLI config");
            return None;
        }
    };

    let api_token = parse_replicate_cli_api_token(&content);
    if api_token.is_some() {
        tracing::debug!(path = ?path, "using API token from Replicate CLI config");
    }
    api_token
}

fn merge_external_credential(config: &mut AppConfig, provider_id: &str, credential: String) {
    let provider_config = config.providers.entry(provider_id.to_string()).or_default();

    // A key explicitly stored by infs always wins over an official provider
    // CLI credential.
    if provider_config.credentials.contains_key("api_key") {
        return;
    }

    provider_config
        .credentials
        .insert("api_key".to_string(), credential);
    provider_config.external_credentials = vec!["api_key".to_string()];
    provider_config.connected = true;
}

fn merge_wavespeed_cli_credentials(config: &mut AppConfig, api_key: String) {
    merge_external_credential(config, "wavespeed", api_key);
}

fn merge_replicate_cli_credentials(config: &mut AppConfig, api_token: String) {
    merge_external_credential(config, "replicate", api_token);
}

// ---------------------------------------------------------------------------
// .env file loading
// ---------------------------------------------------------------------------

/// Load .env files from the current directory and up to MAX_ENV_PARENT_DEPTH parent directories.
/// Returns the path of the .env file that was loaded, if any.
pub fn load_dotenv() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let mut current = cwd.as_path();

    for _ in 0..=MAX_ENV_PARENT_DEPTH {
        let env_path = current.join(".env");
        if env_path.exists() && env_path.is_file() {
            match dotenvy::from_path_override(&env_path) {
                Ok(()) => {
                    tracing::debug!("Loaded .env from: {:?}", env_path);
                    return Some(env_path);
                }
                Err(error) => {
                    tracing::warn!("Failed to load .env from {:?}: {}", env_path, error);
                }
            }
        }

        current = current.parent()?;
    }

    None
}

/// Extract provider credentials from environment variables.
pub fn credentials_from_env() -> HashMap<String, HashMap<String, String>> {
    let mut result: HashMap<String, HashMap<String, String>> = HashMap::new();

    for (provider_id, env_var, credential_key) in PROVIDER_ENV_PATTERNS {
        if let Ok(value) = std::env::var(env_var) {
            if !value.is_empty() {
                result
                    .entry(provider_id.to_string())
                    .or_default()
                    .insert(credential_key.to_string(), value);
            }
        }
    }

    result
}

fn merge_env_credentials(
    config: &mut AppConfig,
    env_credentials: HashMap<String, HashMap<String, String>>,
) {
    for (provider_id, credentials) in env_credentials {
        let provider_config = config.providers.entry(provider_id).or_default();
        provider_config.connected = true;
        for (key, value) in credentials {
            provider_config
                .external_credentials
                .retain(|external_key| external_key != &key);
            provider_config.credentials.insert(key, value);
        }
    }
}

// ---------------------------------------------------------------------------
// Credential source detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSource {
    Environment { var_name: String },
    Config,
    WaveSpeedCli,
    ReplicateCli,
    NotFound,
}

impl CredentialSource {
    pub fn display(&self) -> String {
        match self {
            CredentialSource::Environment { var_name } => format!("env var: {}", var_name),
            CredentialSource::Config => "infs config.json".to_string(),
            CredentialSource::WaveSpeedCli => "WaveSpeed CLI config".to_string(),
            CredentialSource::ReplicateCli => "Replicate CLI config".to_string(),
            CredentialSource::NotFound => "not configured".to_string(),
        }
    }
}

pub fn get_credential_source_with_env(
    provider_id: &str,
    load_env: bool,
) -> Result<CredentialSource, InfsError> {
    let credential_key = PROVIDER_ENV_PATTERNS
        .iter()
        .find(|(known_provider, _, _)| *known_provider == provider_id)
        .map(|(_, _, key)| *key)
        .unwrap_or("api_key");

    if load_env {
        for (known_provider, env_var, _) in PROVIDER_ENV_PATTERNS.iter().rev() {
            if *known_provider == provider_id {
                if let Ok(value) = std::env::var(env_var) {
                    if !value.is_empty() {
                        return Ok(CredentialSource::Environment {
                            var_name: (*env_var).to_string(),
                        });
                    }
                }
            }
        }
    }

    let config = load_config_with_env(false)?;
    if let Some(provider_config) = config.providers.get(provider_id) {
        if provider_config.credentials.contains_key(credential_key) {
            if provider_config
                .external_credentials
                .iter()
                .any(|key| key == credential_key)
            {
                return Ok(match provider_id {
                    "wavespeed" => CredentialSource::WaveSpeedCli,
                    "replicate" => CredentialSource::ReplicateCli,
                    _ => CredentialSource::Config,
                });
            }

            return Ok(CredentialSource::Config);
        }
    }

    Ok(CredentialSource::NotFound)
}

// ---------------------------------------------------------------------------
// Config load / save
// ---------------------------------------------------------------------------

pub fn load_config_with_env(load_env: bool) -> Result<AppConfig, InfsError> {
    let config_path = get_config_path()?;
    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|error| InfsError::ConfigError(format!("Failed to read config: {}", error)))?;
        serde_json::from_str(&content)
            .map_err(|error| InfsError::ConfigError(format!("Failed to parse config: {}", error)))?
    } else {
        AppConfig::default()
    };

    if let Some(api_key) = wavespeed_cli_api_key() {
        merge_wavespeed_cli_credentials(&mut config, api_key);
    }
    if let Some(api_token) = replicate_cli_api_token() {
        merge_replicate_cli_credentials(&mut config, api_token);
    }

    if load_env {
        merge_env_credentials(&mut config, credentials_from_env());
    }

    Ok(config)
}

fn strip_external_credentials(config: &mut AppConfig) {
    for provider_config in config.providers.values_mut() {
        for key in provider_config.external_credentials.clone() {
            provider_config.credentials.remove(&key);
        }
        provider_config.external_credentials.clear();
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), InfsError> {
    let config_dir = get_config_dir()?;
    std::fs::create_dir_all(&config_dir).map_err(|error| {
        InfsError::ConfigError(format!("Failed to create config dir: {}", error))
    })?;

    let mut config_to_save = config.clone();
    strip_external_credentials(&mut config_to_save);
    let content = serde_json::to_string_pretty(&config_to_save).map_err(|error| {
        InfsError::ConfigError(format!("Failed to serialize config: {}", error))
    })?;

    write_config_file(&get_config_path()?, &content)
}

fn write_config_file(path: &Path, content: &str) -> Result<(), InfsError> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .map_err(|error| {
                InfsError::ConfigError(format!("Failed to open config file: {}", error))
            })?;
        file.write_all(content.as_bytes()).map_err(|error| {
            InfsError::ConfigError(format!("Failed to write config file: {}", error))
        })?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(
            |error| InfsError::ConfigError(format!("Failed to secure config file: {}", error)),
        )?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, content).map_err(|error| {
            InfsError::ConfigError(format!("Failed to write config file: {}", error))
        })
    }
}

pub fn save_provider_credentials(
    provider_id: &str,
    credentials: HashMap<String, String>,
) -> Result<(), InfsError> {
    let mut config = load_config_with_env(false)?;

    let provider_config = config.providers.entry(provider_id.to_string()).or_default();
    provider_config.credentials = credentials;
    provider_config.external_credentials.clear();
    provider_config.connected = true;
    save_config(&config)
}

pub fn remove_provider_credentials(provider_id: &str) -> Result<(), InfsError> {
    let mut config = load_config_with_env(false)?;

    if let Some(provider_config) = config.providers.get_mut(provider_id) {
        provider_config.credentials.clear();
        provider_config.external_credentials.clear();
        provider_config.connected = false;
    }
    save_config(&config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn test_env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct TestEnvGuard {
        original_cwd: PathBuf,
        original_vars: HashMap<String, Option<String>>,
    }

    impl TestEnvGuard {
        fn new(vars: &[&str]) -> Self {
            Self {
                original_cwd: std::env::current_dir().unwrap(),
                original_vars: vars
                    .iter()
                    .map(|key| ((*key).to_string(), std::env::var(key).ok()))
                    .collect(),
            }
        }
    }

    impl Drop for TestEnvGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_cwd);
            for (key, value) in &self.original_vars {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    #[test]
    fn test_app_config_default() {
        assert!(AppConfig::default().providers.is_empty());
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
    fn test_config_path_is_json() {
        assert_eq!(
            get_config_path()
                .unwrap()
                .file_name()
                .and_then(|name| name.to_str()),
            Some("config.json")
        );
    }

    #[test]
    fn test_config_json_roundtrip_includes_credentials() {
        let mut config = AppConfig::default();
        config.providers.insert(
            "openrouter".to_string(),
            ProviderConfig {
                auth_method: Some("api_key".to_string()),
                credentials: HashMap::from([("api_key".to_string(), "test-key".to_string())]),
                connected: true,
                ..Default::default()
            },
        );

        let serialized = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized.providers["openrouter"].get_api_key(),
            Some("test-key")
        );
        assert!(!serialized.contains("external_credentials"));
    }

    #[test]
    fn test_external_credentials_are_removed_before_save() {
        let mut config = AppConfig::default();
        config.providers.insert(
            "wavespeed".to_string(),
            ProviderConfig {
                credentials: HashMap::from([("api_key".to_string(), "external-key".to_string())]),
                external_credentials: vec!["api_key".to_string()],
                connected: true,
                ..Default::default()
            },
        );

        strip_external_credentials(&mut config);
        assert!(config.providers["wavespeed"].credentials.is_empty());
    }

    #[test]
    fn test_write_config_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("config.json");
        write_config_file(&path, "{\"ok\":true}").unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "{\"ok\":true}");
    }

    #[test]
    fn test_parse_wavespeed_cli_api_key() {
        assert_eq!(
            parse_wavespeed_cli_api_key(r#"{"apiKey":"  external-key  "}"#),
            Some("external-key".to_string())
        );
        assert_eq!(parse_wavespeed_cli_api_key(r#"{"apiKey":"   "}"#), None);
        assert_eq!(parse_wavespeed_cli_api_key("not-json"), None);
    }

    #[test]
    fn test_parse_replicate_cli_api_token() {
        let content = r#"
api.replicate.com:
  token: "  replicate-token  "
"#;
        assert_eq!(
            parse_replicate_cli_api_token(content),
            Some("replicate-token".to_string())
        );
        assert_eq!(
            parse_replicate_cli_api_token("api.other.com:\n  token: other-token"),
            None
        );
        assert_eq!(parse_replicate_cli_api_token("not-yaml: ["), None);
    }

    #[test]
    fn test_replicate_cli_api_token_reads_xdg_config() {
        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&["XDG_CONFIG_HOME"]);
        let temp_dir = tempfile::tempdir().unwrap();
        let config_dir = temp_dir.path().join("replicate");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("hosts"),
            "api.replicate.com:\n  token: stored-replicate-token\n",
        )
        .unwrap();
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

        assert_eq!(
            replicate_cli_api_token(),
            Some("stored-replicate-token".to_string())
        );
    }

    #[test]
    fn test_external_cli_credentials_are_fallback_only() {
        let mut config = AppConfig::default();
        merge_wavespeed_cli_credentials(&mut config, "external-key".to_string());
        assert_eq!(
            config.providers["wavespeed"].get_api_key(),
            Some("external-key")
        );
        assert!(config.providers["wavespeed"].connected);

        let mut config = AppConfig::default();
        config.providers.insert(
            "replicate".to_string(),
            ProviderConfig {
                credentials: HashMap::from([("api_key".to_string(), "infs-key".to_string())]),
                ..Default::default()
            },
        );
        merge_replicate_cli_credentials(&mut config, "external-token".to_string());
        assert_eq!(
            config.providers["replicate"].get_api_key(),
            Some("infs-key")
        );
    }

    #[test]
    fn test_credentials_from_env_empty() {
        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&[
            "OPENROUTER_API_KEY",
            "FALAI_API_KEY",
            "REPLICATE_API_KEY",
            "REPLICATE_API_TOKEN",
            "WAVESPEED_API_KEY",
        ]);

        for variable in [
            "OPENROUTER_API_KEY",
            "FALAI_API_KEY",
            "REPLICATE_API_KEY",
            "REPLICATE_API_TOKEN",
            "WAVESPEED_API_KEY",
        ] {
            std::env::remove_var(variable);
        }

        assert!(credentials_from_env().is_empty());
    }

    #[test]
    fn test_credentials_from_env_with_values() {
        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&[
            "OPENROUTER_API_KEY",
            "FALAI_API_KEY",
            "REPLICATE_API_KEY",
            "REPLICATE_API_TOKEN",
            "WAVESPEED_API_KEY",
        ]);

        std::env::set_var("OPENROUTER_API_KEY", "test-openrouter-key");
        std::env::set_var("FALAI_API_KEY", "test-falai-key");
        std::env::set_var("REPLICATE_API_TOKEN", "test-replicate-token");

        let credentials = credentials_from_env();
        assert_eq!(credentials["openrouter"]["api_key"], "test-openrouter-key");
        assert_eq!(credentials["falai"]["api_key"], "test-falai-key");
        assert_eq!(credentials["replicate"]["api_key"], "test-replicate-token");
    }

    #[test]
    fn test_env_credentials_override_config() {
        let mut config = AppConfig::default();
        config.providers.insert(
            "openrouter".to_string(),
            ProviderConfig {
                credentials: HashMap::from([("api_key".to_string(), "from-config".to_string())]),
                ..Default::default()
            },
        );

        merge_env_credentials(
            &mut config,
            HashMap::from([(
                "openrouter".to_string(),
                HashMap::from([("api_key".to_string(), "from-env".to_string())]),
            )]),
        );

        assert_eq!(
            config.providers["openrouter"].get_api_key(),
            Some("from-env")
        );
        assert!(config.providers["openrouter"].connected);
    }

    #[test]
    fn test_credential_source_env() {
        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&["OPENROUTER_API_KEY"]);
        std::env::set_var("OPENROUTER_API_KEY", "test-key");

        assert_eq!(
            get_credential_source_with_env("openrouter", true).unwrap(),
            CredentialSource::Environment {
                var_name: "OPENROUTER_API_KEY".to_string()
            }
        );
    }

    #[test]
    fn test_credential_source_replicate_api_token_env() {
        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&["REPLICATE_API_KEY", "REPLICATE_API_TOKEN"]);
        std::env::remove_var("REPLICATE_API_KEY");
        std::env::set_var("REPLICATE_API_TOKEN", "test-token");

        assert_eq!(
            get_credential_source_with_env("replicate", true).unwrap(),
            CredentialSource::Environment {
                var_name: "REPLICATE_API_TOKEN".to_string()
            }
        );
    }

    #[test]
    fn test_credential_source_no_env_ignores_environment() {
        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&["OPENROUTER_API_KEY"]);
        std::env::set_var("OPENROUTER_API_KEY", "test-key");

        assert_eq!(
            get_credential_source_with_env("openrouter", false).unwrap(),
            CredentialSource::NotFound
        );
    }

    #[test]
    fn test_replicate_api_token_env_wins_over_api_key() {
        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&["REPLICATE_API_KEY", "REPLICATE_API_TOKEN"]);
        std::env::set_var("REPLICATE_API_KEY", "api-key");
        std::env::set_var("REPLICATE_API_TOKEN", "api-token");

        let credentials = credentials_from_env();
        assert_eq!(credentials["replicate"]["api_key"], "api-token");
        assert_eq!(
            get_credential_source_with_env("replicate", true).unwrap(),
            CredentialSource::Environment {
                var_name: "REPLICATE_API_TOKEN".to_string()
            }
        );
    }

    #[test]
    fn test_credential_source_not_found() {
        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&[]);
        assert_eq!(
            get_credential_source_with_env("provider-that-does-not-exist", true).unwrap(),
            CredentialSource::NotFound
        );
    }

    #[test]
    fn test_load_dotenv_finds_file_in_current_dir() {
        use std::io::Write;

        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&["TEST_VAR"]);
        let temp_dir = tempfile::tempdir().unwrap();
        let env_path = temp_dir.path().join(".env");
        let mut file = std::fs::File::create(&env_path).unwrap();
        writeln!(file, "TEST_VAR=test_value").unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        std::env::remove_var("TEST_VAR");

        assert!(load_dotenv().is_some());
        assert_eq!(
            std::env::var("TEST_VAR").ok(),
            Some("test_value".to_string())
        );
    }

    #[test]
    fn test_load_dotenv_searches_parent_dirs() {
        use std::io::Write;

        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&["PARENT_TEST_VAR"]);
        let temp_dir = tempfile::tempdir().unwrap();
        let env_path = temp_dir.path().join(".env");
        let mut file = std::fs::File::create(&env_path).unwrap();
        writeln!(file, "PARENT_TEST_VAR=parent_value").unwrap();

        let child_dir = temp_dir.path().join("child");
        std::fs::create_dir(&child_dir).unwrap();
        std::env::set_current_dir(&child_dir).unwrap();
        std::env::remove_var("PARENT_TEST_VAR");

        assert!(load_dotenv().is_some());
        assert_eq!(
            std::env::var("PARENT_TEST_VAR").ok(),
            Some("parent_value".to_string())
        );
    }

    #[test]
    fn test_load_dotenv_returns_none_when_no_file() {
        let _lock = test_env_lock();
        let _guard = TestEnvGuard::new(&[]);
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        assert!(load_dotenv().is_none());
    }
}
