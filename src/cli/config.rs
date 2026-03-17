use crate::config;
use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigCommands {
    #[command(subcommand)]
    pub command: ConfigSubcommands,
}

#[derive(Subcommand)]
pub enum ConfigSubcommands {
    /// Show config file path
    Path,
}

pub async fn handle(cmd: ConfigCommands) -> Result<()> {
    match cmd.command {
        ConfigSubcommands::Path => show_path().await,
    }
}

async fn show_path() -> Result<()> {
    let config_path = config::get_config_path()?;
    let creds_path = config::get_credentials_path()?;

    println!("Config:      {}", config_path.display());
    println!("Credentials: {}", creds_path.display());

    Ok(())
}
