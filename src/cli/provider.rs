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

pub async fn handle(cmd: ProviderCommands, json: bool, load_env: bool) -> Result<()> {
    match cmd.command {
        ProviderSubcommands::List => list_providers(json, load_env).await,
        ProviderSubcommands::Connect { provider } => connect_provider(&provider, load_env).await,
        ProviderSubcommands::Disconnect { provider } => {
            disconnect_provider(&provider, load_env).await
        }
        ProviderSubcommands::Show { provider } => show_provider(&provider, json, load_env).await,
    }
}

async fn list_providers(json: bool, load_env: bool) -> Result<()> {
    let registry = build_registry();
    let app_config = config::load_config_with_env(load_env)?;

    let mut rows = Vec::new();
    for provider in registry.list_providers() {
        let d = provider.descriptor();
        let prov_config = app_config.providers.get(&d.id);
        let status = if prov_config.is_some_and(|c| c.connected && c.get_api_key().is_some()) {
            ProviderConnectionStatus::Connected
        } else {
            ProviderConnectionStatus::NotConnected
        };
        let categories: Vec<String> = d.categories.iter().map(|c| c.to_string()).collect();
        rows.push((d.id.clone(), d.display_name.clone(), status, categories));
    }

    if json {
        let json_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|(id, name, status, cats)| {
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "status": status.to_string(),
                    "categories": cats,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_rows)?);
    } else {
        println!(
            "{:<15} {:<25} {:<15} {:<30}",
            "ID", "NAME", "STATUS", "CATEGORIES"
        );
        println!("{}", "-".repeat(90));
        for (id, name, status, cats) in &rows {
            println!(
                "{:<15} {:<25} {:<15} {:<30}",
                id,
                name,
                status,
                cats.join(", ")
            );
        }
    }

    Ok(())
}

async fn connect_provider(provider_id: &str, load_env: bool) -> Result<()> {
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

    config::save_provider_credentials_with_env(provider_id, credentials, load_env)?;

    eprintln!();
    println!("Successfully connected to {}!", d.display_name);
    println!(
        "Run `infs app list --provider {}` to see available apps.",
        provider_id
    );

    Ok(())
}

async fn disconnect_provider(provider_id: &str, load_env: bool) -> Result<()> {
    let registry = build_registry();
    let provider = registry.find_provider(provider_id)?;

    config::remove_provider_credentials_with_env(provider_id, load_env)?;
    println!("Disconnected from {}.", provider.descriptor().display_name);

    Ok(())
}

async fn show_provider(provider_id: &str, json: bool, load_env: bool) -> Result<()> {
    let registry = build_registry();
    let provider = registry.find_provider(provider_id)?;
    let d = provider.descriptor();
    let app_config = config::load_config_with_env(load_env)?;

    let prov_config = app_config.providers.get(&d.id);
    let status = if prov_config.is_some_and(|c| c.connected && c.get_api_key().is_some()) {
        ProviderConnectionStatus::Connected
    } else {
        ProviderConnectionStatus::NotConnected
    };

    let prov_config_for_list = app_config.providers.get(&d.id).cloned().unwrap_or_default();
    let apps_result = provider.list_apps(&prov_config_for_list).await;

    if json {
        let apps_json: serde_json::Value = match apps_result {
            Ok(apps) => apps
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "id": a.id,
                        "full_id": a.full_id(),
                        "name": a.display_name,
                        "category": a.category.to_string(),
                    })
                })
                .collect(),
            Err(_) => serde_json::Value::Null,
        };
        let categories: Vec<String> = d.categories.iter().map(|c| c.to_string()).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "id": d.id,
                "name": d.display_name,
                "description": d.description,
                "status": status.to_string(),
                "website": d.website,
                "categories": categories,
                "apps": apps_json,
            }))?
        );
    } else {
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
        match apps_result {
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
    }

    Ok(())
}
