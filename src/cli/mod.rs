pub mod app;
pub mod config;
pub mod doctor;
pub mod provider;

use app::AppCommands;
use clap::{Parser, Subcommand};
use config::ConfigCommands;
use provider::ProviderCommands;

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
