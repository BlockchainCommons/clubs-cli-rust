pub mod compose;
pub mod permits;
pub mod verify;

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
    /// Verify the signature and provenance of an edition.
    Verify(verify::CommandArgs),
    /// Extract sealed permits from an edition.
    Permits(permits::CommandArgs),
}

pub fn exec(args: CommandArgs) -> Result<()> {
    match args.command {
        Commands::Compose(args) => compose::exec(args),
        Commands::Verify(args) => verify::exec(args),
        Commands::Permits(args) => permits::exec(args),
    }
}
