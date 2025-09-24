pub mod decrypt;

use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Decrypt edition content using permits, SSKR shards, or raw keys.
    Decrypt(decrypt::CommandArgs),
}

pub fn exec(args: CommandArgs) -> Result<()> {
    match args.command {
        Commands::Decrypt(args) => decrypt::exec(args),
    }
}
