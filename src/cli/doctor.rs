use crate::config;
use crate::providers::registry::build_registry;
use crate::types::ProviderConnectionStatus;
use anyhow::Result;

pub async fn handle(load_env: bool) -> Result<()> {
    println!("infs Doctor");
    println!("===========");
    println!();

    // Check config
    let config_path = config::get_config_path()?;
    println!("Config file: {}", config_path.display());
    println!(
        "  Exists: {}",
        if config_path.exists() {
            "yes"
        } else {
            "no (will be created on first connect)"
        }
    );
    println!();

    // Check providers
    let registry = build_registry();
    let app_config = config::load_config_with_env(load_env)?;

    println!("Providers:");
    for provider in registry.list_providers() {
        let d = provider.descriptor();
        let prov_config = app_config.providers.get(&d.id);
        let status = if prov_config.is_some_and(|c| c.connected) {
            ProviderConnectionStatus::Connected
        } else {
            ProviderConnectionStatus::NotConnected
        };

        let icon = match status {
            ProviderConnectionStatus::Connected => "✓",
            ProviderConnectionStatus::NotConnected => "✗",
        };

        println!("  {} {} ({})", icon, d.display_name, d.id);
        if status == ProviderConnectionStatus::NotConnected {
            println!("    → Run: infs provider connect {}", d.id);
        }
    }

    println!();
    println!("To connect a provider:");
    println!("  infs provider connect openrouter");

    Ok(())
}
