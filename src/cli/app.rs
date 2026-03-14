use clap::{Args, Subcommand};
use anyhow::Result;
use crate::catalog::Catalog;
use crate::config;
use crate::providers::registry::build_registry;
use crate::types::AppId;

#[derive(Args)]
pub struct AppCommands {
    #[command(subcommand)]
    pub command: AppSubcommands,
}

#[derive(Subcommand)]
pub enum AppSubcommands {
    /// List available apps/models
    List {
        /// Filter by category (image, llm, audio, video)
        #[arg(long)]
        category: Option<String>,
        /// Filter by provider
        #[arg(long)]
        provider: Option<String>,
    },
    /// Run an app with JSON input
    Run {
        /// App ID in format provider/app-id (e.g. openrouter/openai/gpt-4o)
        app: String,
        /// Input as JSON string (e.g. '{"prompt": "Hello"}')
        #[arg(long, short)]
        input: String,
    },
    /// Show app details
    Show {
        /// App ID in format provider/app-id
        app: String,
    },
}

pub async fn handle(cmd: AppCommands) -> Result<()> {
    match cmd.command {
        AppSubcommands::List { category, provider } => list_apps(category, provider).await,
        AppSubcommands::Run { app, input } => run_app(app, input).await,
        AppSubcommands::Show { app } => show_app(app).await,
    }
}

async fn list_apps(category_filter: Option<String>, provider_filter: Option<String>) -> Result<()> {
    let registry = build_registry();
    let app_config = config::load_config()?;
    let catalog = Catalog::new(&registry, &app_config);

    let apps = if let Some(provider_id) = &provider_filter {
        // Verify provider exists, then fetch live
        registry.find_provider(provider_id)?;
        catalog.list_apps_by_provider(provider_id).await?
    } else if let Some(cat_str) = &category_filter {
        let category = parse_category(cat_str)?;
        catalog.list_apps_by_category(&category).await
    } else {
        catalog.list_all_apps().await
    };

    println!("{:<45} {:<25} {:<10} {}", "FULL ID", "NAME", "CATEGORY", "DESCRIPTION");
    println!("{}", "-".repeat(110));

    for app in &apps {
        let full_id = app.full_id();
        let truncated_desc = truncate_str(&app.description, 40);
        println!(
            "{:<45} {:<25} {:<10} {}",
            full_id, app.display_name, app.category, truncated_desc
        );
    }

    eprintln!();
    eprintln!("Total: {} apps", apps.len());

    Ok(())
}

async fn run_app(app_str: String, input_str: String) -> Result<()> {
    let app_id = AppId::parse(&app_str)?;

    let input: serde_json::Value = serde_json::from_str(&input_str)
        .map_err(|e| crate::error::InfsError::InvalidInput(format!("Invalid JSON input: {}", e)))?;

    let registry = build_registry();
    let provider = registry.find_provider(&app_id.provider)?;

    let app_config = config::load_config()?;
    let prov_config = app_config
        .providers
        .get(&app_id.provider)
        .cloned()
        .unwrap_or_default();

    provider.validate_config(&prov_config)?;

    eprintln!("Running {}/{}...", app_id.provider, app_id.app);

    let response = provider.run_app(&app_id.app, input, &prov_config).await?;

    match &response.output {
        crate::types::RunOutput::Text(text) => {
            println!("{}", text);
        }
        crate::types::RunOutput::ImageUrls(urls) => {
            for url in urls {
                println!("{}", url);
            }
        }
        crate::types::RunOutput::Json(val) => {
            println!("{}", serde_json::to_string_pretty(val)?);
        }
    }

    if let Some(usage) = &response.usage {
        if let Some(total) = usage.total_tokens {
            eprintln!("Tokens used: {}", total);
        }
    }

    Ok(())
}

async fn show_app(app_str: String) -> Result<()> {
    let app_id = AppId::parse(&app_str)?;

    let registry = build_registry();
    let app_config = config::load_config()?;
    let catalog = Catalog::new(&registry, &app_config);

    let app = catalog
        .find_app(&app_id.provider, &app_id.app)
        .await
        .ok_or_else(|| {
            crate::error::InfsError::InvalidAppId(format!(
                "App '{}' not found in provider '{}'",
                app_id.app, app_id.provider
            ))
        })?;

    println!("App:      {}", app.display_name);
    println!("ID:       {}", app.full_id());
    println!("Category: {}", app.category);
    println!();
    println!("Description:");
    println!("  {}", app.description);

    if !app.tags.is_empty() {
        println!();
        println!("Tags: {}", app.tags.join(", "));
    }

    Ok(())
}

fn parse_category(s: &str) -> Result<crate::types::AppCategory> {
    match s.to_lowercase().as_str() {
        "image" => Ok(crate::types::AppCategory::Image),
        "llm" => Ok(crate::types::AppCategory::Llm),
        "audio" => Ok(crate::types::AppCategory::Audio),
        "video" => Ok(crate::types::AppCategory::Video),
        "other" => Ok(crate::types::AppCategory::Other),
        _ => Err(anyhow::anyhow!(
            "Unknown category: '{}'. Valid: image, llm, audio, video, other",
            s
        )),
    }
}

/// Truncate a string to at most `max_chars` Unicode scalar values, appending "..." if truncated.
fn truncate_str(s: &str, max_chars: usize) -> String {
    let mut char_count = 0usize;
    let mut last_boundary = 0usize; // byte index of the trim point (max_chars - 3)
    let trim_to = max_chars.saturating_sub(3);

    for (byte_idx, _ch) in s.char_indices() {
        if char_count == trim_to {
            last_boundary = byte_idx;
        }
        if char_count == max_chars {
            // Need to truncate — use the saved boundary
            let mut out = s[..last_boundary].to_string();
            out.push_str("...");
            return out;
        }
        char_count += 1;
    }
    // String fits within max_chars — return as-is
    s.to_string()
}
