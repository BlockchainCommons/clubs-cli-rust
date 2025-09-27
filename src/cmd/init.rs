use anyhow::{Result, bail};
use clap::Args;

use super::edition;
use crate::io;

/// Create the genesis edition for a single-publisher club.
#[derive(Debug, Args)]
pub struct CommandArgs {
    #[command(flatten)]
    pub compose: edition::compose::CommandArgs,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    if args.compose.previous.is_some() {
        bail!("genesis editions cannot specify a previous edition");
    }

    let provenance = io::parse_provenance_mark(&args.compose.provenance)?;
    if !provenance.is_genesis() {
        bail!("genesis editions must use a genesis provenance mark");
    }

    edition::compose::exec(args.compose)
}
