use crate::cli::Cli;
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

pub fn handle(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "infs", &mut std::io::stdout());
    Ok(())
}
