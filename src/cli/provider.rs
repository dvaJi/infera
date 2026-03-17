use crate::auth;
use crate::config;
use crate::providers::registry::build_registry;
use crate::types::ProviderConnectionStatus;
use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ProviderCommands {
    #[command(subcommand)]
    pub command: ProviderSubcommands,
}

#[derive(Subcommand)]
pub enum ProviderSubcommands {
    /// List all supported providers
    List,
    /// Connect to a provider
    Connect {
        /// Provider ID (e.g. openrouter, falai)
        provider: String,
    },
    /// Disconnect from a provider
    Disconnect {
        /// Provider ID
        provider: String,
    },
    /// Show provider details
    Show {
        /// Provider ID
        provider: String,
    },
}

pub async fn handle(cmd: ProviderCommands) -> Result<()> {
    match cmd.command {
        ProviderSubcommands::List => list_providers().await,
        ProviderSubcommands::Connect { provider } => connect_provider(&provider).await,
        ProviderSubcommands::Disconnect { provider } => disconnect_provider(&provider).await,
        ProviderSubcommands::Show { provider } => show_provider(&provider).await,
    }
}

async fn list_providers() -> Result<()> {
    let registry = build_registry();
    let app_config = config::load_config()?;

    println!(
        "{:<15} {:<25} {:<15} {:<30}",
        "ID", "NAME", "STATUS", "CATEGORIES"
    );
    println!("{}", "-".repeat(90));

    for provider in registry.list_providers() {
        let d = provider.descriptor();
        let prov_config = app_config.providers.get(&d.id);
        let status = if prov_config.is_some_and(|c| c.connected && c.get_api_key().is_some()) {
            ProviderConnectionStatus::Connected
        } else {
            ProviderConnectionStatus::NotConnected
        };

        let categories: Vec<String> = d.categories.iter().map(|c| c.to_string()).collect();
        println!(
            "{:<15} {:<25} {:<15} {:<30}",
            d.id,
            d.display_name,
            status,
            categories.join(", ")
        );
    }

    Ok(())
}

async fn connect_provider(provider_id: &str) -> Result<()> {
    let registry = build_registry();
    let provider = registry.find_provider(provider_id)?;
    let d = provider.descriptor();

    eprintln!("Connecting to {}...", d.display_name);
    eprintln!("Website: {}", d.website);
    eprintln!();

    let auth_methods = provider.supported_auth_methods();
    let auth_descriptor = match auth_methods.first() {
        Some(crate::types::AuthMethod::ApiKey) => {
            auth::get_api_key_descriptor(&d.display_name, &d.api_key_help_url)
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported auth method for provider: {}",
                provider_id
            ));
        }
    };

    let credentials = auth::prompt_credentials(&auth_descriptor)?;

    config::save_provider_credentials(provider_id, credentials)?;

    eprintln!();
    println!("Successfully connected to {}!", d.display_name);
    println!(
        "Run `infs app list --provider {}` to see available apps.",
        provider_id
    );

    Ok(())
}

async fn disconnect_provider(provider_id: &str) -> Result<()> {
    let registry = build_registry();
    let provider = registry.find_provider(provider_id)?;

    config::remove_provider_credentials(provider_id)?;
    println!("Disconnected from {}.", provider.descriptor().display_name);

    Ok(())
}

async fn show_provider(provider_id: &str) -> Result<()> {
    let registry = build_registry();
    let provider = registry.find_provider(provider_id)?;
    let d = provider.descriptor();
    let app_config = config::load_config()?;

    let prov_config = app_config.providers.get(&d.id);
    let status = if prov_config.is_some_and(|c| c.connected && c.get_api_key().is_some()) {
        ProviderConnectionStatus::Connected
    } else {
        ProviderConnectionStatus::NotConnected
    };

    println!("Provider: {}", d.display_name);
    println!("ID:       {}", d.id);
    println!("Status:   {}", status);
    println!("Website:  {}", d.website);
    println!();
    println!("Description:");
    println!("  {}", d.description);
    println!();

    let categories: Vec<String> = d.categories.iter().map(|c| c.to_string()).collect();
    println!("Categories: {}", categories.join(", "));

    let auth_methods: Vec<String> = provider
        .supported_auth_methods()
        .iter()
        .map(|m| m.to_string())
        .collect();
    println!("Auth:       {}", auth_methods.join(", "));

    println!();
    let prov_config_for_list = app_config.providers.get(&d.id).cloned().unwrap_or_default();
    match provider.list_apps(&prov_config_for_list).await {
        Ok(apps) => {
            println!("Apps ({}): ", apps.len());
            for app in &apps {
                println!("  - {} ({})", app.display_name, app.id);
            }
        }
        Err(e) => {
            eprintln!("Warning: could not fetch app list: {}", e);
            println!(
                "Apps: (unavailable — run `infs provider connect {}` to see live models)",
                provider_id
            );
        }
    }

    Ok(())
}
