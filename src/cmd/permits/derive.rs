use anyhow::{Result, bail};
use clap::Args;

use crate::io;

/// Derive a public-key permit from recipient materials.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Recipient descriptor (XID document or public-keys UR).
    #[arg(long, value_name = "UR")]
    pub recipient: Vec<String>,
    /// Optional label to annotate the permit holder.
    #[arg(long, value_name = "XID")]
    pub label: Option<String>,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    for recipient in &args.recipient {
        let _ = io::parse_recipient_descriptor(recipient)?;
    }
    let _ = args.label;
    bail!("clubs-cli permits derive is not implemented yet")
}
