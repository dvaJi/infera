mod auth;
mod catalog;
mod cli;
mod config;
mod error;
mod providers;
mod types;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Provider(cmd) => cli::provider::handle(cmd).await,
        Commands::App(cmd) => cli::app::handle(cmd).await,
        Commands::Config(cmd) => cli::config::handle(cmd).await,
        Commands::Doctor => cli::doctor::handle().await,
    }
}
