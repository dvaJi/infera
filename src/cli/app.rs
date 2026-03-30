use crate::catalog::Catalog;
use crate::config;
use crate::providers::registry::build_registry;
use crate::types::{AppCategory, AppDescriptor, AppId, ListOptions};
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct AppCommands {
    #[command(subcommand)]
    pub command: AppSubcommands,
}

#[derive(Subcommand)]
pub enum AppSubcommands {
    /// List providers or available apps/models
    List {
        /// Provider ID to list models for (e.g. openrouter)
        provider: Option<String>,
        /// Filter by category (image, llm, audio, video)
        #[arg(long)]
        category: Option<String>,
        /// Page number for paginated output (1-based)
        #[arg(long, default_value_t = 1)]
        page: usize,
        /// Number of results per page
        #[arg(long, default_value_t = 20)]
        per_page: usize,
    },
    /// Run an app with JSON input
    Run {
        /// App ID in format provider/app-id (e.g. openrouter/openai/gpt-4o)
        app: String,
        /// Input as JSON string (e.g. '{"prompt": "Hello"}')
        #[arg(
            long,
            short,
            required_unless_present = "input_file",
            conflicts_with = "input_file"
        )]
        input: Option<String>,
        /// Read input JSON from a file instead of --input
        #[arg(long, conflicts_with = "input")]
        input_file: Option<PathBuf>,
        /// Stream output token by token (LLM providers only)
        #[arg(long)]
        stream: bool,
        /// Save image output to this file path (image providers only)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Show app details
    Show {
        /// App ID in format provider/app-id
        app: String,
    },
}

pub async fn handle(cmd: AppCommands, json: bool, load_env: bool) -> Result<()> {
    match cmd.command {
        AppSubcommands::List {
            category,
            provider,
            page,
            per_page,
        } => list_apps(category, provider, page, per_page, json, load_env).await,
        AppSubcommands::Run {
            app,
            input,
            input_file,
            stream,
            output,
        } => run_app(app, input, input_file, stream, output, json, load_env).await,
        AppSubcommands::Show { app } => show_app(app, json, load_env).await,
    }
}

async fn list_apps(
    category_filter: Option<String>,
    provider_filter: Option<String>,
    page: usize,
    per_page: usize,
    json: bool,
    load_env: bool,
) -> Result<()> {
    let registry = build_registry();
    let app_config = config::load_config_with_env(load_env)?;

    if let Some(provider_id) = &provider_filter {
        let provider = registry.find_provider(provider_id)?;
        let prov_config = app_config
            .providers
            .get(provider_id)
            .cloned()
            .unwrap_or_default();

        let options = ListOptions::new(page, per_page);
        let apps = provider.list_apps(&prov_config, &options).await?;
        let apps = filter_apps_by_category(apps, category_filter.as_deref())?;

        return print_app_list(
            provider.descriptor().display_name.as_str(),
            provider_id,
            apps,
            page,
            per_page,
            json,
        );
    }

    let category = match category_filter.as_deref() {
        Some(cat) => Some(parse_category(cat)?),
        None => None,
    };

    let mut providers = Vec::new();
    for provider in registry.list_providers() {
        let descriptor = provider.descriptor();

        if let Some(category) = &category {
            if !descriptor.categories.contains(category) {
                continue;
            }
        }

        let prov_config = app_config.providers.get(&descriptor.id);
        let status = match prov_config {
            Some(config) if !config.credentials.is_empty() => "available",
            _ => "needs_credentials",
        };

        providers.push(serde_json::json!({
            "id": descriptor.id,
            "name": descriptor.display_name,
            "status": status,
            "categories": descriptor
                .categories
                .iter()
                .map(|category| category.to_string())
                .collect::<Vec<_>>(),
            "website": descriptor.website,
        }));
    }

    let total = providers.len();
    let per_page = per_page.max(1);
    let total_pages = total.div_ceil(per_page).max(1);
    let page = page.max(1).min(total_pages);
    let start = (page - 1) * per_page;
    let page_providers: Vec<_> = providers
        .iter()
        .skip(start)
        .take(per_page)
        .cloned()
        .collect();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "total": total,
                "page": page,
                "per_page": per_page,
                "total_pages": total_pages,
                "providers": page_providers,
            }))?
        );
        return Ok(());
    }

    println!("{:<15} {:<25} {:<20} CATEGORIES", "ID", "NAME", "STATUS");
    println!("{}", "-".repeat(90));

    for provider in &page_providers {
        let id = provider["id"].as_str().unwrap_or_default();
        let name = provider["name"].as_str().unwrap_or_default();
        let status = match provider["status"].as_str().unwrap_or_default() {
            "available" => "available",
            _ => "needs credentials",
        };
        let categories = provider["categories"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        println!("{:<15} {:<25} {:<20} {}", id, name, status, categories);
    }

    eprintln!();
    eprintln!("Use `infs app list <id>` to see all models for a provider.");
    if total_pages > 1 {
        eprintln!(
            "Page {}/{} - showing {}-{} of {} providers",
            page,
            total_pages,
            start + 1,
            (start + page_providers.len()).min(total),
            total
        );
    } else {
        eprintln!("Total: {} providers", total);
    }

    Ok(())
}

fn filter_apps_by_category(
    apps: Vec<AppDescriptor>,
    category_filter: Option<&str>,
) -> Result<Vec<AppDescriptor>> {
    let Some(category_filter) = category_filter else {
        return Ok(apps);
    };

    let category = parse_category(category_filter)?;
    Ok(apps
        .into_iter()
        .filter(|app| app.category == category)
        .collect())
}

