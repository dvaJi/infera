pub mod app;
pub mod completions;
pub mod config;
pub mod doctor;
pub mod provider;
pub mod update;

use app::AppCommands;
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use config::ConfigCommands;
use provider::ProviderCommands;
use update::UpdateCommands;

#[derive(Parser)]
#[command(
    name = "infs",
    about = "A unified AI app runner",
    version,
    long_about = "infs lets you connect to AI providers and run models from a single CLI"
)]
pub struct Cli {
    /// Output results as JSON (machine-readable)
    #[arg(long, global = true)]
    pub json: bool,

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
    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, powershell, elvish)
        shell: Shell,
    },
    /// Self-update commands
    #[command(name = "self")]
    SelfCmd(UpdateCommands),
}
