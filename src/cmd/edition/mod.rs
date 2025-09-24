pub mod compose;
pub mod inspect;

use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Compose and sign an edition.
    Compose(compose::CommandArgs),
    /// Inspect and verify an edition.
    Inspect(inspect::CommandArgs),
}

pub fn exec(args: CommandArgs) -> Result<()> {
    match args.command {
        Commands::Compose(args) => compose::exec(args),
        Commands::Inspect(args) => inspect::exec(args),
    }
}
