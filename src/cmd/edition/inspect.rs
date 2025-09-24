use anyhow::{Result, bail};
use clap::Args;

use crate::io;

/// Arguments for inspecting an existing club edition.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Edition UR to inspect.
    #[arg(long, value_name = "UR")]
    pub edition: String,
    /// Optional previous edition UR for provenance validation.
    #[arg(long, value_name = "UR")]
    pub previous: Option<String>,
    /// Emit summary details alongside normalized UR output.
    #[arg(long)]
    pub summary: bool,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    let _edition = io::parse_envelope(&args.edition)?;
    if let Some(previous) = &args.previous {
        let _ = io::parse_envelope(previous)?;
    }
    let _ = args.summary;
    bail!("clubs-cli edition inspect is not implemented yet")
}
