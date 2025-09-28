use anyhow::{Context, Result, bail};
use clap::Args;
use clubs::{
    edition::Edition, provenance_mark_provider::ProvenanceMarkProvider,
};

use crate::io;

/// Verify the signature and optional provenance of an edition.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Edition UR to verify.
    #[arg(long, value_name = "UR")]
    pub edition: String,
    /// Optional previous edition UR for provenance validation.
    #[arg(long, value_name = "UR")]
    pub previous: Option<String>,
    /// Publisher descriptor (XID document or public-keys UR) used for
    /// signature verification.
    #[arg(long, value_name = "UR")]
    pub publisher: String,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    let edition_env =
        io::parse_envelope(&args.edition).context("failed to parse edition")?;
    let publisher_descriptor = io::parse_recipient_descriptor(&args.publisher)
        .context("failed to parse publisher input")?;
    let publisher_keys = publisher_descriptor.public_keys().clone();

    let inner_envelope = edition_env
        .verify(&publisher_keys)
        .context("failed to verify edition signature")?;
    let edition = Edition::try_from(inner_envelope.clone())
        .context("edition payload is not a valid club edition")?;

    if let Some(expected_xid) = publisher_descriptor.member_xid() {
        if edition.club_xid != expected_xid {
            bail!(
                "edition references club XID {} but publisher descriptor is {}",
                edition.club_xid,
                expected_xid
            );
        }
    }

    if let Some(prev_spec) = args.previous.as_ref() {
        let prev_env = io::parse_envelope(prev_spec)
            .context("failed to parse previous edition")?;
        let prev_inner = prev_env
            .verify(&publisher_keys)
            .context("failed to verify previous edition signature")?;
        let prev_edition = Edition::try_from(prev_inner)
            .context("previous edition is not a valid club edition")?;
        if !prev_edition.precedes(&edition) {
            bail!("previous edition does not precede the verified edition");
        }
    }

    Ok(())
}
