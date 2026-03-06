pub mod provider;
pub mod app;
pub mod config;
pub mod doctor;

use clap::{Parser, Subcommand};
use provider::ProviderCommands;
use app::AppCommands;
use config::ConfigCommands;

#[derive(Parser)]
#[command(
    name = "infs",
    about = "A unified AI app runner",
    version,
    long_about = "infs lets you connect to AI providers and run models from a single CLI"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage AI providers
    Provider(ProviderCommands),
    /// List and run AI apps/models
    App(AppCommands),
    /// Manage configuration
    Config(ConfigCommands),
    /// Check system and provider health
    Doctor,
}
