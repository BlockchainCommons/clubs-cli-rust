use anyhow::Result;
use clap::Args;

use super::edition;

/// Create the genesis edition for a single-publisher club.
#[derive(Debug, Args)]
pub struct CommandArgs {
    #[command(flatten)]
    pub compose: edition::compose::CommandArgs,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    edition::compose::exec(args.compose)
}
