use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Args;

use crate::io;

/// Arguments for composing and signing a club edition.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Publisher's XID document UR (must include signing keys).
    #[arg(long, value_name = "UR", global = true)]
    pub publisher: String,
    /// Content envelope UR for this edition.
    #[arg(long, value_name = "UR")]
    pub content: String,
    /// Provenance mark UR bound to this edition.
    #[arg(long, value_name = "UR")]
    pub provenance: String,
    /// Permit descriptors (XID or public-keys UR).
    #[arg(long = "permit", value_name = "UR")]
    pub permits: Vec<String>,
    /// Optional SSKR specifications (e.g. "2of3").
    #[arg(long = "sskr", value_name = "SPEC")]
    pub sskr: Vec<String>,
    /// Previous edition UR to enforce provenance ordering.
    #[arg(long, value_name = "UR")]
    pub previous: Option<String>,
    /// Output directory for generated artifacts.
    #[arg(long, value_name = "PATH")]
    pub out_dir: Option<PathBuf>,
    /// Print a human-readable summary in addition to UR outputs.
    #[arg(long)]
    pub summary: bool,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    let _publisher = io::parse_xid_document(&args.publisher)?;
    let _content = io::parse_envelope(&args.content)?;
    let _provenance = io::parse_provenance_mark(&args.provenance)?;

    let mut _recipients = Vec::new();
    for recipient in &args.permits {
        _recipients.push(io::parse_recipient_descriptor(recipient)?);
    }

    for descriptor in &_recipients {
        let _ = descriptor.public_keys();
        let _ = descriptor.xid_document();
    }

    if let Some(prev) = &args.previous {
        let _ = io::parse_envelope(prev)?;
    }

    if let Some(key_path) = &args.out_dir {
        let _ = key_path;
    }

    let _ = args.summary;
    let _ = &args.sskr;
    bail!("clubs-cli edition compose is not implemented yet")
}
