use anyhow::{Result, anyhow, bail};
use bc_components::{Digest, DigestProvider};
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

    let content_env =
        io::parse_envelope(&args.compose.content).map_err(|err| {
            anyhow!("failed to load edition content envelope: {err}")
        })?;

    let provenance = io::parse_provenance_mark(&args.compose.provenance)
        .map_err(|err| anyhow!("failed to parse provenance mark: {err}"))?;
    if !provenance.is_genesis() {
        bail!("genesis editions must use a genesis provenance mark");
    }

    let info_cbor = provenance.info().ok_or_else(|| {
        anyhow!(
            "provenance mark info field must contain the content digest for genesis editions"
        )
    })?;
    let info_digest = Digest::try_from(info_cbor).map_err(|err| {
        anyhow!("provenance mark info is not a digest: {err}")
    })?;
    let content_digest = content_env.digest().into_owned();
    if info_digest != content_digest {
        bail!(
            "provenance mark info digest {} does not match content digest {}",
            info_digest.hex(),
            content_digest.hex()
        );
    }

    edition::compose::exec(args.compose)
}
