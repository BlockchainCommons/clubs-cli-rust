use anyhow::{Context, Result, bail};
use bc_components::{Digest, ReferenceProvider};
use bc_envelope::prelude::Envelope;
use bc_ur::UREncodable;
use clap::Args;
use clubs::{
    edition::Edition, provenance_mark_provider::ProvenanceMarkProvider,
    public_key_permit::PublicKeyPermit,
};

use crate::io::{self, RecipientDescriptor};

/// Arguments for inspecting an existing club edition.
#[derive(Debug, Args)]
pub struct CommandArgs {
    /// Edition UR to inspect.
    #[arg(long, value_name = "UR")]
    pub edition: String,
    /// Optional previous edition UR for provenance validation.
    #[arg(long, value_name = "UR")]
    pub previous: Option<String>,
    /// Publisher descriptor (XID document or public-keys UR) for signature
    /// verification.
    #[arg(long, value_name = "UR", alias = "verifier")]
    pub publisher: Option<String>,
    /// Emit summary details alongside normalized UR output.
    #[arg(long)]
    pub summary: bool,
    /// Also emit sealed permit URs in addition to the edition UR.
    #[arg(long)]
    pub emit_permits: bool,
}

pub fn exec(args: CommandArgs) -> Result<()> {
    let edition_env =
        io::parse_envelope(&args.edition).context("failed to parse edition")?;

    let verifier_descriptor = match args.publisher.as_ref() {
        Some(spec) => Some(
            io::parse_recipient_descriptor(spec)
                .context("failed to parse verifier input")?,
        ),
        None => None,
    };

    let verifier_keys = verifier_descriptor
        .as_ref()
        .map(|desc| desc.public_keys().clone());
    let verifier_label =
        verifier_descriptor.as_ref().and_then(descriptor_label);

    let (inner_envelope, signature_metadata, verified) =
        if let Some(ref keys) = verifier_keys {
            let (inner, metadata) = edition_env
                .verify_returning_metadata(keys)
                .context("failed to verify edition signature")?;
            (inner, Some(metadata), true)
        } else {
            (edition_env.clone().try_unwrap()?, None, false)
        };

    let edition = Edition::try_from(inner_envelope.clone())
        .context("edition payload is not a valid club edition")?;

    if let Some(descriptor) = verifier_descriptor.as_ref() {
        if let Some(expected_xid) = descriptor.member_xid() {
            if edition.club_xid != expected_xid {
                bail!(
                    "edition references club XID {} but verifier is {}",
                    edition.club_xid,
                    expected_xid
                );
            }
        }
    }

    let previous_details = if let Some(prev_spec) = args.previous.as_ref() {
        let prev_env = io::parse_envelope(prev_spec)
            .context("failed to parse previous edition")?;
        let prev_inner = if let Some(ref keys) = verifier_keys {
            prev_env
                .verify(keys)
                .context("failed to verify previous edition signature")?
        } else {
            prev_env.clone().try_unwrap()?
        };
        let prev_edition = Edition::try_from(prev_inner)
            .context("previous edition is not a valid club edition")?;
        if !prev_edition.precedes(&edition) {
            bail!("previous edition does not precede the inspected edition");
        }
        Some(prev_edition)
    } else {
        None
    };

    if args.summary {
        emit_summary(
            &edition,
            verified,
            verifier_label.as_deref(),
            previous_details.as_ref(),
            signature_metadata.as_ref(),
        );
    }

    println!("{}", edition_env.ur_string());
    if args.emit_permits {
        for permit in &edition.permits {
            if let PublicKeyPermit::Decode { sealed, .. } = permit {
                println!("{}", sealed.ur_string());
            }
        }
    }

    Ok(())
}

fn descriptor_label(descriptor: &RecipientDescriptor) -> Option<String> {
    if let Some(xid) = descriptor.member_xid() {
        Some(xid.to_string())
    } else {
        Some(descriptor.public_keys().reference().to_string())
    }
}

fn emit_summary(
    edition: &Edition,
    verified: bool,
    verifier_label: Option<&str>,
    previous: Option<&Edition>,
    signature_metadata: Option<&Envelope>,
) {
    let digest: Digest = edition.provisional_id();
    eprintln!("Edition summary:");
    eprintln!("  Club XID: {}", edition.club_xid);
    eprintln!(
        "  Provenance: seq {} ({})",
        edition.provenance.seq(),
        edition.provenance
    );
    if let Some(prev) = previous {
        eprintln!(
            "  Previous: seq {} ({})",
            prev.provenance.seq(),
            prev.provenance
        );
    }
    eprintln!("  Edition digest: {}", digest.short_description());

    match (verified, verifier_label) {
        (true, Some(label)) => {
            eprintln!("  Signature: verified with {label}");
        }
        (true, None) => {
            eprintln!("  Signature: verified");
        }
        (false, _) => {
            eprintln!("  Signature: not verified (no verifier provided)");
        }
    }

    if let Some(metadata) = signature_metadata {
        if !metadata.assertions().is_empty() {
            eprintln!(
                "  Signature metadata assertions: {}",
                metadata.assertions().len()
            );
        }
    }

    let permit_count = edition.permits.len();
    if permit_count > 0 {
        eprintln!("  Permits ({permit_count}):");
        for permit in &edition.permits {
            match permit {
                PublicKeyPermit::Decode { sealed, member_xid } => {
                    let label = member_xid
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| sealed.ur_string());
                    eprintln!("    - {label}");
                }
                PublicKeyPermit::Encode { .. } => {}
            }
        }
    } else {
        eprintln!("  Permits: none");
    }

    if edition.content.is_encrypted() {
        eprintln!("  Content: encrypted");
    } else if edition.content.is_wrapped() {
        eprintln!("  Content: wrapped cleartext");
    } else {
        eprintln!("  Content: cleartext");
    }
}
