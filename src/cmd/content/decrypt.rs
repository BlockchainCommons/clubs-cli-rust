use anyhow::{Result, bail};
use clap::Args;

use crate::io;

/// Decrypt edition content using permits, SSKR shards, or raw keys.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Edition UR containing the encrypted content.
    #[arg(long, value_name = "UR")]
    pub edition: String,
    /// Permit URs capable of unwrapping the content key.
    #[arg(long = "permit", value_name = "UR")]
    pub permits: Vec<String>,
    /// SSKR share URs for recovering the content key.
    #[arg(long = "sskr", value_name = "UR")]
    pub shards: Vec<String>,
    /// Symmetric key UR for decrypting the content directly.
    #[arg(long, value_name = "UR")]
    pub key: Option<String>,
    /// Emit decrypted envelope UR to stdout.
    #[arg(long)]
    pub emit_ur: bool,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    let _edition = io::parse_envelope(&args.edition)?;
    for permit in &args.permits {
        let _ = io::parse_sealed_message(permit)?;
    }
    for shard in &args.shards {
        let _ = io::parse_sskr_share(shard)?;
    }
    if let Some(key) = &args.key {
        let _ = io::parse_symmetric_key(key)?;
    }
    let _ = args.emit_ur;
    bail!("clubs-cli content decrypt is not implemented yet")
}
