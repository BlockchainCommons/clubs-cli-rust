pub mod derive;

use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Derive a public-key permit from recipient materials.
    Derive(derive::CommandArgs),
}

pub fn exec(args: CommandArgs) -> Result<()> {
    match args.command {
        Commands::Derive(args) => derive::exec(args),
    }
}
