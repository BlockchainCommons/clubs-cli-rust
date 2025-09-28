use anyhow::{Context, Result};
use bc_ur::UREncodable;
use clap::Args;
use clubs::{edition::Edition, public_key_permit::PublicKeyPermit};

use crate::io;

/// Arguments for extracting sealed permits from an edition.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Edition UR to inspect for permits.
    #[arg(long, value_name = "UR")]
    pub edition: String,
    /// Emit a human-readable summary to stderr.
    #[arg(long)]
    pub summary: bool,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    let edition_env =
        io::parse_envelope(&args.edition).context("failed to parse edition")?;
    let inner_envelope = edition_env
        .clone()
        .try_unwrap()
        .context("edition envelope is not directly accessible")?;
    let edition = Edition::try_from(inner_envelope)
        .context("edition payload is not a valid club edition")?;

    let mut extracted = 0usize;
    for permit in &edition.permits {
        if let PublicKeyPermit::Decode { sealed, .. } = permit {
            println!("{}", sealed.ur_string());
            extracted += 1;
        }
    }

    if args.summary {
        if extracted == 0 {
            eprintln!("Permits: none");
        } else {
            eprintln!("Permits extracted: {extracted}");
        }
    }

    Ok(())
}