fn print_app_list(
    provider_name: &str,
    provider_id: &str,
    apps: Vec<AppDescriptor>,
    page: usize,
    per_page: usize,
    json: bool,
) -> Result<()> {
    let total = apps.len();
    let per_page = per_page.max(1);
    let total_pages = total.div_ceil(per_page).max(1);
    let page = page.max(1).min(total_pages);
    let start = (page - 1) * per_page;
    let page_apps: Vec<_> = apps.iter().skip(start).take(per_page).collect();

    if json {
        let json_apps: Vec<serde_json::Value> = page_apps
            .iter()
            .map(|app| {
                serde_json::json!({
                    "full_id": app.full_id(),
                    "name": app.display_name,
                    "category": app.category.to_string(),
                    "description": app.description,
                    "tags": app.tags,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "provider": provider_id,
                "total": total,
                "page": page,
                "per_page": per_page,
                "total_pages": total_pages,
                "apps": json_apps,
            }))?
        );
        return Ok(());
    }

    println!("Provider: {} ({})", provider_name, provider_id);
    println!(
        "{:<45} {:<25} {:<10} DESCRIPTION",
        "FULL ID", "NAME", "CATEGORY"
    );
    println!("{}", "-".repeat(110));

    for app in &page_apps {
        let full_id = app.full_id();
        let truncated_desc = truncate_str(&app.description, 40);
        println!(
            "{:<45} {:<25} {:<10} {}",
            full_id, app.display_name, app.category, truncated_desc
        );
    }

    eprintln!();
    if total_pages > 1 {
        eprintln!(
            "Page {}/{} - showing {}-{} of {} apps",
            page,
            total_pages,
            start + 1,
            (start + page_apps.len()).min(total),
            total
        );
        if page < total_pages {
            eprintln!("Use --page {} to see the next page.", page + 1);
        }
    } else {
        eprintln!("Total: {} apps", total);
    }

    Ok(())
}

async fn run_app(
    app_str: String,
    input_arg: Option<String>,
    input_file: Option<PathBuf>,
    stream: bool,
    output: Option<PathBuf>,
    json: bool,
    load_env: bool,
) -> Result<()> {
    let app_id = AppId::parse(&app_str)?;

    if json && stream {
        anyhow::bail!("--json and --stream cannot be used together");
    }

    let input_str = if let Some(path) = input_file {
        std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read input file '{}': {}", path.display(), e))?
    } else if let Some(s) = input_arg {
        s
    } else {
        anyhow::bail!("Provide input via --input or --input-file");
    };

    let input: serde_json::Value = serde_json::from_str(&input_str)
        .map_err(|e| crate::error::InfsError::InvalidInput(format!("Invalid JSON input: {}", e)))?;

    let registry = build_registry();
    let provider = registry.find_provider(&app_id.provider)?;

    let app_config = config::load_config_with_env(load_env)?;
    let prov_config = app_config
        .providers
        .get(&app_id.provider)
        .cloned()
        .unwrap_or_default();

    provider.validate_config(&prov_config)?;

    if !json {
        eprintln!("Running {}/{}...", app_id.provider, app_id.app);
    }

    if stream {
        if provider.supports_streaming() {
            provider
                .stream_app(&app_id.app, input, &prov_config)
                .await?;
            return Ok(());
        }

        eprintln!(
            "Note: --stream is not supported for provider '{}', using non-streaming mode.",
            app_id.provider
        );
    }

    let response = provider.run_app(&app_id.app, input, &prov_config).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        match &response.output {
            crate::types::RunOutput::Text(text) => println!("{}", text),
            crate::types::RunOutput::ImageUrls(urls) => {
                if let Some(out_path) = &output {
                    save_images(urls, out_path).await?;
                } else {
                    for url in urls {
                        println!("{}", url);
                    }
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
    }

    Ok(())
}

async fn save_images(urls: &[String], base_path: &std::path::Path) -> Result<()> {
    let client = reqwest::Client::new();
    for (i, url) in urls.iter().enumerate() {
        let path = if urls.len() == 1 {
            base_path.to_path_buf()
        } else {
            let stem = base_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image");
            let ext = base_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e))
                .unwrap_or_default();
            base_path.with_file_name(format!("{}_{}{}", stem, i, ext))
        };

        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!(
                "Failed to download image from '{}': HTTP {}",
                url,
                resp.status()
            );
        }
        let bytes = resp.bytes().await?;
        std::fs::write(&path, &bytes)?;
        eprintln!("Saved image to: {}", path.display());
    }
    Ok(())
}

async fn show_app(app_str: String, json: bool, load_env: bool) -> Result<()> {
    let app_id = AppId::parse(&app_str)?;

    let registry = build_registry();
    let app_config = config::load_config_with_env(load_env)?;
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

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "id": app.id,
                "full_id": app.full_id(),
                "name": app.display_name,
                "category": app.category.to_string(),
                "description": app.description,
                "tags": app.tags,
            }))?
        );
    } else {
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
    }

    Ok(())
}

fn parse_category(s: &str) -> Result<AppCategory> {
    match s.to_lowercase().as_str() {
        "image" => Ok(AppCategory::Image),
        "llm" => Ok(AppCategory::Llm),
        "audio" => Ok(AppCategory::Audio),
        "video" => Ok(AppCategory::Video),
        "other" => Ok(AppCategory::Other),
        _ => Err(anyhow::anyhow!(
            "Unknown category: '{}'. Valid: image, llm, audio, video, other",
            s
        )),
    }
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let mut last_boundary = 0usize;
    let trim_to = max_chars.saturating_sub(3);

    for (char_count, (byte_idx, _ch)) in s.char_indices().enumerate() {
        if char_count == trim_to {
            last_boundary = byte_idx;
        }
        if char_count == max_chars {
            let mut out = s[..last_boundary].to_string();
            out.push_str("...");
            return out;
        }
    }

    s.to_string()
}
