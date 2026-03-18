use crate::catalog::Catalog;
use crate::config;
use crate::providers::registry::build_registry;
use crate::types::AppId;
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
    /// List available apps/models
    List {
        /// Filter by category (image, llm, audio, video)
        #[arg(long)]
        category: Option<String>,
        /// Filter by provider
        #[arg(long)]
        provider: Option<String>,
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

pub async fn handle(cmd: AppCommands, json: bool) -> Result<()> {
    match cmd.command {
        AppSubcommands::List {
            category,
            provider,
            page,
            per_page,
        } => list_apps(category, provider, page, per_page, json).await,
        AppSubcommands::Run {
            app,
            input,
            input_file,
            stream,
            output,
        } => run_app(app, input, input_file, stream, output, json).await,
        AppSubcommands::Show { app } => show_app(app, json).await,
    }
}

async fn list_apps(
    category_filter: Option<String>,
    provider_filter: Option<String>,
    page: usize,
    per_page: usize,
    json: bool,
) -> Result<()> {
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

    let total = apps.len();

    // Apply pagination
    let page = page.max(1);
    let per_page = per_page.max(1);
    let start = (page - 1) * per_page;
    let page_apps: Vec<_> = apps.iter().skip(start).take(per_page).collect();
    let total_pages = total.div_ceil(per_page);

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
                "total": total,
                "page": page,
                "per_page": per_page,
                "total_pages": total_pages,
                "apps": json_apps,
            }))?
        );
    } else {
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
                "Page {}/{} — showing {}-{} of {} apps",
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
) -> Result<()> {
    let app_id = AppId::parse(&app_str)?;

    // Resolve input: --input takes precedence, --input-file as alternative
    let input_str = if let Some(path) = input_file {
        std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read input file '{}': {}", path.display(), e))?
    } else {
        input_arg.expect("--input is required when --input-file is not provided")
    };

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

    if !json {
        eprintln!("Running {}/{}...", app_id.provider, app_id.app);
    }

    // Streaming mode: print tokens as they arrive
    if stream {
        if provider.supports_streaming() {
            provider
                .stream_app(&app_id.app, input, &prov_config)
                .await?;
            return Ok(());
        } else {
            eprintln!(
                "Note: --stream is not supported for provider '{}', using non-streaming mode.",
                app_id.provider
            );
        }
    }

    let response = provider.run_app(&app_id.app, input, &prov_config).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        match &response.output {
            crate::types::RunOutput::Text(text) => {
                println!("{}", text);
            }
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

/// Download image URLs and save them to local files.
///
/// - Single image: saved to `base_path` as-is.
/// - Multiple images: saved to `{stem}_{i}{ext}` (e.g. `out_0.png`, `out_1.png`).
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

        let bytes = client.get(url).send().await?.bytes().await?;
        std::fs::write(&path, &bytes)?;
        eprintln!("Saved image to: {}", path.display());
    }
    Ok(())
}

async fn show_app(app_str: String, json: bool) -> Result<()> {
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
    let mut last_boundary = 0usize; // byte index of the trim point (max_chars - 3)
    let trim_to = max_chars.saturating_sub(3);

    for (char_count, (byte_idx, _ch)) in s.char_indices().enumerate() {
        if char_count == trim_to {
            last_boundary = byte_idx;
        }
        if char_count == max_chars {
            // Need to truncate — use the saved boundary
            let mut out = s[..last_boundary].to_string();
            out.push_str("...");
            return out;
        }
    }
    // String fits within max_chars — return as-is
    s.to_string()
}
